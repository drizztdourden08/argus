//! Port of `core.config.test.mjs`: paths, config defaults and token, env port, refused host, log redaction and rotation.
//! K2, K3, K4: the config file, the refused host, the redacted log.

// test-tag: critical

use std::fs;
use std::io;

use argus_core::Paths;
use argus_core::config::{
    DEFAULT_PORT, install_dir, load_config, parse_env_port, peek_config, save_config,
};
use argus_core::log::{LOG_KEEP, Logger, err_field};
use argus_core::redact::Redactor;
use serde_json::{Value, json};
use tempfile::TempDir;

fn fresh() -> (TempDir, Paths) {
    let dir = TempDir::new().unwrap();
    let paths = Paths::new(dir.path(), Some(dir.path().join("home")));
    (dir, paths)
}

fn is_token(t: &str) -> bool {
    t.len() == 64 && t.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn read_json(paths: &Paths) -> Value {
    serde_json::from_str(&fs::read_to_string(&paths.config).unwrap()).unwrap()
}

fn edit_file(paths: &Paths, edit: impl FnOnce(&mut Value)) {
    let mut doc = read_json(paths);
    edit(&mut doc);
    fs::write(&paths.config, doc.to_string()).unwrap();
}

fn log_lines(file: &std::path::Path) -> Vec<Value> {
    fs::read_to_string(file)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn paths_follow_coord_home_and_dashboard_home() {
    let dir = TempDir::new().unwrap();
    let coord = dir.path();
    let p = Paths::new(coord, None);
    assert_eq!(p.coord_home, coord);
    assert_eq!(p.home, coord.join("dashboard"));
    assert_eq!(p.db, coord.join("dashboard").join("dashboard.db"));
    assert_eq!(p.pid_file, coord.join("dashboard").join("server.pid"));

    let home = dir.path().join("elsewhere");
    let q = Paths::new(coord, Some(home.clone()));
    assert_eq!(q.home, home);
    assert_eq!(q.config, home.join("config.json"));
    assert_eq!(q.logs, home.join("logs"));
    assert_eq!(q.attachments, home.join("attachments"));
    assert_eq!(q.board_dir("argus"), coord.join("argus"));
}

#[test]
fn load_config_creates_config_json_with_defaults_and_a_64_hex_token() {
    let (_dir, paths) = fresh();
    assert!(peek_config(&paths, None).is_none());
    let c = load_config(&paths, None).unwrap();
    assert_eq!(c.version, 1);
    assert_eq!(c.host, "127.0.0.1");
    assert_eq!(c.port, DEFAULT_PORT);
    assert_eq!(DEFAULT_PORT, 47615);
    assert!(is_token(&c.token));
    let runs = serde_json::to_value(&c.runs).unwrap();
    let want_runs = json!({
        "maxConcurrent": 4, "logCapBytes": 2_097_152, "logHeadBytes": 262_144,
        "keepPerDef": 30, "keepDays": 14, "stopGraceMs": 5000
    });
    assert_eq!(runs, want_runs);
    let mcp = serde_json::to_value(&c.mcp).unwrap();
    assert_eq!(mcp, json!({ "allow": "*", "deny": [], "readOnly": false }));
    let attach = serde_json::to_value(&c.attach).unwrap();
    assert_eq!(attach, json!({ "maxBytes": 10_485_760, "extraRoots": [] }));
    assert!(paths.config.exists());
    assert_eq!(load_config(&paths, None).unwrap().token, c.token);
    let left: Vec<_> = fs::read_dir(&paths.home)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        left,
        ["config.json"],
        "written through the atomic writer, no temp file"
    );
}

#[test]
fn unknown_keys_are_kept_partial_sub_objects_get_defaults_and_a_given_install_dir_is_kept() {
    let (_dir, paths) = fresh();
    let c = load_config(&paths, None).unwrap();
    edit_file(&paths, |doc| {
        doc["custom"] = json!({ "keep": true });
        doc["runs"] = json!({ "maxConcurrent": 9 });
        doc["installDir"] = json!("Z:\\somewhere\\else");
    });
    let again = load_config(&paths, None).unwrap();
    assert_eq!(again.extra["custom"], json!({ "keep": true }));
    assert_eq!(again.runs.max_concurrent, 9);
    assert_eq!(again.runs.log_cap_bytes, 2_097_152);
    assert_eq!(again.token, c.token);
    assert_eq!(
        c.install_dir,
        install_dir(),
        "a new file names the binary's folder"
    );
    assert_eq!(
        again.install_dir, "Z:\\somewhere\\else",
        "a given installDir is kept"
    );
    assert_eq!(
        read_json(&paths)["installDir"],
        json!("Z:\\somewhere\\else")
    );
    assert!(again.exe.is_some(), "exe names the running binary");
}

#[test]
fn the_env_port_overrides_the_port_without_being_saved() {
    let (_dir, paths) = fresh();
    let c = load_config(&paths, None).unwrap();
    let mut e = load_config(&paths, Some(0)).unwrap();
    assert_eq!(e.port, 0);
    e.project_roots.insert("x".into(), "y".into());
    save_config(&paths, &e, Some(0)).unwrap();
    let back = load_config(&paths, None).unwrap();
    assert_eq!(back.port, c.port, "the file kept its own port");
    assert_eq!(back.project_roots.get("x").map(String::as_str), Some("y"));
}

#[test]
fn parse_env_port_reads_a_port_and_refuses_the_rest() {
    assert_eq!(parse_env_port(None).unwrap(), None);
    assert_eq!(parse_env_port(Some("")).unwrap(), None);
    assert_eq!(parse_env_port(Some("0")).unwrap(), Some(0));
    assert_eq!(parse_env_port(Some("47615")).unwrap(), Some(47615));
    for bad in ["70000", "-1", "abc", "1.5"] {
        let err = parse_env_port(Some(bad)).unwrap_err();
        assert!(err.to_string().contains("DASHBOARD_PORT"), "{bad}");
    }
}

#[test]
fn a_host_other_than_loopback_is_refused() {
    let (_dir, paths) = fresh();
    load_config(&paths, None).unwrap();
    edit_file(&paths, |doc| doc["host"] = json!("0.0.0.0"));
    let err = load_config(&paths, None).unwrap_err();
    assert!(err.to_string().contains("127.0.0.1"));
}

#[test]
fn a_malformed_token_is_replaced() {
    let (_dir, paths) = fresh();
    load_config(&paths, None).unwrap();
    edit_file(&paths, |doc| doc["token"] = json!("short"));
    assert!(is_token(&load_config(&paths, None).unwrap().token));
}

#[test]
fn the_log_writes_json_lines_and_redacts_the_token_and_secrets() {
    let (_dir, paths) = fresh();
    let file = paths.logs.join("server.log");
    let token = "a".repeat(64);
    let log = Logger::new(
        &file,
        Redactor::new(std::slice::from_ref(&token)),
        5 * 1024 * 1024,
    );
    log.info("hello", json!({ "n": 1 }));
    log.warn(
        &format!("seen {token} and token=abc123 and Authorization: Bearer xyz"),
        json!({ "nested": { "v": format!("password: hunter2 {token}") } }),
    );
    log.error(
        "boom",
        json!({ "err": err_field(&io::Error::other("bad")) }),
    );
    let lines = log_lines(&file);
    let levels: Vec<&str> = lines.iter().map(|l| l["level"].as_str().unwrap()).collect();
    assert_eq!(levels, ["info", "warn", "error"]);
    assert_eq!(lines[0]["n"], json!(1));
    let text = serde_json::to_string(&lines).unwrap();
    for secret in [token.as_str(), "hunter2", "abc123", "xyz"] {
        assert!(!text.contains(secret), "{secret} leaked");
    }
    assert!(lines[1]["msg"].as_str().unwrap().contains("[redacted]"));
    assert_eq!(lines[2]["err"]["message"], json!("bad"));
}

#[test]
fn the_log_rotates_at_the_size_limit_and_keeps_five_files() {
    let (_dir, paths) = fresh();
    let file = paths.logs.join("server.log");
    let log = Logger::new(&file, Redactor::default(), 2000);
    for i in 0..200 {
        log.info(&format!("line {i} {}", "x".repeat(60)), json!({}));
    }
    let mut names: Vec<String> = fs::read_dir(&paths.logs)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(LOG_KEEP, 5);
    assert_eq!(
        names,
        [
            "server.log",
            "server.log.1",
            "server.log.2",
            "server.log.3",
            "server.log.4",
            "server.log.5"
        ]
    );
    assert!(fs::read_to_string(&file).unwrap().contains("line 199"));
    let newest = format!("{}.1", file.display());
    assert!(fs::read_to_string(newest).unwrap().len() <= 2100);
}
