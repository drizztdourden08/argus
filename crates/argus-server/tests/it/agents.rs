//! The sub-agent services inside a real server: the reconciler runs with the services and
//! `GET /api/agents/reconcile` reports it (ported from `agents.reconciler.http.test.mjs`; the
//! `coord agent reconcile` verb stays with the `coord` port of slice 8), the reaper closes a
//! quiet agent through the sync service and the closed agent reaches the page doc (the last
//! case of `agents.reaper.test.mjs`), and the watcher's live view joins the agents docs.
//! K2, S2: the reconcile report and the reaper through the sync service.
// test-tag: standard

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::wake_rig::{pause, wake_where};
use argus_agents::fakes::{agent_record, assistant_line, text_block, touch};
use argus_agents::{AgentReaper, Ports, ReaperOptions, StdFs, SystemClock};
use argus_core::Coll;
use argus_core::clock::{iso_from_ms, now_ms};
use argus_server::adapters::agents::AgentBoards;
use argus_testkit::{TestOptions, TestServer, alpha_board, make_home, write_board};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::runtime::Handle;

const MIN: i64 = 60_000;
const SID: &str = "sess-0000-aaaa";

struct Rig {
    t: TestServer,
    home: TempDir,
    _claude: TempDir,
}

/// What a test changes about the server it starts.
struct Setup {
    reaper: bool,
    reconciler: bool,
}

/// A server whose alpha board holds `agents` (and no others), a coordinator that owns the
/// session `SID`, and a Claude home with that session's folder.
async fn rig(agents: &[Value], setup: Setup) -> Rig {
    let home = tempfile::tempdir().expect("coord home");
    let claude = tempfile::tempdir().expect("claude home");
    make_home(home.path());
    std::fs::create_dir_all(claude.path().join("projects").join("X--proj").join(SID))
        .expect("session folder");
    let mut board = alpha_board(home.path());
    board["threads"]["coordinator"]["sessionId"] = json!(SID);
    board["threads"]["coordinator"]["root"] = json!("X:\\proj");
    board["agents"] = Value::Object(
        agents
            .iter()
            .map(|a| (a["id"].as_str().unwrap_or_default().to_owned(), a.clone()))
            .collect(),
    );
    write_board(home.path(), "alpha", &board);
    let config = json!({ "claudeHome": claude.path(), "claudeTemp": claude.path().join("tmp"), "reconcileMs": 3_600_000 });
    let t = TestServer::start(TestOptions {
        coord_home: Some(home.path().to_path_buf()),
        config: json!({ "agents": config }),
        tweak: Some(Arc::new(move |o| {
            o.agents.reaper = setup.reaper;
            o.agents.reconciler = setup.reconciler;
        })),
        ..TestOptions::default()
    })
    .await;
    Rig {
        t,
        home,
        _claude: claude,
    }
}

fn agent_doc(t: &TestServer, id: &str) -> Option<Value> {
    t.svc
        .db
        .docs()
        .get("alpha", Coll::Agents, id)
        .ok()?
        .map(|row| row.data)
}

/// Waits until a doc of the agents collection satisfies `check`.
async fn doc_where(t: &TestServer, id: &str, check: impl Fn(&Value) -> bool) -> Value {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(doc) = agent_doc(t, id).filter(&check) {
            return doc;
        }
        assert!(
            Instant::now() < end,
            "timed out waiting for the doc of {id}"
        );
        pause(60).await;
    }
}

/// The report of the first run of the reconciler.
async fn first_run(t: &TestServer) -> Value {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        let body = t.get("/api/agents/reconcile").await.json();
        if body["ran"] == true && body["projects"]["alpha"].is_object() {
            return body;
        }
        assert!(Instant::now() < end, "timed out waiting for the first run");
        pause(60).await;
    }
}

/// A transcript whose last write was `quiet_min` minutes ago.
fn transcript(dir: &Path, name: &str, quiet_min: i64, body: &str) -> String {
    let file = dir.join(format!("{name}.jsonl"));
    std::fs::write(&file, body).expect("transcript");
    touch(&file, now_ms() - quiet_min * MIN);
    file.display().to_string()
}

