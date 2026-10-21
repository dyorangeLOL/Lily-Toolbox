use crate::{
    contracts::{DomainCommand, Request, Response},
    database::Database,
    paths::DataPaths,
    permissions::HostCaller,
    AppError,
};
use serde_json::{json, Value};

pub struct CoreService {
    database: Database,
}

impl CoreService {
    pub fn open(paths: &DataPaths) -> Result<Self, AppError> {
        Ok(Self {
            database: Database::open(&paths.database)?,
        })
    }

    pub fn dispatch(&self, caller: HostCaller, request: Request) -> Response {
        let mut result = self.execute(caller, &request);
        let safe_id = if crate::contracts::valid_id(&request.request_id) {
            request.request_id.as_str()
        } else {
            "invalid"
        };
        let subject = if caller == HostCaller::LocalShell {
            "toolbox.shell"
        } else {
            "remote"
        };
        if let Err(error) = self.database.audit(
            subject,
            safe_id,
            request.params.name(),
            result
                .as_ref()
                .map_or_else(|e| e.code.as_str(), |_| "success"),
        ) {
            result = Err(error);
        }
        Response::from_result(request.request_id, result)
    }

    fn execute(&self, caller: HostCaller, request: &Request) -> Result<Value, AppError> {
        caller.require_local()?;
        request.validate()?;
        // No request-derived fields enter an audit record before validation.
        match &request.params {
            DomainCommand::ModulesList { .. } => self
                .database
                .modules()
                .and_then(|m| serde_json::to_value(m).map_err(|_| AppError::new("INVALID_DATA"))),
            DomainCommand::ModulesCapabilities { .. } => Ok(
                json!({"available":["modules.list","modules.capabilities"],"third_party_execution":false}),
            ),
            DomainCommand::FilesList { path } => {
                self.database.authorize_file(
                    caller,
                    "toolbox.files",
                    crate::modules::Capability::FileRead,
                    std::path::Path::new(path),
                )?;
                Err(AppError::new("NOT_IMPLEMENTED"))
            }
            DomainCommand::BackupVerify { path } => {
                self.database.authorize_file(
                    caller,
                    "toolbox.backup",
                    crate::modules::Capability::FileRead,
                    std::path::Path::new(path),
                )?;
                Err(AppError::new("NOT_IMPLEMENTED"))
            }
            _ => Err(AppError::new("NOT_IMPLEMENTED")),
        }
    }
}
