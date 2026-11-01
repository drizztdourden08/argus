//! E2, K2: the real `argus mcp` process answers locally with no config, exits when stdin ends,
//! and forwards a call to a live server with the default project. Its boot start is off.
// test-tag: e2e

use std::fs;
use std::io::{BufRead, BufReader as StdReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::mcp_stdio::init;
use argus_testkit::{TestOptions, TestServer};
use serde_json::{Value, json};
use tempfile::TempDir;

async fn server(config: Value) -> TestServer {
    TestServer::start(TestOptions {
        seed: true,
        config,
        ..TestOptions::default()
    })
    .await
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
