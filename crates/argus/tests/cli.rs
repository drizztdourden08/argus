//! The server lifecycle verbs end to end, ported from `core.cli.test.mjs`: the real `argus`
//! binary in child processes on a throwaway home and an ephemeral port (`DASHBOARD_PORT=0`, so
//! the owner's 47615 is never touched). `start` (detached, hidden), `status`, `open --print`,
//! `resync`, token rotation, a second `start`, `stop`. Whatever a test started is stopped by the
//! home's drop, through `argus stop`.

use std::path::{Path, PathBuf};
use std::process::Command;

use argus_server::instance::call;
use serde_json::Value;
use tempfile::TempDir;

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

struct Home {
    dir: TempDir,
}

impl Home {
    fn new() -> Home {
        let dir = tempfile::tempdir().expect("a temp home");
        std::fs::create_dir_all(dir.path().join("alpha")).expect("a board folder");
        std::fs::write(dir.path().join("alpha").join("board.json"), "{}").expect("a board");
        Home { dir }
    }

    fn dashboard(&self) -> PathBuf {
        self.dir.path().join("dashboard")
    }

    fn run(&self, args: &[&str]) -> Out {
        let out = Command::new(env!("CARGO_BIN_EXE_argus"))
            .args(args)
            .env("COORD_HOME", self.dir.path())
            .env("DASHBOARD_HOME", self.dashboard())
            .env("DASHBOARD_PORT", "0")
            .output()
            .expect("argus runs");
        Out {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }

    fn json(&self, file: &str) -> Value {
        let path: &Path = &self.dashboard().join(file);
        serde_json::from_str(&std::fs::read_to_string(path).expect("the file")).expect("JSON")
    }

    fn pid_file(&self) -> PathBuf {
        self.dashboard().join("server.pid")
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _stopped = self.run(&["stop"]);
    }
}

fn running_line(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("dashboard running at http://127.0.0.1:") else {
        return false;
    };
    rest.split('/')
        .next()
        .is_some_and(|p| p.parse::<u16>().is_ok())
}

#[test]
fn status_and_stop_with_no_server() {
    let home = Home::new();
    let status = home.run(&["status"]);
    assert_eq!(status.code, 1);
    assert!(status.stdout.contains("not running"), "{}", status.stdout);
    let json: Value = serde_json::from_str(&home.run(&["status", "--json"]).stdout).expect("JSON");
    assert_eq!(json, serde_json::json!({ "running": false }));
    let stop = home.run(&["stop"]);
    assert_eq!(stop.code, 0);
    assert!(stop.stdout.contains("not running"), "{}", stop.stdout);
}

#[test]
fn usage_and_unknown_commands() {
    let home = Home::new();
    assert_eq!(home.run(&[]).code, 2);
    assert_eq!(home.run(&["--help"]).code, 0);
    let bad = home.run(&["frobnicate"]);
    assert_eq!(bad.code, 2);
    assert!(bad.stderr.contains("unknown command"), "{}", bad.stderr);
    assert_eq!(home.run(&["token"]).code, 2);
    assert_eq!(home.run(&["open", "Bad_Key"]).code, 2);
    assert_eq!(home.run(&["logs", "--lines", "0"]).code, 2);
    assert_eq!(home.run(&["logs", "--run", "R-1"]).code, 2);
}

/// A first start, a second one that finds it, and what `status` says of it; the port.
fn started(home: &Home) -> u16 {
    let start = home.run(&["start"]);
    assert_eq!(start.code, 0, "{}", start.stderr);
    assert!(running_line(start.stdout.trim()), "{}", start.stdout);
    let record = home.json("server.pid");
    let (pid, port) = (
        record["pid"].as_u64().unwrap_or(0),
        record["port"].as_u64().unwrap_or(0),
    );
    assert!(pid > 0 && port > 0 && port != 47615);

    let again = home.run(&["start"]);
    assert_eq!(again.code, 0);
    assert!(again.stdout.contains("already running"), "{}", again.stdout);
    assert_eq!(
        home.json("server.pid")["pid"].as_u64(),
        Some(pid),
        "no second server"
    );

    let status: Value =
        serde_json::from_str(&home.run(&["status", "--json"]).stdout).expect("JSON");
    assert_eq!(status["running"], true);
    assert_eq!(
        (status["port"].as_u64(), status["pid"].as_u64()),
        (Some(port), Some(pid))
    );
    assert!(
        home.run(&["status"])
            .stdout
            .contains("dashboard running at")
    );
    u16::try_from(port).expect("a port")
}

/// `open --print`, the page and the old sign-in link, then `resync` and the token rotation.
async fn pages_and_token(home: &Home, port16: u16) {
    let port = port16;
    let page = home.run(&["open", "alpha", "--print"]);
    assert_eq!(page.code, 0, "{}", page.stderr);
    assert_eq!(
        page.stdout.trim(),
        format!("http://127.0.0.1:{port}/p/alpha/")
    );
    let served = call(port16, "GET", "/p/alpha/", None)
        .await
        .expect("the page answers");
    assert_eq!(served.0, 200, "the page needs no sign-in");
    let root = home.run(&["open", "--print"]);
    assert_eq!(root.stdout.trim(), format!("http://127.0.0.1:{port}/"));
    let old = call(port16, "GET", "/auth?code=stale&next=%2Fp%2Falpha%2F", None).await;
    assert_eq!(
        old.map(|r| r.0),
        Some(302),
        "an old sign-in link is harmless"
    );

    let resync = home.run(&["resync", "--project", "alpha"]);
    assert_eq!(resync.code, 0, "{}", resync.stderr);
    let before = home.json("config.json")["token"].clone();
    let rotated = home.run(&["token", "--rotate"]);
    assert_eq!(rotated.code, 0, "{}", rotated.stderr);
    let after = home.json("config.json")["token"].clone();
    assert_ne!(before, after);
    let old_bearer = call(port16, "GET", "/api/health", before.as_str()).await;
    assert_eq!(old_bearer.map(|r| r.0), Some(401));
    assert_eq!(home.run(&["status"]).code, 0, "the CLI needs no token");
}

#[tokio::test(flavor = "multi_thread")]
async fn start_status_open_resync_token_second_start_stop() {
    let home = Home::new();
    let port16 = started(&home);
    pages_and_token(&home, port16).await;
    let log = home.run(&["logs", "--lines", "5"]);
    assert_eq!(log.code, 0);
    let wanted = ["listening", "stopping", "token rotated"];
    assert!(
        wanted.iter().any(|w| log.stdout.contains(w)),
        "{}",
        log.stdout
    );

    let stop = home.run(&["stop"]);
    assert_eq!(stop.code, 0, "{}", stop.stderr);
    assert!(stop.stdout.contains("stopped"), "{}", stop.stdout);
    assert!(!home.pid_file().exists());
    assert_eq!(home.run(&["status"]).code, 1);
    assert!(call(port16, "GET", "/api/health", None).await.is_none());
}
