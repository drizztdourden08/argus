//! The stdio bridge on the real machine seams, ported from `mcp.stdio.test.mjs` and the bridge
//! case of `mcp.server.test.mjs`: the endpoint read from `config.json` on every call, the token
//! sent only when the config requires it, the owner's `quit.flag` and the boot rule that makes a
//! flag from an earlier boot count for nothing, the server started once per down episode, and one
//! run of the real `argus mcp` process. The protocol cases are in `argus-mcp`. Nothing here
//! starts a real server: the start is a counter, and the process test keeps its boot start off.

use std::fs::{self, File};
use std::io::{BufRead, BufReader as StdReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, UNIX_EPOCH};

use argus::mcp::{Clock, DashboardEnv, Launch};
use argus_core::Paths;
use argus_core::clock::now_ms;
use argus_mcp::{BridgeEnv, BridgeOptions, CLOSED_FROM_TRAY, run_bridge};
use argus_native::{Boot, TrayFiles, write_quit_flag};
use argus_testkit::{TestOptions, TestServer};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, Lines, duplex};
use tokio::task::JoinHandle;
use tokio::time::timeout;

const HOUR_MS: u64 = 3_600_000;

struct Home {
    dir: TempDir,
    launches: Arc<AtomicUsize>,
}

impl Home {
    fn new() -> Self {
        Home {
            dir: tempfile::tempdir().expect("temp home"),
            launches: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn config(&self, config: &Value) {
        fs::write(self.path().join("config.json"), config.to_string()).expect("config");
    }

    fn files(&self) -> TrayFiles {
        TrayFiles::new(self.path())
    }

    fn count(&self) -> usize {
        self.launches.load(Ordering::SeqCst)
    }

    fn env(&self) -> DashboardEnv {
        let counted = Arc::clone(&self.launches);
        let launch: Launch = Box::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        let clock: Clock = Box::new(|| {
            Some(Boot {
                now_ms: now_ms().unsigned_abs(),
                uptime_ms: HOUR_MS,
            })
        });
        DashboardEnv::with(
            Paths::new(self.path(), Some(self.path().to_path_buf())),
            launch,
            clock,
        )
    }
}

fn opts(project: Option<&str>, boot: bool) -> BridgeOptions {
    BridgeOptions {
        project: project.map(str::to_owned),
        boot_launch: boot,
        start_wait: Duration::from_millis(600),
        call_timeout: Duration::from_secs(20),
    }
}

struct Wire {
    to_bridge: DuplexStream,
    replies: Lines<BufReader<DuplexStream>>,
    task: JoinHandle<std::io::Result<()>>,
}

impl Wire {
    fn start(env: DashboardEnv, options: BridgeOptions) -> Wire {
        let (to_bridge, stdin) = duplex(1 << 16);
        let (stdout, replies) = duplex(1 << 16);
        let task = tokio::spawn(run_bridge(env, options, BufReader::new(stdin), stdout));
        Wire {
            to_bridge,
            replies: BufReader::new(replies).lines(),
            task,
        }
    }

    async fn send(&mut self, msg: Value) -> Value {
        self.to_bridge
            .write_all(
                format!(
                    "{msg}
"
                )
                .as_bytes(),
            )
            .await
            .expect("write");
        let next = timeout(Duration::from_secs(20), self.replies.next_line());
        let line = next.await.expect("an answer").expect("read");
        serde_json::from_str(&line.expect("a line")).expect("a JSON-RPC line")
    }

    async fn call(&mut self, id: u64, name: &str, args: Value) -> Value {
        let params = json!({ "name": name, "arguments": args });
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": params }))
            .await
    }

    async fn end(mut self) {
        self.to_bridge.shutdown().await.expect("close stdin");
        let ended = timeout(Duration::from_secs(20), self.task).await;
        ended
            .expect("the bridge ends")
            .expect("joined")
            .expect("Ok");
    }
}

fn init() -> Value {
    let client = json!({ "name": "stdio-port", "version": "1" });
    let params =
        json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": client });
    json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": params })
}

fn error_of(reply: &Value) -> &Value {
    &reply["result"]["structuredContent"]["error"]
}

async fn server(config: Value) -> TestServer {
    TestServer::start(TestOptions {
        seed: true,
        config,
        ..TestOptions::default()
    })
    .await
}

#[test]
fn the_endpoint_is_read_from_config_json_on_every_call() {
    let home = Home::new();
    let env = home.env();
    assert_eq!(env.endpoint(), None, "no config yet");
    fs::write(home.path().join("config.json"), "{ nope").expect("write");
    assert_eq!(env.endpoint(), None, "a config that is not JSON");
    home.config(&json!({ "token": "t" }));
    assert_eq!(env.endpoint(), None, "a config with no port");
    home.config(&json!({ "port": 0 }));
    assert_eq!(env.endpoint(), None, "port 0 is no port");
    home.config(&json!({ "port": 4100, "token": "secret" }));
    let plain = env.endpoint().expect("endpoint");
    assert_eq!(
        (plain.base.as_str(), plain.token),
        ("http://127.0.0.1:4100", None)
    );
    home.config(&json!({ "port": 4200, "token": "secret", "requireToken": true }));
    let hardened = env.endpoint().expect("endpoint");
    assert_eq!(hardened.base, "http://127.0.0.1:4200");
    assert_eq!(hardened.token.as_deref(), Some("secret"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_call_goes_through_with_the_default_project_and_no_token() {
    let t = server(json!({})).await;
    let home = Home::new();
    home.config(&json!({ "port": t.port, "token": "x".repeat(64) }));
    let mut wire = Wire::start(home.env(), opts(Some("beta"), false));
    let reply = wire.call(1, "list_items", json!({})).await;
    assert_eq!(
        reply["result"]["structuredContent"]["items"][0]["title"],
        "Health endpoint"
    );
    wire.end().await;
    assert_eq!(home.count(), 0, "a live server is never started again");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn with_require_token_the_bridge_sends_it_and_a_wrong_one_reads_as_not_running() {
    let t = server(json!({ "requireToken": true })).await;
    let home = Home::new();
    home.config(&json!({ "port": t.port, "token": t.token, "requireToken": true }));
    let mut wire = Wire::start(home.env(), opts(Some("beta"), false));
    let ok = wire.call(1, "board_summary", json!({})).await;
    assert_eq!(ok["result"]["isError"], false, "{ok}");
    home.config(&json!({ "port": t.port, "token": "wrong".repeat(13), "requireToken": true }));
    let down = wire.call(2, "health", json!({})).await;
    assert_eq!(down["result"]["isError"], true);
    assert_eq!(error_of(&down)["code"], "unavailable");
    let message = error_of(&down)["message"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(message.contains("dashboard not running"), "{message}");
    wire.end().await;
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_first_call_starts_it_once_waits_then_says_the_dashboard_is_not_running() {
    let home = Home::new();
    home.config(&json!({ "port": 1, "token": "x".repeat(64) }));
    let mut wire = Wire::start(home.env(), opts(None, false));
    wire.send(init()).await;
    let first = wire.call(2, "health", json!({})).await;
    assert_eq!(first["result"]["isError"], true);
    assert_eq!(error_of(&first)["code"], "unavailable");
    assert!(
        error_of(&first)["message"]
            .as_str()
            .is_some_and(|m| m.contains("dashboard not running"))
    );
    assert_eq!(home.count(), 1, "the start command ran");
    let second = wire.call(3, "health", json!({})).await;
    assert_eq!(second["result"]["isError"], true);
    assert_eq!(home.count(), 1, "it tried to start the server only once");
    wire.end().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_server_is_started_as_soon_as_the_bridge_runs() {
    let home = Home::new();
    home.config(&json!({ "port": 1, "token": "x".repeat(64) }));
    let mut wire = Wire::start(home.env(), opts(None, true));
    let init = wire.send(init()).await;
    assert_eq!(init["result"]["serverInfo"]["name"], "coord-dashboard");
    let list = wire
        .send(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
        .await;
    assert_eq!(list["result"]["tools"].as_array().map(Vec::len), Some(23));
    assert_eq!(home.count(), 1, "the start ran without any tools/call");
    wire.end().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_quit_flag_nothing_is_started_and_a_call_says_how_to_reopen_it() {
    let home = Home::new();
    home.config(&json!({ "port": 1, "token": "x".repeat(64) }));
    write_quit_flag(&home.files()).expect("quit flag");
    let mut wire = Wire::start(home.env(), opts(None, true));
    wire.send(init()).await;
    let first = wire.call(2, "health", json!({})).await;
    assert_eq!(first["result"]["isError"], true);
    assert_eq!(error_of(&first)["code"], "closed_from_tray");
    assert_eq!(error_of(&first)["message"], CLOSED_FROM_TRAY);
    let again = wire.call(3, "health", json!({})).await;
    assert_eq!(error_of(&again)["code"], "closed_from_tray");
    let resource = json!({ "jsonrpc": "2.0", "id": 4, "method": "resources/read",
        "params": { "uri": "portfolio://overview" } });
    let read = wire.send(resource).await;
    assert!(
        read["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("closed from the tray"))
    );
    assert_eq!(home.count(), 0, "it never tried to start the dashboard");
    assert!(
        home.files().quit_flag().exists(),
        "the flag is left to `argus tray start`"
    );
    wire.end().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_quit_flag_from_before_this_boot_is_ignored_and_the_server_is_started() {
    let home = Home::new();
    home.config(&json!({ "port": 1, "token": "x".repeat(64) }));
    write_quit_flag(&home.files()).expect("quit flag");
    let file = File::options()
        .write(true)
        .open(home.files().quit_flag())
        .expect("open");
    file.set_modified(UNIX_EPOCH + Duration::from_secs(978_307_200))
        .expect("old flag");
    drop(file);
    let mut wire = Wire::start(home.env(), opts(None, true));
    wire.send(init()).await;
    assert_eq!(home.count(), 1);
    let call = wire.call(2, "health", json!({})).await;
    assert_eq!(
        error_of(&call)["code"],
        "unavailable",
        "down, not closed from the tray"
    );
    wire.end().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn without_the_flag_a_crashed_dashboard_is_started_again_once_per_crash() {
    let t = server(json!({})).await;
    let home = Home::new();
    home.config(&json!({ "port": t.port, "token": t.token }));
    let mut wire = Wire::start(home.env(), opts(None, true));
    wire.send(init()).await;
    assert_eq!(home.count(), 1, "started with the bridge");
    let up = wire
        .call(2, "board_summary", json!({ "project": "alpha" }))
        .await;
    assert_eq!(up["result"]["isError"], false);
    t.stop(false).await;
    let down = wire.call(3, "health", json!({})).await;
    assert_eq!(down["result"]["isError"], true);
    assert_eq!(
        home.count(),
        2,
        "the crash was answered with a second start"
    );
    wire.call(4, "health", json!({})).await;
    assert_eq!(
        home.count(),
        2,
        "not again while that start is still pending"
    );
    wire.end().await;
}

/// The real `argus mcp` process with its boot start off, talking to the given home.
struct Child {
    process: std::process::Child,
    out: StdReader<std::process::ChildStdout>,
}

impl Child {
    fn start(home: &Path, project: Option<&str>) -> Child {
        let mut command = Command::new(env!("CARGO_BIN_EXE_argus"));
        command.arg("mcp").arg("--data-dir").arg(home);
        if let Some(key) = project {
            command.args(["--project", key]);
        }
        let mut process = command
            .env("COORD_HOME", home)
            .env("COORD_DASHBOARD_NO_BOOT_LAUNCH", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("argus mcp starts");
        let out = StdReader::new(process.stdout.take().expect("stdout"));
        Child { process, out }
    }

    fn send(&mut self, msg: &Value) -> Value {
        let stdin = self.process.stdin.as_mut().expect("stdin");
        writeln!(stdin, "{msg}").expect("write");
        stdin.flush().expect("flush");
        let mut line = String::new();
        self.out.read_line(&mut line).expect("a line");
        serde_json::from_str(&line).expect("a JSON-RPC line")
    }

    fn end(mut self) -> Option<i32> {
        drop(self.process.stdin.take());
        self.process.wait().ok().and_then(|s| s.code())
    }
}

fn the_home_of_a_process_test() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("home");
    let path = dir.path().to_path_buf();
    (dir, path)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_argus_mcp_process_answers_locally_with_no_config_and_exits_when_stdin_ends() {
    let (_dir, home) = the_home_of_a_process_test();
    let code = tokio::task::spawn_blocking(move || {
        let mut child = Child::start(&home, None);
        let init = child.send(&init());
        assert_eq!(init["result"]["serverInfo"]["name"], "coord-dashboard");
        let list = child.send(&json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }));
        assert_eq!(list["result"]["tools"].as_array().map(Vec::len), Some(23));
        child.end()
    })
    .await
    .expect("joined");
    assert_eq!(code, Some(0));
}

#[tokio::test(flavor = "multi_thread")]
async fn the_argus_mcp_process_forwards_a_call_to_the_live_server_with_the_default_project() {
    let t = server(json!({})).await;
    let (dir, home) = the_home_of_a_process_test();
    fs::write(
        dir.path().join("config.json"),
        json!({ "port": t.port, "token": t.token }).to_string(),
    )
    .expect("config");
    let reply = tokio::task::spawn_blocking(move || {
        let mut child = Child::start(&home, Some("beta"));
        let params = json!({ "name": "list_items", "arguments": {} });
        let call = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": params });
        let reply = child.send(&call);
        child.end();
        reply
    })
    .await
    .expect("joined");
    assert_eq!(
        reply["result"]["structuredContent"]["items"][0]["title"],
        "Health endpoint"
    );
    t.stop(false).await;
}
