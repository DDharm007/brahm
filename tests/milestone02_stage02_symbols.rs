//! Stage 02 Milestone 02 Verification Test Suite
//! Multi-Language Symbol Extraction Engine & Symbol Index

use brahm::parser::{ParserRegistry, SupportedLanguage};
use brahm::symbols::{extract_and_index, SymbolKind, Visibility};

#[test]
fn test_01_rust_symbol_extraction() {
    let registry = ParserRegistry::new();
    let rust_code = r#"
/// Authentication and session manager.
pub struct AuthService {
    secret: String,
}

impl AuthService {
    /// Create a new session with an auth token.
    pub async fn login(&self, username: &str, token: String) -> Result<bool, Error> {
        true
    }

    fn internal_verify(&self) -> bool {
        false
    }
}

pub enum AuthRole {
    Admin,
    User,
}

pub trait Authenticator {
    fn authenticate(&self) -> bool;
}

pub type TokenAlias = String;
pub const MAX_RETRIES: u32 = 5;
"#;

    let parsed = registry
        .parse_source(rust_code, SupportedLanguage::Rust)
        .unwrap();
    let index = extract_and_index(&parsed);

    assert_eq!(index.get_by_kind(SymbolKind::Struct).len(), 1);
    let auth_struct = index.get_by_name("AuthService")[0];
    assert_eq!(auth_struct.visibility, Visibility::Public);
    assert!(auth_struct
        .doc_comment
        .as_ref()
        .unwrap()
        .contains("Authentication and session"));

    let methods = index.get_by_kind(SymbolKind::Method);
    assert_eq!(methods.len(), 2);

    let login_method = index.get_by_name("login")[0];
    assert_eq!(login_method.qualified_name, "AuthService::login");
    assert_eq!(login_method.visibility, Visibility::Public);
    assert!(login_method.is_async);
    assert!(login_method
        .return_type
        .as_ref()
        .unwrap()
        .contains("Result<bool, Error>"));
    assert_eq!(login_method.parameters.len(), 3); // self, username, token
    assert_eq!(login_method.parent_id, Some(auth_struct.id.clone()));

    let internal_verify = index.get_by_name("internal_verify")[0];
    assert_eq!(internal_verify.visibility, Visibility::Private);

    let enums = index.get_by_kind(SymbolKind::Enum);
    assert_eq!(enums.len(), 1);
    assert_eq!(enums[0].name, "AuthRole");

    let traits = index.get_by_kind(SymbolKind::Trait);
    assert_eq!(traits.len(), 1);
    assert_eq!(traits[0].name, "Authenticator");

    assert_eq!(index.get_by_kind(SymbolKind::TypeAlias).len(), 1);
    assert_eq!(index.get_by_kind(SymbolKind::Constant).len(), 1);
}

#[test]
fn test_02_javascript_typescript_symbol_extraction() {
    let registry = ParserRegistry::new();
    let ts_code = r#"
/**
 * User account repository.
 */
export class UserRepository {
    constructor(private db: Database) {}

    /**
     * Find user by unique email.
     */
    async findByEmail(email: string): Promise<User | null> {
        return null;
    }

    #hashPassword(raw: string): string {
        return "hashed";
    }
}

export interface User {
    id: string;
    email: string;
}

export type UserId = string;

export enum UserStatus {
    Active,
    Suspended,
}

export const formatUserName = (user: User): string => {
    return user.email;
};

export const API_VERSION = "2.0";
"#;

    let parsed = registry
        .parse_source(ts_code, SupportedLanguage::TypeScript)
        .unwrap();
    let index = extract_and_index(&parsed);

    let classes = index.get_by_kind(SymbolKind::Class);
    assert_eq!(classes.len(), 1);
    let user_repo = classes[0];
    assert_eq!(user_repo.visibility, Visibility::Public);
    assert!(user_repo
        .doc_comment
        .as_ref()
        .unwrap()
        .contains("User account repository"));

    let constructors = index.get_by_kind(SymbolKind::Constructor);
    assert_eq!(constructors.len(), 1);

    let find_by_email = index.get_by_name("findByEmail")[0];
    assert_eq!(find_by_email.qualified_name, "UserRepository.findByEmail");
    assert_eq!(find_by_email.visibility, Visibility::Public);
    assert!(find_by_email.is_async);
    assert_eq!(
        find_by_email.return_type.as_deref(),
        Some("Promise<User | null>")
    );

    let hash_pwd = index.get_by_name("#hashPassword")[0];
    assert_eq!(hash_pwd.visibility, Visibility::Private);

    let ifaces = index.get_by_kind(SymbolKind::Interface);
    assert_eq!(ifaces.len(), 1);
    assert_eq!(ifaces[0].name, "User");

    let types = index.get_by_kind(SymbolKind::TypeAlias);
    assert_eq!(types.len(), 1);

    let enums = index.get_by_kind(SymbolKind::Enum);
    assert_eq!(enums.len(), 1);

    // Arrow function assigned to const
    let arrow_fn = index.get_by_name("formatUserName")[0];
    assert_eq!(arrow_fn.kind, SymbolKind::Function);
    assert_eq!(arrow_fn.visibility, Visibility::Public);

    let consts = index.get_by_kind(SymbolKind::Constant);
    assert_eq!(consts.len(), 1);
    assert_eq!(consts[0].name, "API_VERSION");
}

