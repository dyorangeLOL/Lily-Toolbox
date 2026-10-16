use rusqlite::Connection;
use serde_json::{json, Value};
use std::{
    fs,
    sync::{Arc, Barrier},
    thread,
};
use toolbox_core::{
    contracts::{Event, Request, Response},
    database::{migrate, Database, Migration, MIGRATIONS},
    modules::{Capability, Manifest},
    paths::{scoped_existing_path, DataPaths},
    permissions::HostCaller,
    service::CoreService,
};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../packages/shared-types/fixtures/core.json"
    ))
    .unwrap()
}

#[test]
fn shared_contracts_roundtrip_and_reject_invalid_vectors() {
    let f = fixture();
    for value in f["requests"].as_array().unwrap() {
        let r: Request = serde_json::from_value(value.clone()).unwrap();
        r.validate().unwrap();
        assert_eq!(serde_json::to_value(r).unwrap(), *value);
    }
    for value in f["invalid_requests"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<Request>(value.clone())
                .and_then(|r| r.validate().map(|_| r).map_err(serde::de::Error::custom))
                .is_err(),
            "{value}"
        );
    }
    for value in f["responses"].as_array().unwrap() {
        let r: Response = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(r).unwrap(), *value);
    }
    for value in f["invalid_responses"].as_array().unwrap() {
        assert!(serde_json::from_value::<Response>(value.clone()).is_err());
    }
    for value in f["events"].as_array().unwrap() {
        let e: Event = serde_json::from_value(value.clone()).unwrap();
        e.validate().unwrap();
        assert_eq!(serde_json::to_value(e).unwrap(), *value);
    }
    for value in f["manifests"].as_array().unwrap() {
        let m: Manifest = serde_json::from_value(value.clone()).unwrap();
        m.validate().unwrap();
        assert_eq!(serde_json::to_value(m).unwrap(), *value);
    }
}

#[test]
fn fresh_database_reopens_preserves_json_and_enforces_sql_constraints() {
    let dir = tempfile::tempdir().unwrap();
    let paths = DataPaths::at_root(dir.path()).unwrap();
    let db = Database::open(&paths.database).unwrap();
    db.set_setting(
        "layout",
        &json!({"value":"引号 ' ; DROP TABLE settings; --"}),
    )
    .unwrap();
    assert!(db.foreign_keys_enabled().unwrap());
    assert_eq!(db.modules().unwrap().len(), 7);
    drop(db);
    let reopened = Database::open(&paths.database).unwrap();
    assert_eq!(
        reopened.setting("layout").unwrap().unwrap()["value"],
        "引号 ' ; DROP TABLE settings; --"
    );
    let c = Connection::open(&paths.database).unwrap();
    assert_eq!(
        c.query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "wal"
    );
    assert!(c
        .execute(
            "INSERT INTO settings VALUES('bad','not-json','global',0)",
            []
        )
        .is_err());
    assert!(c
        .execute(
            "INSERT INTO modules VALUES('third','1','optional',1,NULL,1,'{}')",
            []
        )
        .is_err());
    // Foreign keys are per connection; the production connection must enforce them.
    assert!(reopened
        .grant_file_scope(
            "orphan",
            "unregistered",
            Capability::FileRead,
            dir.path(),
            true
        )
        .is_err());
}

