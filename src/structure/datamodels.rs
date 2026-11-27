//! Detection of database models, ORM entities, and schema migration scripts.

use std::path::Path;

use super::types::{DataModel, DataModelKind};
use crate::parser::ParsedSource;

/// Detector responsible for locating persistent database models and migration files.
#[derive(Debug, Default, Clone)]
pub struct DataModelDetector;

impl DataModelDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect ORM models and schema declarations from a parsed source file.
    pub fn detect_models(&self, parsed: &ParsedSource) -> Vec<DataModel> {
        let file_path = parsed.file_path().unwrap_or(Path::new("unknown"));
        self.detect_from_text(file_path, &parsed.source)
    }

    /// Detect ORM models and database schemas directly from file path and raw text.
    pub fn detect_from_text(&self, file_path: &Path, source: &str) -> Vec<DataModel> {
        let mut models = Vec::new();
        let norm_path = file_path.to_string_lossy().replace('\\', "/");

        // 1. Relational SQL Migration & Schema files
        if norm_path.ends_with(".sql") {
            if norm_path.contains("/migration")
                || norm_path.contains("/db/")
                || norm_path.contains("/sql/")
            {
                let name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("migration.sql")
                    .to_string();

                models.push(DataModel {
                    name,
                    kind: DataModelKind::SqlMigration,
                    file_path: file_path.to_path_buf(),
                    span: None,
                    framework_or_orm: "SQL Migration".to_string(),
                    table_name: None,
                    fields: Vec::new(),
                });
                return models;
            }

            // Otherwise parse CREATE TABLE declarations in .sql schema files
            for line in source.lines() {
                let upper = line.trim().to_ascii_uppercase();
                if let Some(after_create) = upper.strip_prefix("CREATE TABLE ") {
                    let clean_after = after_create.trim_start_matches("IF NOT EXISTS ").trim();
                    let raw_table = clean_after
                        .split(['(', ' ', '\t'])
                        .next()
                        .unwrap_or("")
                        .trim_matches(['"', '`', '[', ']', ';'])
                        .to_ascii_lowercase();
                    if !raw_table.is_empty() {
                        models.push(DataModel {
                            name: raw_table.clone(),
                            kind: DataModelKind::SchemaDefinition,
                            file_path: file_path.to_path_buf(),
                            span: None,
                            framework_or_orm: "SQL DDL".to_string(),
                            table_name: Some(raw_table),
                            fields: Vec::new(),
                        });
                    }
                }
            }
            return models;
        }

        // 2. Prisma schema models
        if norm_path.ends_with(".prisma") {
            for line in source.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("model ") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let model_name = parts[1].to_string();
                        models.push(DataModel {
                            name: model_name.clone(),
                            kind: DataModelKind::OrmEntity,
                            file_path: file_path.to_path_buf(),
                            span: None,
                            framework_or_orm: "Prisma".to_string(),
                            table_name: Some(model_name.to_ascii_lowercase()),
                            fields: Vec::new(),
                        });
                    }
                }
            }
            return models;
        }

        // 3. Python ORM models (Django `models.Model`, SQLAlchemy `Base`)
        if source.contains("models.Model") || source.contains("(Base)") {
            for line in source.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("class ")
                    && (trimmed.contains("models.Model") || trimmed.contains("(Base)"))
                {
                    let name_part = trimmed
                        .trim_start_matches("class ")
                        .split('(')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !name_part.is_empty() {
                        let orm = if trimmed.contains("models.Model") {
                            "Django ORM"
                        } else {
                            "SQLAlchemy"
                        };

                        models.push(DataModel {
                            name: name_part.to_string(),
                            kind: DataModelKind::OrmEntity,
                            file_path: file_path.to_path_buf(),
                            span: None,
                            framework_or_orm: orm.to_string(),
                            table_name: Some(name_part.to_ascii_lowercase()),
                            fields: Vec::new(),
                        });
                    }
                }
            }
        }

        // 4. TypeORM entities in TypeScript (`@Entity()`)
        if source.contains("@Entity") {
            for line in source.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("export class ") || trimmed.starts_with("class ") {
                    let name_part = trimmed
                        .trim_start_matches("export ")
                        .trim_start_matches("class ")
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim();

                    if !name_part.is_empty() {
                        models.push(DataModel {
                            name: name_part.to_string(),
                            kind: DataModelKind::OrmEntity,
                            file_path: file_path.to_path_buf(),
                            span: None,
                            framework_or_orm: "TypeORM".to_string(),
                            table_name: Some(name_part.to_ascii_lowercase()),
                            fields: Vec::new(),
                        });
                    }
                }
            }
        }

        models
    }
}
