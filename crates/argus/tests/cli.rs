//! The server lifecycle verbs end to end, ported from `core.cli.test.mjs`: the real `argus`
//! binary in child processes on a throwaway home and an ephemeral port (`DASHBOARD_PORT=0`, so
//! the owner's 47615 is never touched). `start` (detached, hidden), `status`, `open --print`,
//! `resync`, token rotation, a second `start`, `stop`. Whatever a test started is stopped by the
//! home's drop, through `argus stop`.

use std::path::{Path, PathBuf};
use std::process::Command;

use argus_server::instance::{PROBE_ENV, call};
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
            .env(PROBE_ENV, "60000")
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

const NODE_TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// A stand-in for the Node dashboard on an ephemeral port: its health body has no `impl` field.
/// Every request line it gets is kept.
struct FakeNode {
    port: u16,
    seen: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl FakeNode {
    /// Listening, and proven to answer, before the first call of a test: the listener is bound
    /// here, and one probe of its own goes through before this returns.
    fn start() -> FakeNode {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a port");
        let port = listener.local_addr().expect("an address").port();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = std::sync::Arc::clone(&seen);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let log = std::sync::Arc::clone(&log);
                std::thread::spawn(move || Self::answer(stream, &log));
            }
        });
        let node = FakeNode { port, seen };
        node.ready();
        node
    }

    /// One health call of the test itself, then its line is forgotten.
    fn ready(&self) {
        use std::io::{Read, Write};

        let mut probe = std::net::TcpStream::connect(("127.0.0.1", self.port)).expect("connects");
        let ask = "GET /api/health HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n";
        probe.write_all(ask.as_bytes()).expect("asks");
        let mut reply = String::new();
        probe.read_to_string(&mut reply).expect("answers");
        assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
        self.seen.lock().expect("the log").clear();
    }

    /// Reads the whole request, so the close never resets the client's read, keeps its request
    /// line and answers the Node health.
    fn answer(mut stream: std::net::TcpStream, log: &std::sync::Mutex<Vec<String>>) {
        use std::io::{BufRead, BufReader, Read, Write};

        let mut reader = BufReader::new(stream.try_clone().expect("a clone"));
        let mut first = String::new();
        let _read = reader.read_line(&mut first);
        let mut length = 0usize;
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok_and(|n| n > 2) {
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = v.trim().parse().unwrap_or(0);
            }
            line.clear();
        }
        let mut body = vec![0u8; length];
        let _read = reader.read_exact(&mut body);
        log.lock().expect("the log").push(first.trim().to_owned());
        let body = r#"{"ok":true,"version":"0.0.0","instance":"node-1","startedAt":"2026-01-01T00:00:00.000Z","runs":{"running":0,"queued":0}}"#;
        let reply = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _written = stream.write_all(reply.as_bytes());
        let _closed = stream.shutdown(std::net::Shutdown::Write);
    }

    fn requests(&self) -> Vec<String> {
        self.seen.lock().expect("the log").clone()
    }

    fn only_health_calls(&self) -> bool {
        let seen = self.requests();
        !seen.is_empty() && seen.iter().all(|l| l.starts_with("GET /api/health "))
    }
}

impl Home {
    /// A home that the fake Node server holds: its config, and a pid file naming `pid`.
    fn held_by(node: &FakeNode, pid: u32) -> Home {
        let home = Home::new();
        std::fs::create_dir_all(home.dashboard()).expect("the home");
        let config = serde_json::json!({
            "host": "127.0.0.1", "port": node.port, "token": NODE_TOKEN, "requireToken": false
        });
        std::fs::write(home.dashboard().join("config.json"), config.to_string()).expect("a config");
        let record = serde_json::json!({
            "pid": pid, "port": node.port, "startedAt": "2026-01-01T00:00:00.000Z", "instance": "node-1"
        });
        std::fs::write(home.pid_file(), record.to_string()).expect("a pid file");
        home
    }
}

#[test]
fn the_verbs_that_change_a_server_refuse_the_node_dashboard() {
    let node = FakeNode::start();
    let home = Home::held_by(&node, std::process::id());
    for args in [
        &["stop"][..],
        &["start"],
        &["open", "--print"],
        &["resync"],
        &["token", "--rotate"],
    ] {
        let out = home.run(args);
        assert_eq!(out.code, 1, "{args:?}: {}", out.stdout);
        assert!(
            out.stderr.contains("Node dashboard"),
            "{args:?}: {}",
            out.stderr
        );
    }
    assert_eq!(
        home.json("config.json")["token"],
        NODE_TOKEN,
        "the token is untouched"
    );
    let status = home.run(&["status"]);
    assert_eq!(status.code, 0);
    assert!(
        status.stdout.contains("the Node dashboard running at"),
        "{}",
        status.stdout
    );
    let json: Value = serde_json::from_str(&home.run(&["status", "--json"]).stdout).expect("JSON");
    assert_eq!(json["running"], true);
    assert_eq!(json["impl"], "node");
    assert!(
        node.only_health_calls(),
        "the Node server only saw health calls: {:?}",
        node.requests()
    );
}

#[test]
fn serve_does_not_open_a_home_the_node_dashboard_uses() {
    let node = FakeNode::start();
    let live = Home::held_by(&node, std::process::id());
    let refused = live.run(&["serve"]);
    assert_eq!(refused.code, 1, "{}", refused.stdout);
    assert!(refused.stderr.contains("--data-dir"), "{}", refused.stderr);
    assert!(
        refused.stderr.contains("Node dashboard"),
        "{}",
        refused.stderr
    );
    // The pid is gone but the port still answers with the Node health: still refused.
    let gone = Home::held_by(&node, 2_000_000_000);
    let refused = gone.run(&["serve"]);
    assert_eq!(refused.code, 1, "{}", refused.stdout);
    assert!(refused.stderr.contains("--data-dir"), "{}", refused.stderr);
    assert!(node.only_health_calls());
}

/// A port nobody holds right now.
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a port");
    listener.local_addr().expect("an address").port()
}

#[test]
fn without_a_flag_or_the_environment_the_server_takes_the_port_of_the_config() {
    let home = Home::new();
    std::fs::create_dir_all(home.dashboard()).expect("the home");
    let port = free_port();
    let config = serde_json::json!({ "host": "127.0.0.1", "port": port });
    std::fs::write(home.dashboard().join("config.json"), config.to_string()).expect("a config");
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_argus"))
            .args(args)
            .env("COORD_HOME", home.dir.path())
            .env("DASHBOARD_HOME", home.dashboard())
            .env_remove("DASHBOARD_PORT")
            .env(PROBE_ENV, "60000")
            .output()
            .expect("argus runs")
    };
    let start = run(&["start"]);
    assert_eq!(start.status.code(), Some(0), "{start:?}");
    assert_eq!(
        home.json("server.pid")["port"].as_u64(),
        Some(u64::from(port))
    );
    assert!(run(&["status"]).status.success());
}
