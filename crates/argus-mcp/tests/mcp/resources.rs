//! The resources: listing, templates and reads.
//! K2: the resource list and reads.
// test-tag: critical

use argus_mcp::RpcHost;
use serde_json::{Value, json};

use crate::support::{Rig, ctx};

async fn read(rig: &Rig, uri: &str) -> Result<Value, argus_mcp::ToolError> {
    rig.host.read_resource(uri, &ctx()).await
}

fn json_of(contents: &Value) -> Value {
    serde_json::from_str(contents["contents"][0]["text"].as_str().expect("text"))
        .expect("JSON text")
}

#[test]
fn listing_and_templates() {
    let rig = Rig::new();
    let uris: Vec<String> = rig
        .host
        .list_resources()
        .iter()
        .filter_map(|r| r["uri"].as_str().map(str::to_owned))
        .collect();
    for want in [
        "portfolio://overview",
        "board://alpha/summary",
        "board://beta/tests/to-run",
        "runs://alpha/active",
    ] {
        assert!(uris.iter().any(|u| u == want), "{want}");
    }
    assert!(
        !uris.iter().any(|u| u.contains("gamma")),
        "a project without a board is not listed"
    );
    assert_eq!(uris.len(), 7);
    assert_eq!(rig.host.list_resource_templates().len(), 4);
}

#[tokio::test]
async fn reads_return_the_same_data_as_the_tools() {
    let rig = Rig::new();
    let overview = json_of(&read(&rig, "portfolio://overview").await.expect("overview"));
    assert_eq!(overview["cards"].as_array().expect("cards").len(), 2);
    let summary = json_of(&read(&rig, "board://alpha/summary").await.expect("summary"));
    assert_eq!(summary["project"], "alpha");
    let to_run = json_of(
        &read(&rig, "board://alpha/tests/to-run")
            .await
            .expect("to-run"),
    );
    assert_eq!(to_run["handedRun"]["id"], "I-6");
    let groups: Vec<Value> = to_run["groups"]
        .as_array()
        .expect("groups")
        .iter()
        .map(|g| json!([g["run"], g["build"], g["handed"], g["tests"][0]["id"]]))
        .collect();
    assert_eq!(groups, [json!(["I-6", "build-1", true, "T-1"])]);
    let active = json_of(&read(&rig, "runs://alpha/active").await.expect("active"));
    assert_eq!(
        active["runs"]
            .as_array()
            .expect("runs")
            .iter()
            .map(|r| r["id"].clone())
            .collect::<Vec<_>>(),
        [json!("R-2")]
    );
    let log = read(&rig, "runs://alpha/R-1/log").await.expect("log");
    assert_eq!(log["contents"][0]["mimeType"], "text/plain");
    assert_eq!(log["contents"][0]["text"], "hi\nthere");
}

#[tokio::test]
async fn json_resources_are_pretty_printed_with_two_spaces() {
    let rig = Rig::new();
    let text = read(&rig, "runs://alpha/active").await.expect("active")["contents"][0]["text"]
        .as_str()
        .expect("text")
        .to_owned();
    assert!(text.starts_with("{\n  \"runs\": ["), "{text}");
}

#[tokio::test]
async fn unknown_resources_and_projects_are_not_found() {
    let rig = Rig::new();
    for uri in [
        "board://nope/summary",
        "board://alpha/other",
        "runs://alpha/R-99/log",
        "file:///etc/passwd",
        "garbage",
    ] {
        let e = read(&rig, uri).await.expect_err(uri);
        assert_eq!(e.code, "not_found", "{uri}");
    }
}

#[tokio::test]
async fn a_log_needs_the_runs_service() {
    let rig = Rig::over(argus_mcp::MemoryBackend::new(crate::support::fixture()));
    let e = read(&rig, "runs://alpha/R-1/log").await.expect_err("down");
    assert_eq!(e.code, "unavailable");
    let active = json_of(&read(&rig, "runs://alpha/active").await.expect("active"));
    assert_eq!(active["runs"], json!([]));
}
