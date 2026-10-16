//! Standalone validation tool, never linked into the production desktop entry.
use serde_json::json;
use std::{fs, path::PathBuf};
use toolbox_core::{
    contracts::Request, database::Database, paths::DataPaths, permissions::HostCaller,
    service::CoreService,
};

fn main() {
    if let Err(code) = run() {
        eprintln!("WF03_PROBE_FAILED:{code}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let mode = args.next().ok_or("MISSING_MODE")?;
    let root = PathBuf::from(args.next().ok_or("MISSING_ROOT")?);
    let output = PathBuf::from(args.next().ok_or("MISSING_OUTPUT")?);
    if args.next().is_some() {
        return Err("EXTRA_ARGUMENT".into());
    }
    if mode == "--expect-unwritable" {
        let error = match DataPaths::at_root(&root) {
            Ok(_) => return Err("UNEXPECTED_WRITE_ACCESS".into()),
            Err(e) => e,
        };
        if error.code != "DATA_DIRECTORY_NOT_WRITABLE" {
            return Err(error.code);
        }
        fs::write(
            output,
            json!({"result":"PASS","negative_path":error.code}).to_string(),
        )
        .map_err(|_| "REPORT_WRITE_FAILED")?;
        return Ok(());
    }
    if mode != "--verify" {
        return Err("INVALID_MODE".into());
    }
    let paths = DataPaths::at_root(&root).map_err(|e| e.code)?;
    let db = Database::open(&paths.database).map_err(|e| e.code)?;
    if !db.foreign_keys_enabled().map_err(|e| e.code)? {
        return Err("FOREIGN_KEYS_DISABLED".into());
    }
    db.set_setting("wf03-probe", &json!({"value":"中文 ' & quote"}))
        .map_err(|e| e.code)?;
    drop(db);
    let db = Database::open(&paths.database).map_err(|e| e.code)?;
    if db.setting("wf03-probe").map_err(|e| e.code)? != Some(json!({"value":"中文 ' & quote"})) {
        return Err("REOPEN_MISMATCH".into());
    }
    drop(db);
    let service = CoreService::open(&paths).map_err(|e| e.code)?;
    let request:Request=serde_json::from_value(json!({"request_id":"probe-1","api_version":1,"caller_context":{"module_id":"toolbox.shell"},"params":{"command":"modules.list","args":{}}})).map_err(|_|"REQUEST_INVALID")?;
    let local = serde_json::to_value(service.dispatch(HostCaller::LocalShell, request.clone()))
        .map_err(|_| "ENCODE_FAILED")?;
    let remote = serde_json::to_value(service.dispatch(HostCaller::Remote, request))
        .map_err(|_| "ENCODE_FAILED")?;
    if local["success"] != true
        || local["data"].as_array().map(Vec::len) != Some(7)
        || remote["error"]["code"] != "ACCESS_DENIED"
    {
        return Err("DISPATCH_FAILED".into());
    }
    fs::write(output,json!({"result":"PASS","database_reopen":true,"foreign_keys":true,"module_count":7,"remote_denied":true,"sqlite_version":rusqlite::version()}).to_string()).map_err(|_|"REPORT_WRITE_FAILED")?;
    Ok(())
}
