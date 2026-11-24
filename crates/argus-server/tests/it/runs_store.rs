//! The run rows in the dashboard database: what a restart does to rows a crash left, retention,
//! the log cap, the wake link a failed run tells, and the launcher wakes over the boards.
//! K3, K1: the run rows in the database over a fake process host: crash recovery, retention, the log cap and the wakes.
// test-tag: standard

use crate::runs_rig::{Rig, until, wakes_of};
use argus_core::Paths;
use argus_store::rusqlite::{Connection, params};
use argus_testkit::{TestOptions, TestServer, alpha_board, write_board};
use serde_json::{Value, json};

/// A row an earlier server left, with the status it died in.
fn leave(db: &std::path::Path, rid: &str, status: &str, pid: Option<i64>) {
    let conn = Connection::open(db).expect("the database opens");
    conn.execute(
        "INSERT INTO runs (id, scope, rid, name, kind, status, pid, cmd, cwd, origin, \"by\", queued_at, instance) VALUES (?, 'alpha', ?, 'job', 'task', ?, ?, 'fake job', '.', 'cli', 'cli', '2026-01-01T00:00:00.000Z', 'old')",
        params![format!("alpha:{rid}"), rid, status, pid],
    )
    .expect("a row is left");
    conn.execute(
        "INSERT OR REPLACE INTO kv (key, value) VALUES ('run:alpha', '2')",
        [],
    )
    .expect("a counter");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_restart_fails_what_a_crash_left_marks_the_orphan_rebuilds_the_docs_and_keeps_counting() {
    let first = Rig::start().await;
    first.define(&[("job", json!({}))]);
    let (coord_home, dashboard_home) = first.t.stop(true).await;
    let db = Paths::new(&coord_home, Some(dashboard_home.clone())).db;
    leave(&db, "R-1", "running", Some(4242));
    leave(&db, "R-2", "queued", None);
    let r = Rig::with(TestOptions {
        coord_home: Some(coord_home),
        dashboard_home: Some(dashboard_home),
        ..TestOptions::default()
    })
    .await;
    let one = r.run("R-1").await;
    assert_eq!(
        (&one["status"], &one["orphan"], &one["stdin"]),
        (&json!("failed"), &json!(true), &json!(false))
    );
    assert!(
        one["error"]
            .as_str()
            .unwrap()
            .starts_with("server restarted while running")
    );
    let two = r.run("R-2").await;
    assert_eq!(
        (&two["status"], &two["error"]),
        (&json!("failed"), &json!("server restarted before start"))
    );
    let docs = r.t.get("/api/p/alpha/db/runs").await.json();
    assert_eq!(
        docs["docs"].as_array().unwrap().len(),
        2,
        "the run docs are rebuilt"
    );
    let fresh = r.begin("job").await;
    assert_eq!(fresh["id"], "R-3", "the counter continues");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn retention_keeps_the_newest_finished_runs_and_drops_the_rest_with_their_logs_and_docs() {
    let config = json!({ "runs": { "keepPerDef": 2 } });
    let r = Rig::with(TestOptions {
        config,
        ..TestOptions::default()
    })
    .await;
    r.define(&[("job", json!({}))]);
    for n in 1..=4 {
        let pid = u32::try_from(r.begin("job").await["pid"].as_u64().unwrap()).unwrap();
        r.host.say(pid, "work\n").await;
        r.host.exit(pid, Some(0)).await;
        r.until_status(&format!("R-{n}"), "exited").await;
    }
    until("the oldest runs to go", || async {
        (r.run("R-1").await.is_null()).then_some(())
    })
    .await;
    assert!(r.run("R-2").await.is_null());
    let kept = r.t.get("/api/p/alpha/runs").await.json();
    let rids: Vec<&str> = kept["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x["id"].as_str())
        .collect();
    assert_eq!(rids, ["R-4", "R-3"], "newest first");
    let lines = r.t.svc.db.with_conn(|c| {
        c.query_row(
            "SELECT COUNT(*) FROM run_logs WHERE run_id IN ('alpha:R-1', 'alpha:R-2')",
            [],
            |row| row.get::<_, i64>(0),
        )
    });
    assert_eq!(lines.unwrap(), 0, "their log lines went with them");
    let docs = r.t.get("/api/p/alpha/db/runs").await.json();
    assert_eq!(docs["docs"].as_array().unwrap().len(), 2);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_log_cap_drops_the_middle_keeps_the_head_and_counts_what_it_dropped() {
    let config = json!({ "runs": { "logCapBytes": 600, "logHeadBytes": 100 } });
    let r = Rig::with(TestOptions {
        config,
        ..TestOptions::default()
    })
    .await;
    r.define(&[("noisy", json!({}))]);
    let pid = u32::try_from(r.begin("noisy").await["pid"].as_u64().unwrap()).unwrap();
    for n in 0..60 {
        r.host.say(pid, &format!("line number {n:02}\n")).await;
    }
    r.host.exit(pid, Some(0)).await;
    let run = r.until_status("R-1", "exited").await;
    assert!(run["logDropped"].as_u64().unwrap() > 0, "{run}");
    assert!(run["logBytes"].as_u64().unwrap() <= 600, "{run}");
    let texts = r.texts("R-1").await;
    assert!(
        texts[0].starts_with("started by"),
        "the head is never dropped: {texts:?}"
    );
    assert!(
        texts.iter().any(|t| t == "line number 59"),
        "the newest output stays"
    );
    assert!(
        !texts.iter().any(|t| t == "line number 20"),
        "the middle went"
    );
    let page = r.t.get("/api/p/alpha/runs/R-1/logs?tail=3").await.json();
    assert_eq!(page["dropped"], run["logDropped"]);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_run_that_fails_wakes_the_lead_of_its_project() {
    let r = Rig::served(TestOptions {
        seed: true,
        ..TestOptions::default()
    })
    .await;
    let pid = u32::try_from(r.begin("fail").await["pid"].as_u64().unwrap()).unwrap();
    r.host.say(pid, "it broke\n").await;
    r.host.exit(pid, Some(3)).await;
    r.until_status("R-1", "failed").await;
    let wake = until("the wake", || async {
        wakes_of(&r.t, "alpha", "run-failed").await.pop()
    })
    .await;
    assert_eq!(wake["thread"], "lead");
    let text = wake["text"].as_str().unwrap();
    assert!(text.starts_with("R-1 fail failed (exit 3)"), "{text}");
    let stopped = r.begin("slow").await;
    r.cli("POST", "/api/p/alpha/runs/R-2/stop", &json!({}))
        .await;
    assert_eq!(stopped["id"], "R-2");
    r.until_status("R-2", "stopped").await;
    let pid = u32::try_from(r.begin("fail").await["pid"].as_u64().unwrap()).unwrap();
    r.host.exit(pid, Some(3)).await;
    let rows = until("the second failure wake", || async {
        let rows = wakes_of(&r.t, "alpha", "run-failed").await;
        (rows.len() >= 2).then_some(rows)
    })
    .await;
    let texts: Vec<&str> = rows.iter().filter_map(|w| w["text"].as_str()).collect();
    assert!(
        texts.iter().all(|t| !t.starts_with("R-2")),
        "a stop is not a failure to act on: {texts:?}"
    );
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_launcher_wakes_tell_the_lead_of_a_project_with_no_master_launcher() {
    let t = TestServer::start(TestOptions {
        seed: true,
        ..TestOptions::default()
    })
    .await;
    let mut board: Value = alpha_board(&t.coord_home);
    board["launchers"] = json!({});
    write_board(&t.coord_home, "alpha", &board);
    t.write("POST", "/api/admin/resync", &json!({ "project": "alpha" }))
        .await;
    let stale = t
        .svc
        .launcher_stale
        .clone()
        .expect("the launcher wakes are on");
    assert!(stale.tick() >= 1);
    let wakes = wakes_of(&t, "alpha", "launcher-master-missing").await;
    assert_eq!(
        (wakes[0]["thread"].clone(), wakes[0]["subject"].clone()),
        (json!("lead"), json!("alpha"))
    );
    let activity = t.get("/api/activity?kind=launcher-stale").await.json();
    assert!(
        activity["activity"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["text"].as_str().unwrap().contains("no master launcher"))
    );
    t.stop(false).await;
}
