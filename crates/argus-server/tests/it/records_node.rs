//! E2: the registry's Node status script run for real with the registry home, and the failure of a
//! script that exits badly or a Node that is not there.
// test-tag: e2e

use argus_core::fixture::put_file as put;
use argus_server::ConsumerStatus;
use serde_json::{Value, json};

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn the_node_status_runs_the_script_with_the_registry_home_and_reports_a_failure() {
    let dir = tempfile::tempdir().expect("temp dir");
    put(
        dir.path(),
        "ok.mjs",
        "console.log(JSON.stringify({ consumers: [{ root: process.env.LOCALPKG_HOME }], args: process.argv.slice(2) }));",
    );
    put(dir.path(), "bad.mjs", "process.exit(2);");
    let run = |script: &str, node: &str| argus_server::NodeStatus {
        node: node.to_owned(),
        script: dir.path().join(script),
        home: dir.path().to_path_buf(),
    };
    let said = run("ok.mjs", "node").status_json().await.expect("ran");
    let parsed: Value = serde_json::from_str(&said).expect("json");
    assert_eq!(parsed["args"], json!(["status", "--json"]));
    assert_eq!(
        parsed["consumers"][0]["root"],
        dir.path().display().to_string()
    );
    assert!(run("bad.mjs", "node").status_json().await.is_err());
    assert!(
        run("ok.mjs", "no-such-program-argus")
            .status_json()
            .await
            .is_err()
    );
}
