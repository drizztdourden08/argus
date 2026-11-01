//! Wake faults end to end, ported from the HTTP side of `wake.faults.test.mjs`: a wake nobody
//! listens for becomes a fault, mirrored to the `registry` docs and told once to the main
//! coordinator as a passive notice (the wake policy never wakes it for a fault); the clear route refuses a browser and anyone but the Overlord. The `coord
//! wake-fault` verbs stay in the Node `coord` tests until the `coord` port of slice 8.

use std::time::{Duration, Instant};

use crate::wake_rig::{pause, post_wake, wake_stream, wakes};
use argus_core::Coll;
use argus_core::clock::now_ms;
use argus_core::ids::PORTFOLIO;
use argus_testkit::{
    Request, SseReader, TestOptions, TestServer, alpha_board, beta_board, make_home, write_board,
};
use serde_json::{Value, json};
use tempfile::TempDir;

const NOTE: &str = "the ui session had crashed, restarted its wake-stream";

struct Faulted {
    t: TestServer,
    main: SseReader,
    _home: TempDir,
}

fn thread_like(board: &Value, from: &str, id: &str) -> Value {
    let mut thread = board["threads"][from].clone();
    thread["id"] = json!(id);
    thread["name"] = json!(id);
    thread
}

fn alpha_threads(home: &std::path::Path) -> Value {
    let mut board = alpha_board(home);
    for id in ["api", "web"] {
        board["threads"][id] = thread_like(&board, "ui", id);
    }
    for thread in board["threads"]
        .as_object_mut()
        .into_iter()
        .flatten()
        .map(|(_, t)| t)
    {
        thread["activity"] = json!("idle");
    }
    board
}

fn overlord_board() -> Value {
    let mut board = beta_board();
    board["project"] = json!("portfolio");
    board
}

fn registry_doc(t: &TestServer, scope: &str, id: &str) -> Option<Value> {
    let row = t.svc.db.docs().get(scope, Coll::Registry, id).ok()??;
    Some(row.data)
}

fn state_of(t: &TestServer, scope: &str, id: &str) -> Option<String> {
    let doc = registry_doc(t, scope, id)?;
    doc["state"].as_str().map(str::to_owned)
}

/// Waits until `check` is true.
async fn until(what: &str, ms: u64, check: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_millis(ms);
    while !check() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        pause(40).await;
    }
}

/// Two wakes for threads nobody listens for, and the main coordinator listening.
async fn faulted() -> Faulted {
    let home = tempfile::tempdir().expect("coord home");
    make_home(home.path());
    write_board(home.path(), "alpha", &alpha_threads(home.path()));
    write_board(home.path(), "portfolio", &overlord_board());
    let config = json!({ "wake": { "windowMs": 30, "noListenerMs": 300, "reactionMs": 60000, "tickMs": 50, "aliveMs": 1 }, "wakePolicy": { "debounceMs": 0 } });
    let t = TestServer::start(TestOptions {
        coord_home: Some(home.path().to_path_buf()),
        config,
        ..TestOptions::default()
    })
    .await;
    let main = wake_stream(&t, "coordinator", "portfolio").await;
    for thread in ["ui", "api"] {
        let body = json!({ "thread": thread, "reason": "thread-message", "text": format!("lead: look at {thread}") });
        assert_eq!(post_wake(&t, "alpha", &body).await.json()["queued"], true);
    }
    until("both faults", 8000, || {
        state_of(&t, "alpha", "ui").as_deref() == Some("fault")
            && state_of(&t, "alpha", "api").as_deref() == Some("fault")
    })
    .await;
    Faulted {
        t,
        main,
        _home: home,
    }
}

async fn clear(t: &TestServer, body: &Value) -> u16 {
    t.write("POST", "/api/wake-faults/clear", body).await.status
}