#[test]
fn test_03_python_symbol_extraction() {
    let registry = ParserRegistry::new();
    let py_code = r#"
"""Module for cryptographic session handling."""

class SecureSession:
    """Manages secure HMAC authenticated sessions."""

    def __init__(self, key: str):
        self._key = key

    async def validate_token(self, token: str) -> bool:
        """Validates incoming token."""
        return True

    def _internal_refresh(self) -> None:
        pass

    def __private_clean(self) -> None:
        pass

def global_healthcheck() -> dict:
    return {"status": "ok"}
"#;

    let parsed = registry
        .parse_source(py_code, SupportedLanguage::Python)
        .unwrap();
    let index = extract_and_index(&parsed);

    let classes = index.get_by_kind(SymbolKind::Class);
    assert_eq!(classes.len(), 1);
    let session_class = classes[0];
    assert_eq!(session_class.visibility, Visibility::Public);
    assert_eq!(
        session_class.doc_comment.as_deref(),
        Some("Manages secure HMAC authenticated sessions.")
    );

    let ctors = index.get_by_kind(SymbolKind::Constructor);
    assert_eq!(ctors.len(), 1);
    assert_eq!(ctors[0].name, "__init__");

    let validate_token = index.get_by_name("validate_token")[0];
    assert_eq!(
        validate_token.qualified_name,
        "SecureSession.validate_token"
    );
    assert_eq!(validate_token.visibility, Visibility::Public);
    assert!(validate_token.is_async);
    assert_eq!(validate_token.return_type.as_deref(), Some("bool"));

    let internal_refresh = index.get_by_name("_internal_refresh")[0];
    assert_eq!(internal_refresh.visibility, Visibility::Internal);

    let private_clean = index.get_by_name("__private_clean")[0];
    assert_eq!(private_clean.visibility, Visibility::Private);

    let free_fn = index.get_by_name("global_healthcheck")[0];
    assert_eq!(free_fn.kind, SymbolKind::Function);
    assert_eq!(free_fn.visibility, Visibility::Public);
}

#[test]
fn test_04_go_symbol_extraction() {
    let registry = ParserRegistry::new();
    let go_code = r#"
package service

// Configuration holds server properties.
type Config struct {
    Port int
}

type TokenValidator interface {
    Validate(token string) bool
}

// NewServer initializes server instance.
func NewServer(cfg Config) *Server {
    return &Server{}
}

func (s *Server) HandleRequest(req string) (string, error) {
    return req, nil
}

func (s *Server) privateWorker() {
}

const DefaultPort = 8080
"#;

    let parsed = registry
        .parse_source(go_code, SupportedLanguage::Go)
        .unwrap();
    let index = extract_and_index(&parsed);

    let structs = index.get_by_kind(SymbolKind::Struct);
    assert_eq!(structs.len(), 1);
    assert_eq!(structs[0].name, "Config");
    assert_eq!(structs[0].visibility, Visibility::Public);

    let ifaces = index.get_by_kind(SymbolKind::Interface);
    assert_eq!(ifaces.len(), 1);
    assert_eq!(ifaces[0].name, "TokenValidator");

    let fns = index.get_by_kind(SymbolKind::Function);
    assert_eq!(fns.len(), 1);
    assert_eq!(fns[0].name, "NewServer");
    assert_eq!(fns[0].visibility, Visibility::Public);

    let methods = index.get_by_kind(SymbolKind::Method);
    assert_eq!(methods.len(), 2);

    let handle_req = index.get_by_name("HandleRequest")[0];
    assert_eq!(handle_req.qualified_name, "Server.HandleRequest");
    assert_eq!(handle_req.visibility, Visibility::Public);

    let worker = index.get_by_name("privateWorker")[0];
    assert_eq!(worker.visibility, Visibility::Internal);

    let consts = index.get_by_kind(SymbolKind::Constant);
    assert_eq!(consts.len(), 1);
    assert_eq!(consts[0].name, "DefaultPort");
}