#[test]
fn upgrade_is_atomic_and_rejects_checksum_future_version_and_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.sqlite");
    let mut c = Connection::open(&path).unwrap();
    migrate(&mut c, &MIGRATIONS[..1]).unwrap();
    c.execute("INSERT INTO settings VALUES('keep','42','global',0)", [])
        .unwrap();
    drop(c);
    let db = Database::open(&path).unwrap();
    assert_eq!(db.setting("keep").unwrap(), Some(json!(42)));
    drop(db);
    let mut c = Connection::open(&path).unwrap();
    let broken = [
        Migration {
            version: 1,
            sql: MIGRATIONS[0].sql,
        },
        Migration {
            version: 2,
            sql: MIGRATIONS[1].sql,
        },
        Migration {
            version: 3,
            sql: "CREATE TABLE must_rollback(id INTEGER); INVALID SQL;",
        },
    ];
    assert_eq!(
        migrate(&mut c, &broken).unwrap_err().code,
        "MIGRATION_FAILED"
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='must_rollback'",
            [],
            |r| r.get::<_, i32>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM schema_migrations", [], |r| r
            .get::<_, i32>(0))
            .unwrap(),
        2
    );
    c.execute(
        "UPDATE schema_migrations SET checksum='tampered' WHERE version=1",
        [],
    )
    .unwrap();
    assert_eq!(
        migrate(&mut c, MIGRATIONS).unwrap_err().code,
        "SCHEMA_MISMATCH"
    );
    c.execute("DELETE FROM schema_migrations", []).unwrap();
    c.execute("INSERT INTO schema_migrations VALUES(99,0,'future')", [])
        .unwrap();
    assert_eq!(
        migrate(&mut c, MIGRATIONS).unwrap_err().code,
        "SCHEMA_MISMATCH"
    );
    let bad = dir.path().join("bad.sqlite");
    fs::write(&bad, b"not a sqlite database").unwrap();
    assert!(Database::open(&bad).is_err());
}

#[test]
fn concurrent_writers_use_transactions_without_lost_rows() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shared.sqlite");
    drop(Database::open(&path).unwrap());
    let barrier = Arc::new(Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|n| {
            let path = path.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let db = Database::open(&path).unwrap();
                barrier.wait();
                for i in 0..20 {
                    db.set_setting(&format!("worker-{n}-{i}"), &json!(i))
                        .unwrap();
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    let c = Connection::open(path).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM settings", [], |r| r.get::<_, i32>(0))
            .unwrap(),
        80
    );
}

#[test]
fn grants_are_scoped_default_denied_and_immediately_revocable() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("allowed 中文 space");
    fs::create_dir(&root).unwrap();
    let file = root.join("quote ' & file.txt");
    fs::write(&file, b"original").unwrap();
    let sibling = dir.path().join("allowed 中文 space-other");
    fs::create_dir(&sibling).unwrap();
    let outside = sibling.join("private");
    fs::write(&outside, b"private").unwrap();
    let db = Database::open(&dir.path().join("db.sqlite")).unwrap();
    assert!(db
        .authorize_file(
            HostCaller::LocalShell,
            "toolbox.files",
            Capability::FileRead,
            &file
        )
        .is_err());
    assert!(db
        .grant_file_scope("g", "toolbox.files", Capability::FileRead, &root, false)
        .is_err());
    db.grant_file_scope("g", "toolbox.files", Capability::FileRead, &root, true)
        .unwrap();
    assert!(db
        .authorize_file(
            HostCaller::LocalShell,
            "toolbox.files",
            Capability::FileRead,
            &file
        )
        .is_ok());
    assert!(db
        .authorize_file(
            HostCaller::Remote,
            "toolbox.files",
            Capability::FileRead,
            &file
        )
        .is_err());
    assert!(db
        .authorize_file(
            HostCaller::LocalShell,
            "toolbox.files",
            Capability::FileWrite,
            &file
        )
        .is_err());
    assert!(db
        .authorize_file(
            HostCaller::LocalShell,
            "toolbox.files",
            Capability::FileRead,
            &outside
        )
        .is_err());
    assert!(
        scoped_existing_path(&root, &root.join("../allowed 中文 space-other/private")).is_err()
    );
    assert!(scoped_existing_path(&root, &root.join("quote ' & file.txt:stream")).is_err());
    db.revoke_grant("g").unwrap();
    assert!(db
        .authorize_file(
            HostCaller::LocalShell,
            "toolbox.files",
            Capability::FileRead,
            &file
        )
        .is_err());
    assert_eq!(fs::read(&file).unwrap(), b"original");
}

