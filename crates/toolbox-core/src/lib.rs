use serde::{Deserialize, Serialize};

pub mod contracts;
pub mod database;
pub mod modules;
pub mod paths;
pub mod permissions;
pub mod service;

/// WF-01 health DTO only; WF-03 will add versioned domain contracts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HealthStatus {
    pub api_version: u32,
    pub version: String,
    pub runtime: String,
}

pub fn health() -> HealthStatus {
    HealthStatus {
        api_version: 1,
        version: env!("CARGO_PKG_VERSION").to_string(),
        runtime: "tauri-rust".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: &str) -> Self {
        let (message, retryable) = match code {
            "ACCESS_DENIED" => ("The caller or resource is not authorized", false),
            "INVALID_REQUEST" => ("The command does not match API version 1", false),
            "NOT_IMPLEMENTED" => ("This domain operation is not available yet", false),
            "DATA_DIRECTORY_NOT_WRITABLE" => ("The selected data directory is not writable", false),
            "MIGRATION_FAILED" => (
                "Database migration failed; no pending migration was committed",
                false,
            ),
            "SCHEMA_MISMATCH" => (
                "Database version or migration checksum is unsupported",
                false,
            ),
            "DATABASE_BUSY" => ("Database is busy; retry later", true),
            "INVALID_DATA" => ("Stored data failed validation", false),
            _ => ("Local storage operation failed", false),
        };
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
        }
    }

    pub fn diagnostic_failed() -> Self {
        Self {
            code: "DIAGNOSTIC_WRITE_FAILED".into(),
            message: "Unable to write the requested smoke-test report".into(),
            retryable: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_matches_shared_contract_fixture() {
        let fixture: HealthStatus = serde_json::from_str(include_str!(
            "../../../packages/shared-types/fixtures/health.json"
        ))
        .expect("valid shared fixture");
        assert_eq!(health(), fixture);
    }

    #[test]
    fn rejects_unknown_health_fields() {
        assert!(serde_json::from_str::<HealthStatus>(
            r#"{"api_version":1,"version":"0.1.0","runtime":"tauri-rust","token":"secret"}"#
        )
        .is_err());
    }

    #[test]
    fn error_does_not_disclose_device_paths() {
        let error = serde_json::to_string(&AppError::diagnostic_failed()).unwrap();
        assert!(error.contains("DIAGNOSTIC_WRITE_FAILED"));
        assert!(!error.contains("C:\\"));
    }
}