fn orphan() -> Value {
    let long = iso_from_ms(now_ms() - 3 * 60 * MIN);
    agent_record(
        "A-44",
        now_ms() - 2 * 60 * MIN,
        &json!({ "name": "controllers-opus", "task": "Rebuild Controllers", "model": "opus", "modelWhy": "owner asked", "startedAt": long, "checkinMin": 20 }),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn the_first_run_finds_the_orphan_flags_the_record_and_wakes_the_owning_thread() {
    let r = rig(
        &[orphan()],
        Setup {
            reaper: true,
            reconciler: true,
        },
    )
    .await;
    let t = &r.t;
    let run = first_run(t).await;
    assert_eq!(run["projects"]["alpha"]["orphan"], 1);
    assert_eq!(run["projects"]["alpha"]["woken"], 1);
    let findings = run["findings"].as_array().cloned().unwrap_or_default();
    let finding = findings
        .iter()
        .find(|x| x["id"] == "A-44")
        .expect("a finding");
    assert_eq!(finding["class"], "orphan");
    assert_eq!(finding["project"], "alpha");
    let record = t.board("alpha")["agents"]["A-44"].clone();
    assert!(record["orphanSince"].is_string() && record["orphanWokeAt"].is_string());
    let wake = wake_where(t, "alpha", |w| w["reason"] == "agent-orphan").await;
    assert_eq!(wake["thread"], "coordinator");
    let text = wake["text"].as_str().unwrap_or_default();
    assert!(text.starts_with("A-44 controllers-opus (opus) is on the board but Claude has no sub-agent or background task for it"), "{text}");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_project_query_narrows_the_report() {
    let r = rig(
        &[orphan()],
        Setup {
            reaper: true,
            reconciler: true,
        },
    )
    .await;
    let t = &r.t;
    first_run(t).await;
    let none = t.get("/api/agents/reconcile?project=beta").await.json();
    assert_eq!(none["projects"], json!({}));
    assert_eq!(none["findings"], json!([]));
    let alpha = t.get("/api/agents/reconcile?project=alpha").await.json();
    assert_eq!(alpha["findings"].as_array().map(Vec::len), Some(1));
    assert_eq!(alpha["projects"]["alpha"]["orphan"], 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_reconciler_switched_off_the_route_answers_503() {
    let r = rig(
        &[],
        Setup {
            reaper: true,
            reconciler: false,
        },
    )
    .await;
    let reply = r.t.get("/api/agents/reconcile").await;
    assert_eq!(reply.status, 503);
    assert_eq!(reply.code(), "unavailable");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_reaper_closes_a_quiet_agent_through_the_sync_service_and_the_doc_follows() {
    let r = rig(
        &[],
        Setup {
            reaper: false,
            reconciler: true,
        },
    )
    .await;
    let t = &r.t;
    let file = transcript(r.home.path(), "g91", 50, "{\"type\":\"user\"}\n");
    let quiet = agent_record(
        "A-91",
        now_ms(),
        &json!({ "source": "hook", "hookId": "h-91", "transcriptFile": file }),
    );
    let mut board = t.board("alpha");
    board["agents"]["A-91"] = quiet;
    write_board(&t.coord_home, "alpha", &board);
    t.write("POST", "/api/admin/resync", &json!({ "project": "alpha" }))
        .await;
    assert_eq!(agent_doc(t, "A-91").expect("the doc")["status"], "working");

    let boards = AgentBoards::new(
        t.svc.boards(),
        t.coord_home.clone(),
        t.svc.queue.clone(),
        Handle::current(),
    );
    let ports = Ports::new(Arc::new(boards), Arc::new(StdFs), Arc::new(SystemClock));
    let reaper = AgentReaper::new(ports, ReaperOptions::default());
    let closed = tokio::task::spawn_blocking(move || reaper.tick())
        .await
        .expect("the poll");
    assert_eq!(closed, ["alpha/A-91"]);
    let doc = doc_where(t, "A-91", |d| d["status"] == "done").await;
    let note = doc["note"].as_str().unwrap_or_default();
    assert!(
        note.starts_with("no activity for 5") && note.ends_with(" min, closed automatically"),
        "{note}"
    );
    assert_eq!(doc["reaped"], true);
    assert_eq!(doc["updatedBy"], "dashboard");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_reaper_of_the_server_closes_a_quiet_agent_at_the_first_poll() {
    let dir = tempfile::tempdir().expect("transcripts");
    let file = transcript(dir.path(), "h92", 50, "{\"type\":\"user\"}\n");
    let quiet = agent_record(
        "A-92",
        now_ms(),
        &json!({ "source": "hook", "hookId": "h-92", "transcriptFile": file }),
    );
    let r = rig(
        &[quiet],
        Setup {
            reaper: true,
            reconciler: false,
        },
    )
    .await;
    let doc = doc_where(&r.t, "A-92", |d| d["status"] == "done").await;
    assert_eq!(doc["reaped"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_live_view_of_a_transcript_joins_the_agents_doc() {
    let dir = tempfile::tempdir().expect("transcripts");
    let line = assistant_line(
        now_ms(),
        &json!([text_block("hello from the agent")]),
        &json!({}),
    );
    let file = transcript(dir.path(), "w93", 0, &line);
    let working = agent_record(
        "A-93",
        now_ms(),
        &json!({ "source": "hook", "hookId": "h-93", "transcriptFile": file }),
    );
    let r = rig(
        &[working],
        Setup {
            reaper: true,
            reconciler: false,
        },
    )
    .await;
    let doc = doc_where(&r.t, "A-93", |d| {
        d["live"]["lastMessage"] == "hello from the agent"
    })
    .await;
    assert_eq!(doc["live"]["turns"], 1);
    assert_eq!(doc["status"], "working");
}
