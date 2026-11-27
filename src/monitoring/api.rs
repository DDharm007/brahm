//! Optional Authenticated Local Monitoring API (Milestone 06.7).
//!
//! Exposes authorized read-only monitoring endpoints via `axum`.
//! Security controls:
//! - Binds strictly to `127.0.0.1` by default (explicit `allow_remote` required otherwise)
//! - Requires valid Bearer token authentication on intelligence endpoints
//! - Enforces fixed-window request rate limiting and parameter validation
//! - Never exposes credentials or source code contents

use axum::{
    extract::{Query, Request, State},
    http::{
        header::{
            ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, ORIGIN,
        },
        HeaderMap, HeaderValue, Method, StatusCode,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::storage::{EventStore, SqliteEventStore};
use crate::deception::decoy::DecoyRegistry;
use crate::deception::InspectionSessionManager;
use crate::errors::{BrahmError, Result};
use crate::integration::RepositoryRecord;

/// Runtime configuration for the optional local monitoring API.
#[derive(Debug, Clone)]
pub struct MonitoringApiConfig {
    pub bind_addr: SocketAddr,
    pub allow_remote: bool,
    pub auth_token: String,
    pub rate_limit_per_minute: usize,
    pub state_dir: PathBuf,
}

impl MonitoringApiConfig {
    /// Validate safe binding and non-empty authentication token.
    pub fn validate(&self) -> Result<()> {
        if !self.bind_addr.ip().is_loopback() && !self.allow_remote {
            return Err(BrahmError::Unauthorized(format!(
                "Security policy: refusing non-loopback bind '{}' without explicit allow_remote=true",
                self.bind_addr
            )));
        }
        if self.auth_token.trim().len() < 16 {
            return Err(BrahmError::Config(
                "Monitoring API requires an auth_token of at least 16 characters".to_string(),
            ));
        }
        Ok(())
    }
}

/// Shared state injected into Axum route handlers.
#[derive(Clone)]
pub struct ApiState {
    pub store: SqliteEventStore,
    pub config: MonitoringApiConfig,
    pub request_timestamps: Arc<Mutex<VecDeque<Instant>>>,
}

#[derive(Debug, Deserialize)]
pub struct EventQueryParams {
    pub repository_id: Option<String>,
    pub session_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: String,
    pub status: u16,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

/// Construct the authenticated Axum router for Milestone 06.7.
pub fn build_monitoring_router(state: ApiState) -> Router {
    Router::new()
        .route("/api/v1/health", get(handle_health))
        .route("/api/v1/repositories", get(handle_repositories))
        .route("/api/v1/architecture", get(handle_architecture))
        .route("/api/v1/deception", get(handle_deception))
        .route("/api/v1/events", get(handle_events))
        .route("/api/v1/sessions", get(handle_sessions))
        .route("/api/v1/decoys", get(handle_decoys))
        .route("/api/v1/integrity", get(handle_integrity))
        .route("/api/v1/reports", get(handle_reports))
        .route("/api/v1/rating", get(handle_rating))
        .route("/api/v1/experiment", get(handle_experiment))
        .with_state(state)
        .layer(middleware::from_fn(localhost_cors_middleware))
}

async fn localhost_cors_middleware(req: Request, next: Next) -> Response {
    let origin = req
        .headers()
        .get(ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let allowed_origin = origin.as_deref().filter(|o| is_localhost_origin(o));

    if req.method() == Method::OPTIONS {
        let mut resp = StatusCode::NO_CONTENT.into_response();
        if let Some(orig) = allowed_origin {
            if let Ok(val) = HeaderValue::from_str(orig) {
                resp.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, val);
                resp.headers_mut().insert(
                    ACCESS_CONTROL_ALLOW_HEADERS,
                    HeaderValue::from_static("Authorization, Content-Type"),
                );
                resp.headers_mut().insert(
                    ACCESS_CONTROL_ALLOW_METHODS,
                    HeaderValue::from_static("GET, OPTIONS"),
                );
            }
        }
        return resp;
    }

    let orig_owned = allowed_origin.map(|s| s.to_string());
    let mut resp = next.run(req).await;
    if let Some(orig) = orig_owned {
        if let Ok(val) = HeaderValue::from_str(&orig) {
            resp.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, val);
            resp.headers_mut().insert(
                ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("Authorization, Content-Type"),
            );
            resp.headers_mut().insert(
                ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("GET, OPTIONS"),
            );
        }
    }
    resp
}

fn is_localhost_origin(origin: &str) -> bool {
    if origin == "http://localhost" || origin == "http://127.0.0.1" {
        return true;
    }
    if let Some(port) = origin
        .strip_prefix("http://localhost:")
        .or_else(|| origin.strip_prefix("http://127.0.0.1:"))
    {
        return !port.is_empty() && port.chars().all(|c| c.is_ascii_digit());
    }
    false
}

/// Bind and serve the Axum monitoring API asynchronously until shutdown.
pub async fn serve_monitoring_api(
    config: MonitoringApiConfig,
    store: SqliteEventStore,
) -> Result<()> {
    config.validate()?;
    let addr = config.bind_addr;
    let state = ApiState {
        store,
        config,
        request_timestamps: Arc::new(Mutex::new(VecDeque::new())),
    };
    let app = build_monitoring_router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(BrahmError::Io)?;
    axum::serve(listener, app)
        .await
        .map_err(|e| BrahmError::Monitoring(format!("Axum server error: {}", e)))?;
    Ok(())
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            status,
            message: msg.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                error: self.message,
                status: self.status.as_u16(),
            }),
        )
            .into_response()
    }
}

