//! K3, K4: a server in shadow mode: it starts from the Node database, a marked home serves in
//! shadow mode whatever the start asked, and the shadow flag writes the marker.
// test-tag: standard

use std::sync::Arc;

use crate::node_home::node_home;
use argus_testkit::{TestOptions, TestServer, Tweak};
use tempfile::TempDir;

#[tokio::test]
async fn serve_in_shadow_mode_starts_from_the_node_database() {
    let dir = TempDir::new().unwrap();
    let (node, _live) = node_home(dir.path());
    let tweak: Tweak = Arc::new(move |o| o.shadow_seed = Some(node.clone()));
    let t = TestServer::start(TestOptions {
        shadow: true,
        tweak: Some(tweak),
        ..TestOptions::default()
    })
    .await;
    assert_eq!(t.svc.db.kv_get("seen").unwrap().as_deref(), Some("yes"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_marked_home_serves_in_shadow_mode_whatever_the_start_asked() {
    let t = TestServer::start(TestOptions {
        shadow: false,
        seed: true,
        config: serde_json::json!({ "shadow": true }),
        ..TestOptions::default()
    })
    .await;
    assert!(
        t.svc.shadow,
        "the marker in config.json turns shadow mode on"
    );
    let health = t.get("/api/health").await.json();
    assert_eq!(health["shadow"], true);
    let note = serde_json::json!({ "project": "beta", "text": "Shadow" });
    let sent = t.write("POST", "/api/portfolio/send", &note).await;
    assert_eq!((sent.status, sent.code().as_str()), (403, "read_only"));
}

#[tokio::test(flavor = "multi_thread")]
async fn the_shadow_flag_writes_the_marker_so_the_next_start_needs_no_flag() {
    let t = TestServer::start(TestOptions {
        shadow: true,
        ..TestOptions::default()
    })
    .await;
    assert!(argus_server::shadow::marked(
        &t.dashboard_home.join("config.json")
    ));
}
