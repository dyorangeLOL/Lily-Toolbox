use crate::AppError;
use std::{
    fs::{self, OpenOptions},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct DataPaths {
    pub root: PathBuf,
    pub database: PathBuf,
    pub webview: PathBuf,
}

impl DataPaths {
    // executable and app_data are supplied by the host, never by an IPC request.
    // Portable opt-in: <root>/portable.flag and executable under <root>/app/.
    pub fn resolve(executable: &Path, app_data: &Path) -> Result<Self, AppError> {
        let app = executable
            .parent()
            .ok_or_else(|| AppError::new("INVALID_DATA"))?;
        let portable = app.parent().filter(|root| {
            app.file_name().is_some_and(|n| n == "app") && root.join("portable.flag").is_file()
        });
        Self::at_root(&portable.map_or_else(|| app_data.to_path_buf(), |p| p.join("data")))
    }

    pub fn at_root(root: &Path) -> Result<Self, AppError> {
        if !root.is_absolute()
            || root
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        {
            return Err(AppError::new("INVALID_DATA"));
        }
        reject_links(root)?;
        fs::create_dir_all(root).map_err(|_| AppError::new("DATA_DIRECTORY_NOT_WRITABLE"))?;
        reject_links(root)?;
        let root =
            fs::canonicalize(root).map_err(|_| AppError::new("DATA_DIRECTORY_NOT_WRITABLE"))?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let probe = root.join(format!(".write-probe-{}-{nonce}", std::process::id()));
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&probe)
            .map_err(|_| AppError::new("DATA_DIRECTORY_NOT_WRITABLE"))?;
        drop(file);
        fs::remove_file(probe).map_err(|_| AppError::new("DATA_DIRECTORY_NOT_WRITABLE"))?;
        for name in ["db", "webview", "logs"] {
            let p = root.join(name);
            reject_links(&p)?;
            fs::create_dir_all(&p).map_err(|_| AppError::new("DATA_DIRECTORY_NOT_WRITABLE"))?;
        }
        let database = root.join("db/toolbox.sqlite");
        reject_links(&database)?;
        Ok(Self {
            database,
            webview: root.join("webview"),
            root,
        })
    }
}

pub fn reject_links(path: &Path) -> Result<(), AppError> {
    let mut current = PathBuf::new();
    for part in path.components() {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(AppError::new("ACCESS_DENIED"));
                    }
                }
                if meta.file_type().is_symlink() {
                    return Err(AppError::new("ACCESS_DENIED"));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(AppError::new("ACCESS_DENIED")),
        }
    }
    Ok(())
}

pub fn scoped_existing_path(scope: &Path, requested: &Path) -> Result<PathBuf, AppError> {
    if !requested.is_absolute()
        || requested.components().any(|c| match c {
            Component::ParentDir | Component::CurDir => true,
            Component::Normal(s) => {
                s.to_string_lossy().contains(':') || s.to_string_lossy().contains('\0')
            }
            #[cfg(windows)]
            Component::Prefix(p) => !matches!(
                p.kind(),
                std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)
            ),
            _ => false,
        })
    {
        return Err(AppError::new("ACCESS_DENIED"));
    }
    reject_links(scope)?;
    reject_links(requested)?;
    let root = fs::canonicalize(scope).map_err(|_| AppError::new("ACCESS_DENIED"))?;
    let target = fs::canonicalize(requested).map_err(|_| AppError::new("ACCESS_DENIED"))?;
    if target.starts_with(root) {
        Ok(target)
    } else {
        Err(AppError::new("ACCESS_DENIED"))
    }
}
