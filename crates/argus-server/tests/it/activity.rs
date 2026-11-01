//! The activity log over HTTP (DESIGN 13.11): the route, the docs of `_portfolio` and their
//! rebuild after a hard resync, the dashboard's own lines, and the adapters that give every
//! service crate its activity port. Ported from the hook and route cases of
//! `activity.test.mjs` and `procs.http.test.mjs`.
//! K2, K3: the activity route shape, the portfolio docs and the sink adapters.
// test-tag: standard

use argus_agents::{ActivityLine as AgentLine, ActivitySink as AgentSink};
use argus_core::{Coll, ResetEvent};
use argus_launchers::launcher::StaleNote;
use argus_notify::{
    ActivityLevel as NoteLevel, ActivityLine as NoteLine, ActivitySink as NoteSink,
};
use argus_procs::{Activity as ProcLine, Level as ProcLevel};
use argus_server::adapters::activity::ActivityAdapter;
use argus_store::ACTIVITY_SCOPE;
use argus_testkit::{TestOptions, TestServer};
use argus_wake::activity::{ActivityLine as WakeLine, ActivitySink as WakeSink, Level};
use argus_worktrees::ActivityLine as TreeLine;
use serde_json::{Value, json};

async fn start() -> TestServer {
    TestServer::start(TestOptions::default()).await
}

async fn lines(t: &TestServer, query: &str) -> Vec<Value> {
    let reply = t.get(&format!("/api/activity{query}")).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    reply.json()["activity"].as_array().cloned().unwrap()
}

fn texts(rows: &[Value]) -> Vec<String> {
    rows.iter()
        .map(|r| r["text"].as_str().unwrap().to_owned())
        .collect()
}

fn shown(row: &Value) -> Value {
    json!([
        row["kind"],
        row["level"],
        row["text"],
        row["project"],
        row["thread"]
    ])
}