fn check_auth_and_rate(state: &ApiState, headers: &HeaderMap) -> std::result::Result<(), ApiError> {
    // 1. Constant-time Bearer token check
    let auth_hdr = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let provided = auth_hdr.strip_prefix("Bearer ").unwrap_or("");
    if provided.is_empty()
        || !constant_time_eq(provided.as_bytes(), state.config.auth_token.as_bytes())
    {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "Missing or invalid Bearer authentication token",
        ));
    }

    // 2. Sliding-window 60-second rate limiter
    let mut ts = state.request_timestamps.lock().map_err(|_| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal rate limiter error",
        )
    })?;
    let now = Instant::now();
    let window = Duration::from_secs(60);
    while let Some(&front) = ts.front() {
        if now.duration_since(front) > window {
            ts.pop_front();
        } else {
            break;
        }
    }
    if ts.len() >= state.config.rate_limit_per_minute {
        return Err(ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded",
        ));
    }
    ts.push_back(now);
    Ok(())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

async fn handle_health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn handle_repositories(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<Vec<RepositoryRecord>>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    let cfg_path = state.config.state_dir.join("BRAHM_REPOSITORY_CONFIG.json");
    if cfg_path.is_file() {
        let raw = std::fs::read_to_string(&cfg_path)
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let rec: RepositoryRecord = serde_json::from_str(&raw)
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(vec![rec]))
    } else {
        Ok(Json(Vec::new()))
    }
}

async fn handle_events(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(q): Query<EventQueryParams>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    if let Some(lim) = q.limit {
        if lim == 0 || lim > 1000 {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "Query parameter 'limit' must be between 1 and 1000",
            ));
        }
    }
    let limit = q.limit.unwrap_or(100);
    let events = state
        .store
        .list_events(q.repository_id.as_deref(), q.session_id.as_deref(), limit)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(events).unwrap_or_default()))
}

async fn handle_sessions(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    let mgr = InspectionSessionManager::new(state.config.state_dir.join("sessions"));
    let sessions = mgr
        .list_sessions()
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(sessions).unwrap_or_default()))
}

async fn handle_decoys(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    let reg_path = state
        .config
        .state_dir
        .join("decoys/BRAHM_DECOY_REGISTRY.json");
    if reg_path.is_file() {
        let reg = DecoyRegistry::load_from_path(&reg_path)
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(
            serde_json::to_value(reg.artifacts()).unwrap_or_default(),
        ))
    } else {
        Ok(Json(serde_json::json!([])))
    }
}

async fn handle_integrity(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    let int_path = state.config.state_dir.join("BRAHM_INTEGRITY_REPORT.json");
    if int_path.is_file() {
        let raw = std::fs::read_to_string(&int_path)
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let val: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
        Ok(Json(val))
    } else {
        Ok(Json(
            serde_json::json!({ "status": "no_integrity_check_recorded" }),
        ))
    }
}

async fn handle_reports(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    read_state_json(
        &state.config.state_dir.join("BRAHM_MONITORING_REPORT.json"),
        serde_json::json!({ "status": "no_monitoring_report_generated" }),
    )
}

async fn handle_architecture(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    read_state_json(
        &state.config.state_dir.join("BRAHM_ARCHITECTURE.json"),
        serde_json::json!({ "status": "no_architecture_report_generated" }),
    )
}

async fn handle_deception(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    let mgr = InspectionSessionManager::new(state.config.state_dir.join("sessions"));
    let sessions = mgr
        .list_sessions()
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Some(latest) = sessions.last() {
        let pol_path = latest.state_dir.join("BRAHM_DECEPTION_POLICY.json");
        let manifest_path = latest.state_dir.join("BRAHM_INSPECTION_MANIFEST.json");
        let policy = if pol_path.is_file() {
            std::fs::read_to_string(&pol_path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        } else {
            None
        };
        let manifest = if manifest_path.is_file() {
            std::fs::read_to_string(&manifest_path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        } else {
            None
        };
        Ok(Json(serde_json::json!({
            "active_session": latest,
            "policy": policy,
            "inspection_manifest": manifest,
        })))
    } else {
        Ok(Json(
            serde_json::json!({ "status": "no_deception_sessions" }),
        ))
    }
}

async fn handle_rating(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    let candidates = [
        state.config.state_dir.join("BRAHM_RATING.json"),
        PathBuf::from("reports/BRAHM_RATING.json"),
    ];
    for p in candidates {
        if p.is_file() {
            return read_state_json(&p, serde_json::json!({}));
        }
    }
    Ok(Json(
        serde_json::json!({ "status": "no_rating_calculated" }),
    ))
}

async fn handle_experiment(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    check_auth_and_rate(&state, &headers)?;
    let candidates = [
        state.config.state_dir.join("BRAHM_SCORECARD.json"),
        PathBuf::from("reports/BRAHM_SCORECARD.json"),
    ];
    for p in candidates {
        if p.is_file() {
            return read_state_json(&p, serde_json::json!({}));
        }
    }
    Ok(Json(
        serde_json::json!({ "status": "no_experiment_scorecard" }),
    ))
}

fn read_state_json(
    path: &std::path::Path,
    fallback: serde_json::Value,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    if path.is_file() {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let val: serde_json::Value = serde_json::from_str(&raw).unwrap_or(fallback);
        Ok(Json(val))
    } else {
        Ok(Json(fallback))
    }
}
