//! The stale marks on the launchers docs of an export, as the Node dashboard shows them: the
//! newest Play run of a standalone launcher is a use (`lastUsedAt`, `stale`), an older one
//! changes nothing. Ported from the `overlay` and service cases of `launchers.stale.test.mjs`.

use argus_core::clock::now_ms;
use argus_core::fixture::ago;
use argus_testkit::{TestOptions, TestServer, write_board};
use serde_json::{Value, json};

const DAY_MIN: i64 = 24 * 60;
const DAY_MS: i64 = DAY_MIN * 60_000;

fn launcher(name: &str) -> Value {
    json!({
        "name": name, "cmd": format!("run {name}"), "kind": "app", "current": false,
        "retiredAt": null, "createdAt": ago(9 * DAY_MIN), "createdBy": "worker",
        "updatedAt": ago(9 * DAY_MIN), "by": "worker", "useEvery": "7d",
    })
}

async fn doc(t: &TestServer, name: &str) -> Value {
    t.get(&format!("/api/p/alpha/db/launchers/{name}"))
        .await
        .json()["data"]
        .clone()
}

async fn resync(t: &TestServer) {
    let reply = t
        .write("POST", "/api/admin/resync", &json!({ "project": "alpha" }))
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_newest_play_run_is_a_use_on_the_launchers_docs_of_an_export() {
    let t = TestServer::start(TestOptions::default()).await;
    write_board(
        &t.coord_home,
        "alpha",
        &json!({
            "project": "alpha", "schema": 2,
            "launchers": { "fresh": launcher("fresh"), "old": launcher("old"), "idle": launcher("idle") },
        }),
    );
    resync(&t).await;
    for name in ["fresh", "old", "idle"] {
        assert_eq!(
            doc(&t, name).await["stale"],
            true,
            "{name} is unused for 9 days"
        );
    }
    t.svc
        .db
        .kv_set(
            "launcher-played:alpha:fresh",
            &(now_ms() - DAY_MS).to_string(),
        )
        .expect("kv");
    t.svc
        .db
        .kv_set(
            "launcher-played:alpha:old",
            &(now_ms() - 20 * DAY_MS).to_string(),
        )
        .expect("kv");
    resync(&t).await;
    let fresh = doc(&t, "fresh").await;
    assert_eq!(fresh["stale"], false, "played yesterday");
    assert!(fresh["lastUsedAt"].as_str() > Some(ago(2 * DAY_MIN).as_str()));
    let old = doc(&t, "old").await;
    assert_eq!(old["stale"], true, "an older run changes nothing");
    assert!(old["lastUsedAt"].as_str() < Some(ago(8 * DAY_MIN).as_str()));
    assert_eq!(doc(&t, "idle").await["stale"], true, "never played");
    t.stop(false).await;
}