#[test]
fn test_05_java_symbol_extraction() {
    let registry = ParserRegistry::new();
    let java_code = r#"
package com.brahm.security;

/**
 * Enterprise token authenticator.
 */
public class TokenAuthenticator {
    private final String secretKey;

    public TokenAuthenticator(String secretKey) {
        this.secretKey = secretKey;
    }

    public boolean authenticate(String token) {
        return true;
    }

    private void cleanup() {
    }
}
"#;

    let parsed = registry
        .parse_source(java_code, SupportedLanguage::Java)
        .unwrap();
    let index = extract_and_index(&parsed);

    let classes = index.get_by_kind(SymbolKind::Class);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].name, "TokenAuthenticator");
    assert_eq!(classes[0].visibility, Visibility::Public);

    let ctors = index.get_by_kind(SymbolKind::Constructor);
    assert_eq!(ctors.len(), 1);
    assert_eq!(ctors[0].visibility, Visibility::Public);

    let methods = index.get_by_kind(SymbolKind::Method);
    assert_eq!(methods.len(), 2);

    let auth_method = index.get_by_name("authenticate")[0];
    assert_eq!(auth_method.visibility, Visibility::Public);
    assert_eq!(auth_method.return_type.as_deref(), Some("boolean"));

    let cleanup_method = index.get_by_name("cleanup")[0];
    assert_eq!(cleanup_method.visibility, Visibility::Private);
}

#[test]
fn test_06_c_cpp_symbol_extraction() {
    let registry = ParserRegistry::new();
    let cpp_code = r#"
namespace brahm {
    class SecurityContext {
    public:
        SecurityContext();
        bool isAuthorized() const;
    };

    struct SessionData {
        int userId;
    };

    enum class AccessLevel {
        None,
        Read,
        Admin
    };
}

int initialize_subsystem(int flags) {
    return 0;
}
"#;

    let parsed = registry
        .parse_source(cpp_code, SupportedLanguage::Cpp)
        .unwrap();
    let index = extract_and_index(&parsed);

    let modules = index.get_by_kind(SymbolKind::Module);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].name, "brahm");

    let classes = index.get_by_kind(SymbolKind::Class);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].qualified_name, "brahm::SecurityContext");

    let structs = index.get_by_kind(SymbolKind::Struct);
    assert_eq!(structs.len(), 1);
    assert_eq!(structs[0].qualified_name, "brahm::SessionData");

    let enums = index.get_by_kind(SymbolKind::Enum);
    assert_eq!(enums.len(), 1);
    assert_eq!(enums[0].qualified_name, "brahm::AccessLevel");

    let fns = index.get_by_kind(SymbolKind::Function);
    assert_eq!(fns.len(), 1);
    assert_eq!(fns[0].name, "initialize_subsystem");
}

#[test]
fn test_07_stable_symbol_id_determinism() {
    let registry = ParserRegistry::new();
    let code = "pub fn compute_hash(val: u64) -> u64 { val }";

    let parsed1 = registry
        .parse_source(code, SupportedLanguage::Rust)
        .unwrap();
    let index1 = extract_and_index(&parsed1);

    let parsed2 = registry
        .parse_source(code, SupportedLanguage::Rust)
        .unwrap();
    let index2 = extract_and_index(&parsed2);

    assert_eq!(index1.len(), 1);
    assert_eq!(index2.len(), 1);
    assert_eq!(index1.all()[0].id, index2.all()[0].id);
    assert!(index1.all()[0].id.starts_with("sym_"));
}

#[test]
fn test_08_symbol_index_queries_and_summary() {
    let registry = ParserRegistry::new();
    let code = r#"
pub struct Engine;
pub struct Driver;
pub fn start_all() {}
pub fn stop_all() {}
"#;
    let parsed = registry
        .parse_source(code, SupportedLanguage::Rust)
        .unwrap();
    let index = extract_and_index(&parsed);

    assert_eq!(index.len(), 4);
    assert!(!index.is_empty());

    let structs = index.get_by_kind(SymbolKind::Struct);
    assert_eq!(structs.len(), 2);

    let fns = index.get_by_kind(SymbolKind::Function);
    assert_eq!(fns.len(), 2);

    let summary = index.summary();
    assert_eq!(summary.total_symbols, 4);
    assert_eq!(*summary.by_kind.get("struct").unwrap(), 2);
    assert_eq!(*summary.by_kind.get("function").unwrap(), 2);
    assert_eq!(*summary.by_visibility.get("public").unwrap(), 4);
}
