//! S2: coord lead wakes the target lead through an in-process Argus test server.

// test-tag: standard

use crate::lead_verbs::seed;
use crate::remote::{Rig, ok};
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn with_the_dashboard_running_the_target_lead_is_woken_with_the_sender_and_the_first_100_characters()
 {
    let rig = Rig::start(json!({ "wake": { "windowMs": 100 } }), seed).await;
    for argv in [
        &["portfolio", "init"][..],
        &["portfolio", "add", "ghost", "--name", "Ghost"],
    ] {
        ok(&rig.run(argv));
    }
    let star = rig.t.sse("/api/p/brock/wakes/stream?thread=*").await;
    star.next(Some("ready"), 5000).await.expect("ready");
    let long = "z".repeat(120);
    let cwd = rig.dir.path().join("checkout-pixelarttool");
    let ran = rig.run_in(&["lead", "brock", &long, "--kind", "decision"], None, &cwd);
    assert_eq!(ok(&ran), format!("queued for brock lead: {long}\n"));
    let heard = star
        .next(Some("wake"), 5000)
        .await
        .expect("the lead was woken");
    assert_eq!(heard.data["thread"], "lead");
    assert_eq!(
        heard.data["lines"],
        json!([format!("pixelarttool lead: {}", "z".repeat(100))])
    );
    let rows = rig.t.get("/api/p/brock/wakes").await.json();
    let woke: Vec<&Value> = rows["wakes"]
        .as_array()
        .expect("wakes")
        .iter()
        .filter(|w| w["reason"] == "lead-message")
        .collect();
    assert_eq!(woke.len(), 1);
    let note = rig.run_in(&["lead", "brock", "fyi: released 1.2"], None, &cwd);
    assert_eq!(ok(&note), "queued for brock lead: fyi: released 1.2\n");
    let rows = rig.t.get("/api/p/brock/wakes").await.json();
    let info: Vec<&Value> = rows["wakes"]
        .as_array()
        .expect("wakes")
        .iter()
        .filter(|w| w["reason"] == "lead-info")
        .collect();
    assert_eq!(info.len(), 1, "a note is information only");
    assert_eq!(info[0]["status"], "passive", "kept off the listener");
    star.close();
    rig.stop().await;
}
