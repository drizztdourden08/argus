//! The stdio bridge: answers `initialize` and `tools/list` itself, forwards the rest to a
//! running server over HTTP, starts a down server once and says so when it cannot, and keeps
//! stdout to JSON-RPC lines only. The cases of `mcp.stdio` that do not touch the config file
//! or the quit flag file, which belong to the binary.
//! K2: the stdio bridge on its in-memory wire: local answers, the start rules, the quit flag.
// test-tag: critical

use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use argus_mcp::{BridgeEnv, BridgeOptions, CLOSED_FROM_TRAY, Endpoint, run_bridge};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, duplex};
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::support::call;
use crate::support::stub::TOKEN;

#[derive(Default)]
struct Inner {
    endpoint: Mutex<Option<Endpoint>>,
    quit: AtomicBool,
    launches: AtomicUsize,
    launch_error: Mutex<Option<String>>,
    logs: Mutex<Vec<String>>,
}

#[derive(Clone, Default)]
pub struct Env(Arc<Inner>);

impl Env {
    pub fn at(endpoint: Option<Endpoint>) -> Self {
        let env = Self::default();
        *env.0.endpoint.lock().expect("lock") = endpoint;
        env
    }

    pub fn port(port: u16) -> Self {
        Self::at(Some(Endpoint::loopback(port, Some(TOKEN), false)))
    }

    pub fn launches(&self) -> usize {
        self.0.launches.load(Ordering::SeqCst)
    }

    pub fn logs(&self) -> Vec<String> {
        self.0.logs.lock().expect("lock").clone()
    }
}

impl BridgeEnv for Env {
    fn endpoint(&self) -> Option<Endpoint> {
        self.0.endpoint.lock().expect("lock").clone()
    }

    fn quit_flag_active(&self) -> bool {
        self.0.quit.load(Ordering::SeqCst)
    }

    fn launch(&self) -> Result<(), String> {
        self.0.launches.fetch_add(1, Ordering::SeqCst);
        self.0
            .launch_error
            .lock()
            .expect("lock")
            .clone()
            .map_or(Ok(()), Err)
    }

    fn log(&self, line: &str) {
        self.0.logs.lock().expect("lock").push(line.to_owned());
    }
}

pub fn opts(project: Option<&str>, boot: bool, wait_ms: u64) -> BridgeOptions {
    BridgeOptions {
        project: project.map(str::to_owned),
        boot_launch: boot,
        start_wait: Duration::from_millis(wait_ms),
        call_timeout: Duration::from_secs(20),
    }
}

pub struct Wire {
    to_bridge: DuplexStream,
    from_bridge: BufReader<DuplexStream>,
    task: JoinHandle<io::Result<()>>,
    lines: Vec<String>,
}

impl Wire {
    pub fn start(env: Env, options: BridgeOptions) -> Wire {
        let (to_bridge, bridge_in) = duplex(1 << 16);
        let (bridge_out, from_bridge) = duplex(1 << 16);
        let task = tokio::spawn(run_bridge(
            env,
            options,
            BufReader::new(bridge_in),
            bridge_out,
        ));
        Wire {
            to_bridge,
            from_bridge: BufReader::new(from_bridge),
            task,
            lines: Vec::new(),
        }
    }

    pub async fn raw(&mut self, text: &str) {
        self.to_bridge
            .write_all(format!("{text}\n").as_bytes())
            .await
            .expect("write");
    }

    pub(super) async fn line(&mut self) -> Option<Value> {
        let mut line = String::new();
        let read = timeout(
            Duration::from_secs(20),
            self.from_bridge.read_line(&mut line),
        )
        .await
        .expect("no answer from the bridge");
        if read.expect("read") == 0 {
            return None;
        }
        self.lines.push(line.trim_end().to_owned());
        serde_json::from_str(&line).ok()
    }

    pub async fn send(&mut self, msg: Value) -> Value {
        self.raw(&msg.to_string()).await;
        self.line().await.expect("a reply")
    }

    pub async fn end(mut self) -> Vec<String> {
        self.to_bridge.shutdown().await.expect("close stdin");
        while self.line().await.is_some() {}
        let done = timeout(Duration::from_secs(20), self.task)
            .await
            .expect("the bridge ends");
        done.expect("joined").expect("the bridge returns Ok");
        self.lines
    }
}

pub fn init() -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "bridge-test", "version": "0" } } })
}

pub fn error_of(reply: &Value) -> &Value {
    &reply["result"]["structuredContent"]["error"]
}

#[test]
fn the_loopback_endpoint_carries_the_token_only_when_the_config_requires_it() {
    let plain = Endpoint::loopback(4000, Some("tok"), false);
    assert_eq!(
        plain,
        Endpoint {
            base: "http://127.0.0.1:4000".into(),
            token: None
        }
    );
    assert_eq!(
        Endpoint::loopback(4000, Some("tok"), true).token.as_deref(),
        Some("tok")
    );
    assert_eq!(Endpoint::loopback(4000, None, true).token, None);
}

