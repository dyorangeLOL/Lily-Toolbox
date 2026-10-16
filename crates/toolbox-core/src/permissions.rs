use crate::{
    database::{storage_error, Database},
    modules::Capability,
    paths::scoped_existing_path,
    AppError,
};
use rusqlite::params;
use std::path::{Path, PathBuf};

// These identities are constructed by native code. Never deserialize from IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostCaller {
    LocalShell,
    Remote,
}

impl HostCaller {
    pub fn require_local(self) -> Result<(), AppError> {
        if self == Self::LocalShell {
            Ok(())
        } else {
            Err(AppError::new("ACCESS_DENIED"))
        }
    }
}

impl Database {
    // Internal native entry point after explicit user consent; not an IPC command.
    pub fn grant_file_scope(
        &self,
        id: &str,
        module: &str,
        capability: Capability,
        scope: &Path,
        confirmed: bool,
    ) -> Result<(), AppError> {
        if !confirmed || !crate::contracts::valid_id(id) {
            return Err(AppError::new("ACCESS_DENIED"));
        }
        let c = file_capability(&capability)?;
        let m = crate::modules::builtins()
            .into_iter()
            .find(|m| m.id == module && m.permissions.contains(&capability))
            .ok_or_else(|| AppError::new("ACCESS_DENIED"))?;
        let canonical = scoped_existing_path(scope, scope)?;
        if !canonical.is_dir() {
            return Err(AppError::new("ACCESS_DENIED"));
        }
        let scope = canonical
            .to_str()
            .ok_or_else(|| AppError::new("INVALID_DATA"))?;
        self.connection.execute("INSERT INTO permission_grants(id,subject_type,subject_id,capability,resource_scope,grant_type) VALUES(?1,'module',?2,?3,?4,'directory')",params![id,m.id,c,scope]).map_err(storage_error)?;
        Ok(())
    }

    pub fn revoke_grant(&self, id: &str) -> Result<(), AppError> {
        self.connection
            .execute(
                "UPDATE permission_grants SET revoked_at=unixepoch() WHERE id=?1",
                [id],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    // Re-read live grants on every call; revocation is not cached.
    pub fn authorize_file(
        &self,
        caller: HostCaller,
        module: &str,
        capability: Capability,
        path: &Path,
    ) -> Result<PathBuf, AppError> {
        caller.require_local()?;
        let c = file_capability(&capability)?;
        let mut stmt = self.connection.prepare("SELECT resource_scope FROM permission_grants WHERE subject_id=?1 AND capability=?2 AND revoked_at IS NULL").map_err(storage_error)?;
        let rows = stmt
            .query_map(params![module, c], |r| r.get::<_, String>(0))
            .map_err(storage_error)?;
        for scope in rows {
            if let Ok(validated) =
                scoped_existing_path(Path::new(&scope.map_err(storage_error)?), path)
            {
                return Ok(validated);
            }
        }
        Err(AppError::new("ACCESS_DENIED"))
    }
}

fn file_capability(c: &Capability) -> Result<&'static str, AppError> {
    match c {
        Capability::FileRead => Ok("file_read"),
        Capability::FileWrite => Ok("file_write"),
        _ => Err(AppError::new("ACCESS_DENIED")),
    }
}