#[tokio::test(flavor = "multi_thread")]
async fn a_wake_nobody_listens_for_becomes_a_fault_and_the_main_coordinator_gets_one_notice_per_fault_never_a_wake()
 {
    let f = faulted().await;
    let ui = registry_doc(&f.t, "alpha", "ui").expect("the ui row");
    assert_eq!(ui["fault"]["reason"], "no-listener");
    assert!(
        ui["fault"]["detail"]
            .as_str()
            .is_some_and(|d| d.contains("nobody listening"))
    );
    assert_eq!(ui["pendingWake"], Value::Null);
    assert_eq!(
        state_of(&f.t, "alpha", "coordinator").as_deref(),
        Some("idle")
    );
    assert_eq!(
        state_of(&f.t, PORTFOLIO, "alpha/ui").as_deref(),
        Some("fault"),
        "the portfolio page shows it"
    );
    assert!(registry_doc(&f.t, PORTFOLIO, "alpha/coordinator").is_some());
    assert_eq!(
        registry_doc(&f.t, PORTFOLIO, "alpha/api").expect("api")["project"],
        "alpha"
    );

    let rows: Vec<Value> = wakes(&f.t, "portfolio")
        .await
        .into_iter()
        .filter(|w| w["reason"] == "wake-fault")
        .collect();
    assert_eq!(rows.len(), 2, "one notice per fault");
    assert!(
        rows.iter().all(|w| w["status"] == "passive"),
        "a wake fault is a notice on the board, never a wake"
    );
    let texts: Vec<&str> = rows.iter().filter_map(|w| w["text"].as_str()).collect();
    let mut who: Vec<&str> = texts.iter().filter_map(|l| l.split(':').next()).collect();
    who.sort_unstable();
    assert_eq!(who, ["alpha/api", "alpha/ui"]);
    let line = texts
        .iter()
        .find(|l| l.starts_with("alpha/ui"))
        .expect("the ui line");
    assert!(
        line.starts_with("alpha/ui: no-listener (queued for "),
        "{line}"
    );
    assert!(line.ends_with("s, nobody listening)"), "{line}");
    assert_eq!(f.main.queued(), 0, "the main coordinator was not woken");
    two_fault_notices_in_the_digest(&f.t).await;
}

