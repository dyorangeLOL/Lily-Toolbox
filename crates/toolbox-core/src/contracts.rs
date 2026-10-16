use crate::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const API_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CallerContext {
    // A claim, never an authorization credential. Host identity is separate.
    pub module_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub request_id: String,
    pub api_version: u32,
    pub caller_context: CallerContext,
    pub params: DomainCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "command", content = "args", deny_unknown_fields)]
pub enum DomainCommand {
    #[serde(rename = "modules.list")]
    ModulesList {},
    #[serde(rename = "modules.capabilities")]
    ModulesCapabilities {},
    #[serde(rename = "apps.list")]
    AppsList {},
    #[serde(rename = "browser.listProfiles")]
    BrowserListProfiles {},
    #[serde(rename = "files.list")]
    FilesList { path: String },
    #[serde(rename = "search.query")]
    SearchQuery { query: String },
    #[serde(rename = "convert.capabilities")]
    ConvertCapabilities {},
    #[serde(rename = "backup.verify")]
    BackupVerify { path: String },
}

impl DomainCommand {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ModulesList { .. } => "modules.list",
            Self::ModulesCapabilities { .. } => "modules.capabilities",
            Self::AppsList { .. } => "apps.list",
            Self::BrowserListProfiles { .. } => "browser.listProfiles",
            Self::FilesList { .. } => "files.list",
            Self::SearchQuery { .. } => "search.query",
            Self::ConvertCapabilities { .. } => "convert.capabilities",
            Self::BackupVerify { .. } => "backup.verify",
        }
    }
}

impl Request {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.api_version != API_VERSION
            || !valid_id(&self.request_id)
            || self.caller_context.module_id != "toolbox.shell"
        {
            return Err(AppError::new("INVALID_REQUEST"));
        }
        Ok(())
    }
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

// Separate variants reject contradictory success/data/error combinations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged, deny_unknown_fields)]
pub enum Response {
    Success {
        request_id: String,
        api_version: u32,
        success: True,
        data: Value,
    },
    Failure {
        request_id: String,
        api_version: u32,
        success: False,
        error: AppError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "bool", into = "bool")]
pub struct True;
impl TryFrom<bool> for True {
    type Error = &'static str;
    fn try_from(v: bool) -> Result<Self, Self::Error> {
        if v {
            Ok(Self)
        } else {
            Err("expected true")
        }
    }
}
impl From<True> for bool {
    fn from(_: True) -> bool {
        true
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "bool", into = "bool")]
pub struct False;
impl TryFrom<bool> for False {
    type Error = &'static str;
    fn try_from(v: bool) -> Result<Self, Self::Error> {
        if !v {
            Ok(Self)
        } else {
            Err("expected false")
        }
    }
}
impl From<False> for bool {
    fn from(_: False) -> bool {
        false
    }
}

impl Response {
    pub fn from_result(id: String, result: Result<Value, AppError>) -> Self {
        match result {
            Ok(data) => Self::Success {
                request_id: id,
                api_version: API_VERSION,
                success: True,
                data,
            },
            Err(error) => Self::Failure {
                request_id: id,
                api_version: API_VERSION,
                success: False,
                error,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JobAccepted {
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "topic", content = "data", deny_unknown_fields)]
pub enum EventPayload {
    #[serde(rename = "convert.jobChanged")]
    JobChanged {
        job_id: String,
        state: JobState,
        progress_percent: u8,
    },
    #[serde(rename = "permission.changed")]
    PermissionChanged { grant_id: String, revoked: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub event_id: String,
    pub timestamp: u64,
    pub source_module: String,
    pub schema_version: u32,
    pub payload: EventPayload,
}

impl Event {
    pub fn validate(&self) -> Result<(), AppError> {
        let payload_valid = match &self.payload {
            EventPayload::JobChanged {
                job_id,
                progress_percent,
                ..
            } => {
                valid_id(job_id)
                    && *progress_percent <= 100
                    && self.source_module == "toolbox.convert"
            }
            EventPayload::PermissionChanged { grant_id, .. } => {
                valid_id(grant_id) && self.source_module == "toolbox.modules"
            }
        };
        if self.schema_version == 1
            && valid_id(&self.event_id)
            && self.timestamp > 0
            && payload_valid
        {
            Ok(())
        } else {
            Err(AppError::new("INVALID_REQUEST"))
        }
    }
}