#[tokio::test]
async fn initialize_tools_list_ping_and_templates_are_answered_locally_with_no_server_and_no_config()
 {
    let mut wire = Wire::start(Env::at(None), opts(Some("alpha"), false, 5));
    let reply = wire.send(init()).await;
    assert_eq!(reply["result"]["serverInfo"]["name"], "coord-dashboard");
    wire.raw(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string())
        .await;
    let list = wire
        .send(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
        .await;
    let tools = list["result"]["tools"].as_array().expect("tools");
    assert_eq!(tools.len(), 25);
    assert!(tools.iter().any(|t| t["name"] == "board_summary"));
    assert_eq!(
        wire.send(json!({ "jsonrpc": "2.0", "id": 3, "method": "ping" }))
            .await["result"],
        json!({})
    );
    let templates = wire
        .send(json!({ "jsonrpc": "2.0", "id": 4, "method": "resources/templates/list" }))
        .await;
    assert_eq!(
        templates["result"]["resourceTemplates"]
            .as_array()
            .expect("templates")
            .len(),
        4
    );
    let lines = wire.end().await;
    assert_eq!(
        lines.len(),
        4,
        "one line per request, none for the notification"
    );
    for l in &lines {
        assert_eq!(
            serde_json::from_str::<Value>(l).expect("JSON")["jsonrpc"],
            "2.0"
        );
    }
}

#[tokio::test]
async fn a_garbage_line_gets_a_parse_error_and_the_bridge_keeps_going() {
    let mut wire = Wire::start(Env::at(None), opts(None, false, 5));
    wire.raw("not json").await;
    let error = wire.line().await.expect("an error line");
    assert_eq!(error["error"]["code"], -32700);
    assert_eq!(error["error"]["message"], "not valid JSON");
    let pong = wire
        .send(json!({ "jsonrpc": "2.0", "id": 9, "method": "ping" }))
        .await;
    assert_eq!(pong["result"], json!({}));
    wire.end().await;
}

#[tokio::test]
async fn with_the_server_down_the_first_call_starts_it_once_waits_then_says_not_running() {
    let env = Env::port(1);
    let mut wire = Wire::start(env.clone(), opts(None, false, 5));
    wire.send(init()).await;
    let first = wire.send(call(2, "health", json!({}))).await;
    assert_eq!(first["result"]["isError"], true);
    assert!(
        error_of(&first)["message"]
            .as_str()
            .is_some_and(|m| m.starts_with("dashboard not running: ")
                && m.ends_with(". Run `dashboard start` and try again."))
    );
    assert_eq!(error_of(&first)["code"], "unavailable");
    assert_eq!(env.launches(), 1, "the start command ran");
    let second = wire.send(call(3, "health", json!({}))).await;
    assert_eq!(second["result"]["isError"], true);
    assert_eq!(env.launches(), 1, "it tried to start the server only once");
    let res = wire.send(json!({ "jsonrpc": "2.0", "id": 4, "method": "resources/read", "params": { "uri": "portfolio://overview" } })).await;
    assert_eq!(res["error"]["code"], -32603);
    for l in wire.end().await {
        serde_json::from_str::<Value>(&l).expect("stdout is JSON-RPC only");
    }
}

#[tokio::test]
async fn the_server_is_started_as_soon_as_the_bridge_runs_and_initialize_does_not_wait_for_it() {
    let env = Env::port(1);
    let mut wire = Wire::start(env.clone(), opts(None, true, 5));
    let reply = wire.send(init()).await;
    assert_eq!(reply["result"]["serverInfo"]["name"], "coord-dashboard");
    let list = wire
        .send(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
        .await;
    assert_eq!(list["result"]["tools"].as_array().expect("tools").len(), 25);
    assert_eq!(env.launches(), 1, "the launcher ran without any tools/call");
    assert!(
        env.logs()
            .iter()
            .any(|l| l == "[coord-dashboard mcp] starting the dashboard")
    );
    wire.end().await;
}

#[tokio::test]
async fn with_the_quit_flag_nothing_is_started_and_a_call_says_how_to_reopen_it() {
    let env = Env::port(1);
    env.0.quit.store(true, Ordering::SeqCst);
    let mut wire = Wire::start(env.clone(), opts(None, true, 5));
    wire.send(init()).await;
    let r = wire.send(call(2, "health", json!({}))).await;
    assert_eq!(r["result"]["isError"], true);
    assert_eq!(error_of(&r)["code"], "closed_from_tray");
    assert_eq!(
        error_of(&r)["message"],
        "the dashboard was closed from the tray; run `dashboard tray start` to reopen it"
    );
    assert_eq!(error_of(&r)["message"], CLOSED_FROM_TRAY);
    assert_eq!(
        error_of(&wire.send(call(3, "health", json!({}))).await)["code"],
        "closed_from_tray"
    );
    let res = wire.send(json!({ "jsonrpc": "2.0", "id": 4, "method": "resources/read", "params": { "uri": "portfolio://overview" } })).await;
    assert!(
        res["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("closed from the tray"))
    );
    assert_eq!(env.launches(), 0, "it never tried to start the dashboard");
    wire.end().await;
}

#[tokio::test]
async fn a_launch_that_fails_says_why_in_the_not_running_message() {
    let env = Env::port(1);
    *env.0.launch_error.lock().expect("lock") = Some("no such file".to_owned());
    let mut wire = Wire::start(env, opts(None, false, 5));
    wire.send(init()).await;
    let r = wire.send(call(2, "health", json!({}))).await;
    assert!(
        error_of(&r)["message"]
            .as_str()
            .is_some_and(|m| m.starts_with("dashboard not running: no such file."))
    );
    wire.end().await;
}
