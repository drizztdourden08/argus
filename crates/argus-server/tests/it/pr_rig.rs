//! What the pull request and check tests share: a server on temp homes whose `checks.command` is a
//! fake `brock` (a Node script, never the real Brock), and the calls the tests make over and over.

use std::path::Path;

use argus_testkit::{TestOptions, TestServer};
use serde_json::{Value, json};

use crate::wait::until;
use tempfile::TempDir;

pub const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

pub struct Rig {
    pub t: TestServer,
    pub out: TempDir,
    pub root: TempDir,
}

pub fn script_source(coord_home: &Path, out: &Path) -> String {
    let coord = json!(coord_home.join("ci").join("alpha").display().to_string());
    let argv = json!(out.join("argv.json").display().to_string());
    format!(
        r#"
const fs = require("fs"), path = require("path");
const args = process.argv.slice(2);
const at = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : "");
const dir = {coord};
fs.mkdirSync(dir, {{ recursive: true }});
const runId = "20261029T000000000Z-" + Date.now().toString(36);
const file = path.join(dir, runId + ".json");
const record = {{ kind: "check-run", id: runId, project: "alpha", commit: at("--commit"), pr: at("--pr"),
  trigger: at("--trigger") || "manual", status: "running", startedAt: new Date().toISOString(), steps: [] }};
fs.writeFileSync(file + ".tmp", JSON.stringify(record));
fs.renameSync(file + ".tmp", file);
fs.writeFileSync({argv}, JSON.stringify(args));
console.log(file);
if (at("--trigger") === "release") setInterval(() => {{}}, 1000);
if (at("--trigger") === "push") fs.rmSync("tracked.txt");
"#
    )
}

pub async fn rig(shadow: bool) -> Rig {
    let (out, root) = (
        tempfile::tempdir().expect("out"),
        tempfile::tempdir().expect("root"),
    );
    let coord = tempfile::tempdir().expect("coord").keep();
    let script = out.path().join("fake-brock.cjs");
    std::fs::write(&script, script_source(&coord, out.path())).expect("fake brock");
    let command = format!("node \"{}\"", script.display());
    let config = json!({
        "checks": { "command": command, "enabled": true },
        "projectRoots": { "alpha": root.path().display().to_string() },
    });
    let t = TestServer::start(TestOptions {
        config,
        seed: true,
        coord_home: Some(coord),
        shadow,
        ..TestOptions::default()
    })
    .await;
    Rig { t, out, root }
}

pub fn pr_body() -> Value {
    json!({ "title": "Add x", "body": "why", "head": "feat/x", "headCommit": SHA, "item": "I-1" })
}

pub async fn open_pr(t: &TestServer) -> Value {
    let got = t.write("POST", "/api/p/alpha/prs", &pr_body()).await;
    assert_eq!(got.status, 200, "{}", got.text());
    got.json()
}

pub async fn start_check(t: &TestServer, body: &Value) -> String {
    let got = t.write("POST", "/api/p/alpha/checks", body).await;
    assert_eq!(got.status, 200, "{}", got.text());
    got.json()["run"].as_str().expect("a run id").to_owned()
}

pub async fn linked_record(t: &TestServer, query: &str, rid: &str) -> Value {
    let path = format!("/api/p/alpha/checks{query}");
    let rid = rid.to_owned();
    until("the record linked to its run", || async {
        let runs = t.get(&path).await.json()["runs"].clone();
        runs.as_array()?
            .iter()
            .find(|r| r["run"] == rid.as_str())
            .cloned()
    })
    .await
}
