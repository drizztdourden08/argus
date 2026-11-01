//! E2: the lead reads a 20000 character send of the Overlord page whole with the real Node
//! `coord inbox`, with no ellipsis. Needs `ARGUS_COORD_CLI`; with no CLI located the case ends at
//! once.
// test-tag: e2e

use std::sync::atomic::{AtomicU32, Ordering};

use crate::wake_rig::rig;
use argus_testkit::{CoordCli, Request, TestServer};
use serde_json::{Value, json};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn post_id() -> String {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    format!("O-{}-{n:04}", argus_core::clock::now_ms())
}

/// A long text with lines, a list and markdown, the way the owner writes a brief.
fn brief(len: usize) -> String {
    let unit = "Plan:\n- first **bold** step\n- second step\n1. numbered\n";
    unit.repeat(len.div_ceil(50))[..len].trim().to_owned()
}

/// A send the way the page makes it: a write of the `_portfolio` entries collection.
async fn from_page(t: &TestServer, extra: Value) -> Value {
    let mut data = json!({ "author": "human", "createdAt": argus_core::clock::now_iso() });
    data.as_object_mut()
        .expect("an object")
        .extend(extra.as_object().expect("an object").clone());
    let request = Request::new("PUT", &format!("/api/p/_portfolio/db/entries/{}", post_id()))
        .json(&json!({ "data": data }));
    let reply = t.send(t.page(request)).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    reply.json()
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn the_lead_reads_it_whole_with_coord_inbox_no_ellipsis_nothing_cut() {
    let Some(cli) = CoordCli::locate() else {
        return;
    };
    let r = rig().await;
    let t = &r.t;
    let text = brief(20000);
    let sent = from_page(
        t,
        json!({ "kind": "followup", "project": "beta", "text": text }),
    )
    .await;
    assert_eq!(sent["status"], "delivered", "{sent}");
    let out = cli
        .run(
            t,
            &["inbox", "--project", "beta", "--as", "coordinator"],
            &t.coord_home,
            &[],
        )
        .await;
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert!(
        out.stdout.contains(&text),
        "all 20000 characters are in the output"
    );
    assert!(
        !out.stdout.contains('…'),
        "nothing is shortened with an ellipsis"
    );
}