#[test]
fn portable_layout_and_unwritable_target_never_fall_back() {
    let dir = tempfile::tempdir().unwrap();
    let portable = dir.path().join("便携 space");
    let app = portable.join("app");
    fs::create_dir_all(&app).unwrap();
    fs::write(portable.join("portable.flag"), b"").unwrap();
    let installed = dir.path().join("app-data");
    let paths = DataPaths::resolve(&app.join("Toolbox.exe"), &installed).unwrap();
    assert!(paths
        .database
        .starts_with(fs::canonicalize(portable.join("data")).unwrap()));
    assert!(!installed.exists());
    let bad = dir.path().join("a-file");
    fs::write(&bad, b"original").unwrap();
    assert!(DataPaths::at_root(&bad.join("data")).is_err());
    assert_eq!(fs::read(bad).unwrap(), b"original");
    let normal = DataPaths::resolve(&dir.path().join("installed/Toolbox.exe"), &installed).unwrap();
    assert_eq!(normal.root, fs::canonicalize(&installed).unwrap());
}

#[test]
fn dispatch_rejects_spoofed_remote_and_reserved_commands_with_safe_audit() {
    let dir = tempfile::tempdir().unwrap();
    let paths = DataPaths::at_root(dir.path()).unwrap();
    let service = CoreService::open(&paths).unwrap();
    let mut req: Request = serde_json::from_value(fixture()["requests"][0].clone()).unwrap();
    let remote = serde_json::to_value(service.dispatch(HostCaller::Remote, req.clone())).unwrap();
    assert_eq!(remote["error"]["code"], "ACCESS_DENIED");
    let local =
        serde_json::to_value(service.dispatch(HostCaller::LocalShell, req.clone())).unwrap();
    assert_eq!(local["data"].as_array().unwrap().len(), 7);
    req.params = toolbox_core::contracts::DomainCommand::AppsList {};
    let reserved =
        serde_json::to_value(service.dispatch(HostCaller::LocalShell, req.clone())).unwrap();
    assert_eq!(reserved["error"]["code"], "NOT_IMPLEMENTED");
    req.params = toolbox_core::contracts::DomainCommand::FilesList {
        path: "C:\\private\\secret?token=must-not-log".into(),
    };
    let denied = serde_json::to_value(service.dispatch(HostCaller::LocalShell, req)).unwrap();
    assert_eq!(denied["error"]["code"], "ACCESS_DENIED");
    let c = Connection::open(paths.database).unwrap();
    let audit: String = c
        .query_row(
            "SELECT group_concat(safe_details_json) FROM audit_events",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!audit.contains("private"));
    assert!(!audit.contains("token"));
    assert!(!audit.contains("must-not-log"));
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM audit_events WHERE outcome='ACCESS_DENIED'",
            [],
            |r| r.get::<_, i32>(0)
        )
        .unwrap(),
        2
    );
}

#[cfg(unix)]
#[test]
fn symlink_escape_and_loop_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    fs::create_dir(&root).unwrap();
    let outside = dir.path().join("secret");
    fs::write(&outside, b"x").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
    assert!(scoped_existing_path(&root, &root.join("link")).is_err());
    std::os::unix::fs::symlink(&root, root.join("loop")).unwrap();
    assert!(scoped_existing_path(&root, &root.join("loop/loop")).is_err());
}

#[cfg(windows)]
#[test]
fn windows_junction_escape_is_rejected() {
    use std::os::windows::process::CommandExt;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let outside = dir.path().join("outside");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("secret"), b"x").unwrap();
    let junction = root.join("junction");
    // Paths are generated inside this test-owned temporary directory, no user text.
    let status = std::process::Command::new("cmd.exe")
        .args(["/D", "/C", "mklink", "/J"])
        .arg(&junction)
        .arg(&outside)
        .creation_flags(0x08000000)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(scoped_existing_path(&root, &junction.join("secret")).is_err());
    fs::remove_dir(junction).unwrap();
}
