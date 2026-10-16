use crate::{contracts::valid_id, AppError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    ModulesRead,
    FileRead,
    FileWrite,
    AppLaunch,
    BrowserManage,
    ConvertRun,
    BackupRestore,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    Builtin,
    Optional,
    Sidecar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Registered,
    Available,
    Activated,
    Deactivated,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub version: String,
    pub api_version: u32,
    pub kind: ModuleKind,
    pub features: Vec<String>,
    pub permissions: Vec<Capability>,
    pub dependencies: Vec<Dependency>,
    pub entry: String,
    pub lifecycle: Lifecycle,
}

impl Manifest {
    pub fn validate(&self) -> Result<(), AppError> {
        let ok = self.id.starts_with("toolbox.")
            && valid_id(&self.id)
            && semver::Version::parse(&self.version).is_ok()
            && self.api_version == 1
            && self.kind == ModuleKind::Builtin
            && self.entry == format!("builtin:{}", self.id)
            && self.features.len() <= 32
            && self.permissions.len() <= 16
            && self.dependencies.is_empty()
            && self.features.iter().all(|f| valid_id(f))
            && self
                .dependencies
                .iter()
                .all(|d| valid_id(&d.id) && semver::VersionReq::parse(&d.version).is_ok());
        if ok {
            Ok(())
        } else {
            Err(AppError::new("INVALID_DATA"))
        }
    }
}

// This is a native registry, not a UI navigation list or a third-party loader.
pub fn builtins() -> Vec<Manifest> {
    [
        "apps", "browser", "files", "search", "convert", "backup", "modules",
    ]
    .into_iter()
    .map(|name| {
        let id = format!("toolbox.{name}");
        let implemented = name == "modules";
        Manifest {
            entry: format!("builtin:{id}"),
            id,
            version: env!("CARGO_PKG_VERSION").into(),
            api_version: 1,
            kind: ModuleKind::Builtin,
            features: if implemented {
                vec!["modules.list".into(), "modules.capabilities".into()]
            } else {
                vec![]
            },
            permissions: match name {
                "modules" => vec![Capability::ModulesRead],
                "files" => vec![Capability::FileRead, Capability::FileWrite],
                "backup" => vec![Capability::FileRead, Capability::BackupRestore],
                _ => vec![],
            },
            dependencies: vec![],
            lifecycle: if implemented {
                Lifecycle::Available
            } else {
                Lifecycle::Registered
            },
        }
    })
    .collect()
}
