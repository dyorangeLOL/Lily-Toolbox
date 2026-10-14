use serde::{Deserialize, Serialize};

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