/// The main coordinator's prompt digest counts the two fault notices.
async fn two_fault_notices_in_the_digest(t: &TestServer) {
    let digest = t
        .get("/api/p/portfolio/wakes/notices?thread=coordinator")
        .await
        .json();
    assert_eq!(digest["count"], 2);
    assert_eq!(
        digest["kinds"],
        json!([{ "reason": "wake-fault", "count": 2 }])
    );
    assert!(
        digest["line"]
            .as_str()
            .is_some_and(|l| l.starts_with("Board notices since your last turn: 2 (wake-fault 2)."))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_listening_thread_is_waking_until_its_next_sign_of_life_then_listening_with_no_fault() {
    let f = faulted().await;
    let t = &f.t;
    let path = "/api/p/alpha/wakes/stream?thread=web&v=2&id=web%20session";
    let s = t.sse(path).await;
    s.next(Some("ready"), 5000).await.expect("ready");
    until("listening", 8000, || {
        state_of(t, "alpha", "web").as_deref() == Some("listening")
    })
    .await;
    assert_eq!(
        registry_doc(t, "alpha", "web").expect("web")["listenerLabel"],
        "web session"
    );
    let body = json!({ "thread": "web", "reason": "thread-message", "text": "lead: look at web" });
    post_wake(t, "alpha", &body).await;
    assert!(s.next(Some("wake"), 5000).await.is_some(), "delivered");
    until("waking", 8000, || {
        state_of(t, "alpha", "web").as_deref() == Some("waking")
    })
    .await;
    assert_eq!(
        registry_doc(t, "alpha", "web").expect("web")["pendingWake"]["reason"],
        "thread-message"
    );

    let mut board = t.board("alpha");
    board["threads"]["web"]["lastSeen"] = json!(argus_core::clock::iso_from_ms(now_ms() + 5000));
    write_board(&t.coord_home, "alpha", &board);
    until("listening again", 10000, || {
        state_of(t, "alpha", "web").as_deref() == Some("listening")
    })
    .await;
    assert_eq!(
        registry_doc(t, "alpha", "web").expect("web")["pendingWake"],
        Value::Null
    );
    s.close();
    until("idle", 8000, || {
        state_of(t, "alpha", "web").as_deref() == Some("idle")
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_faults_list_for_anyone_open_ones_by_default_and_all_with_a_flag() {
    let f = faulted().await;
    let list = f.t.get("/api/wake-faults").await.json();
    let mut names: Vec<String> = list["faults"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|x| {
            format!(
                "{}/{}",
                x["project"].as_str().unwrap_or_default(),
                x["thread"].as_str().unwrap_or_default()
            )
        })
        .collect();
    names.sort();
    assert_eq!(names, ["alpha/api", "alpha/ui"]);
    assert_eq!(
        f.t.get("/api/wake-faults?all=1").await.json()["faults"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_clear_route_refuses_a_browser_request_with_or_without_the_bearer() {
    let f = faulted().await;
    let t = &f.t;
    let body = json!({ "project": "alpha", "thread": "ui", "note": NOTE, "by": "coordinator" });
    let page = t
        .send(t.page(Request::new("POST", "/api/wake-faults/clear").json(&body)))
        .await;
    assert_eq!(page.status, 403);
    assert_eq!(page.code(), "forbidden");
    assert!(
        page.json()["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("cannot be cleared from the page"))
    );
    let meta = Request::new("POST", "/api/wake-faults/clear")
        .json(&body)
        .header("Sec-Fetch-Site", "same-origin");
    assert_eq!(t.send(t.bearer(meta)).await.status, 403);
    assert_eq!(state_of(t, "alpha", "ui").as_deref(), Some("fault"));
}

/// A clear request for alpha/ui as the main coordinator, with `extra` laid over it.
fn with(extra: &Value) -> Value {
    let mut body = json!({ "project": "alpha", "thread": "ui", "by": "coordinator" });
    body.as_object_mut()
        .expect("object")
        .extend(extra.as_object().cloned().unwrap_or_default());
    body
}

#[tokio::test(flavor = "multi_thread")]
async fn the_clear_route_needs_a_note_and_the_main_coordinator() {
    let f = faulted().await;
    let t = &f.t;
    assert_eq!(
        clear(
            t,
            &json!({ "project": "alpha", "thread": "ui", "by": "coordinator" })
        )
        .await,
        400,
        "no note"
    );
    assert_eq!(clear(t, &with(&json!({ "note": "too short" }))).await, 400);
    assert_eq!(
        clear(t, &with(&json!({ "note": "x".repeat(301) }))).await,
        400
    );
    assert_eq!(
        clear(t, &with(&json!({ "thread": "*", "note": NOTE }))).await,
        400
    );
    for by in ["api", "ui", "nobody", ""] {
        let r = t
            .write(
                "POST",
                "/api/wake-faults/clear",
                &with(&json!({ "by": by, "note": NOTE })),
            )
            .await;
        assert_eq!(r.status, 403, "by \"{by}\"");
        assert_eq!(r.code(), "forbidden");
    }
    assert_eq!(
        clear(t, &with(&json!({ "thread": "nothing", "note": NOTE }))).await,
        404
    );
    assert_eq!(state_of(t, "alpha", "ui").as_deref(), Some("fault"));
}

#[tokio::test(flavor = "multi_thread")]
async fn the_main_coordinator_clears_a_fault_with_a_note() {
    let f = faulted().await;
    let t = &f.t;
    let done = t
        .write(
            "POST",
            "/api/wake-faults/clear",
            &with(&json!({ "note": NOTE })),
        )
        .await;
    assert_eq!(done.status, 200);
    assert_eq!(done.json()["fault"]["clearedBy"], "coordinator");
    until("the row to go idle", 8000, || {
        state_of(t, "alpha", "ui").as_deref() == Some("idle")
    })
    .await;
    assert!(
        registry_doc(t, PORTFOLIO, "alpha/ui").is_none(),
        "a cleared non-lead row leaves the portfolio strip"
    );
    assert_eq!(
        clear(t, &with(&json!({ "note": NOTE }))).await,
        404,
        "already cleared"
    );
    let all = t.get("/api/wake-faults?all=1").await.json();
    let ui = all["faults"]
        .as_array()
        .and_then(|l| l.iter().find(|f| f["thread"] == "ui"))
        .cloned()
        .expect("the cleared fault");
    assert_eq!(ui["clearedBy"], "coordinator");
    assert_eq!(ui["note"], NOTE);
}