#[tokio::test(flavor = "multi_thread")]
async fn the_dashboard_start_is_a_line_newest_first_with_filters_and_a_bad_level() {
    let t = start().await;
    let all = lines(&t, "?limit=500").await;
    let started = all.iter().any(|a| {
        let text = a["text"].as_str().unwrap();
        a["kind"] == "dashboard" && text.starts_with("dashboard started (pid ")
    });
    assert!(started, "{all:?}");
    let ns: Vec<i64> = all.iter().map(|a| a["n"].as_i64().unwrap()).collect();
    assert!(ns.windows(2).all(|w| w[0] > w[1]));
    t.svc.activity.add(argus_store::NewActivity::new(
        "k",
        argus_store::ActivityLevel::Warn,
        "careful",
    ));
    let warns = lines(&t, "?level=warn").await;
    assert!(warns.iter().all(|a| a["level"] == "warn"));
    assert_eq!(texts(&warns), ["careful"]);
    assert_eq!(texts(&lines(&t, "?kind=k&limit=1").await), ["careful"]);
    let bad = t.get("/api/activity?level=loud").await;
    assert_eq!((bad.status, bad.code().as_str()), (400, "bad_request"));
    assert_eq!(lines(&t, "?limit=0").await.len(), 1);
    assert_eq!(lines(&t, "?limit=lots").await.len(), all.len() + 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_docs_of_the_portfolio_scope_follow_the_log_and_come_back_after_a_hard_resync() {
    let t = start().await;
    let rows = lines(&t, "").await;
    let docs = t
        .svc
        .db
        .docs()
        .list(ACTIVITY_SCOPE, Coll::Activity)
        .unwrap();
    assert!(docs.len() >= rows.len());
    let page = t.get("/api/p/_portfolio/db/activity").await;
    assert_eq!(page.status, 200);
    t.svc.db.docs().clear(ACTIVITY_SCOPE).unwrap();
    let wiped = t
        .svc
        .db
        .docs()
        .list(ACTIVITY_SCOPE, Coll::Activity)
        .unwrap();
    assert_eq!(wiped.len(), 0);
    t.svc.bus.emit_reset(&ResetEvent {
        scope: ACTIVITY_SCOPE.into(),
        reason: "resync".into(),
    });
    let again = t
        .svc
        .db
        .docs()
        .list(ACTIVITY_SCOPE, Coll::Activity)
        .unwrap();
    assert_eq!(again.len(), docs.len());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_reset_of_another_scope_leaves_the_activity_docs_alone() {
    let t = start().await;
    let before = t.svc.db.docs().seq().unwrap();
    t.svc.bus.emit_reset(&ResetEvent {
        scope: "alpha".into(),
        reason: "resync".into(),
    });
    assert_eq!(t.svc.db.docs().seq().unwrap(), before);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_wake_and_agent_and_notify_sinks_write_their_lines() {
    let t = start().await;
    let adapter = ActivityAdapter::of(&t.svc);
    WakeSink::add(
        &adapter,
        WakeLine {
            kind: "wake-fault".into(),
            level: Level::Warn,
            text: "rotp/lead: its wake listener is not running".into(),
            project: Some("rotp".into()),
            thread: Some("lead".into()),
        },
    );
    AgentSink::add(
        &adapter,
        AgentLine {
            kind: "agent".into(),
            level: "info".into(),
            text: "reaper closed rotp/A-1".into(),
            project: "rotp".into(),
            thread: None,
        },
    );
    NoteSink::note(
        &adapter,
        NoteLine {
            level: NoteLevel::Warn,
            text: "lead: Done (toast failed)".into(),
            project: Some("rotp".into()),
            thread: Some("lead".into()),
        },
    );
    let rows = lines(&t, "?limit=3").await;
    let got: Vec<Value> = rows.iter().rev().map(shown).collect();
    let want = [
        json!([
            "wake-fault",
            "warn",
            "rotp/lead: its wake listener is not running",
            "rotp",
            "lead"
        ]),
        json!(["agent", "info", "reaper closed rotp/A-1", "rotp", null]),
        json!(["note", "warn", "lead: Done (toast failed)", "rotp", "lead"]),
    ];
    assert_eq!(got, want);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_process_worktree_and_launcher_lines_carry_their_kind() {
    let t = start().await;
    let adapter = ActivityAdapter::of(&t.svc);
    adapter.procs_line(&ProcLine {
        kind: "process-kill",
        level: ProcLevel::Error,
        text: "could not kill node 12".into(),
    });
    adapter.worktree_line(TreeLine {
        kind: "worktree-remove",
        level: "warn",
        text: "rotp: removal refused".into(),
        project: "rotp".into(),
        thread: Some("ui".into()),
    });
    adapter.stale_note(StaleNote {
        level: "info",
        text: "rotp/ui: 1 launcher not used".into(),
        project: "rotp".into(),
        thread: Some("ui".into()),
    });
    let rows = lines(&t, "?limit=3").await;
    let got: Vec<Value> = rows.iter().rev().map(shown).collect();
    let want = [
        json!([
            "process-kill",
            "error",
            "could not kill node 12",
            null,
            null
        ]),
        json!([
            "worktree-remove",
            "warn",
            "rotp: removal refused",
            "rotp",
            "ui"
        ]),
        json!([
            "launcher-stale",
            "info",
            "rotp/ui: 1 launcher not used",
            "rotp",
            "ui"
        ]),
    ];
    assert_eq!(got, want);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_service_warning_reaches_the_server_log_and_the_log_once_a_minute() {
    let t = start().await;
    let adapter = ActivityAdapter::of(&t.svc);
    adapter.warn("procs", "process sample failed", "access denied");
    adapter.warn("procs", "process sample failed", "access denied");
    let rows = lines(&t, "?kind=procs").await;
    assert_eq!(texts(&rows), ["process sample failed: access denied"]);
    assert_eq!(rows[0]["level"], "warn");
    let log = std::fs::read_to_string(t.svc.log.file()).unwrap();
    assert_eq!(log.matches("process sample failed").count(), 2);
}
