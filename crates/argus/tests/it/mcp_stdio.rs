//! The stdio bridge on the real machine seams, ported from `mcp.stdio.test.mjs`: the endpoint
//! read from `config.json` on every call, and the boot rule that makes a quit flag from an
//! earlier boot count for nothing. The rest of the bridge is in `argus-mcp`; the real `argus mcp`
//! process is in `mcp_stdio_process`. Nothing here starts a server: the start is a counter.
//! K1, K2: the config and boot rules of the dashboard environment.
// test-tag: critical

use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, UNIX_EPOCH};

use argus::mcp::{Clock, DashboardEnv, Launch};
use argus_core::Paths;
use argus_core::clock::now_ms;
use argus_mcp::{BridgeEnv, BridgeOptions, run_bridge};
use argus_native::{Boot, TrayFiles, write_quit_flag};
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
        start_wait: Duration::from_millis(5),
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

pub fn init() -> Value {
    let client = json!({ "name": "stdio-port", "version": "1" });
    let params =
        json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": client });
    json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": params })
}

fn error_of(reply: &Value) -> &Value {
    &reply["result"]["structuredContent"]["error"]
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
