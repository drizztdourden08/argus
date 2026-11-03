//! A preview serves a copy of the boards: owner posts are accepted and write the board, while
//! Play and every run, a worktree removal and a process kill are refused, and a preview does
//! not start on the live boards.
//! K2, K3: preview mode accepts board writes, refuses every outward effect and refuses the live boards.
// test-tag: critical

use std::path::PathBuf;
use std::sync::Arc;

use crate::machine::{Machine, install, row};
use crate::posting::{cli, entry_count};
use crate::runs_rig::Rig;
use argus_core::Paths;
use argus_server::{ServeError, ServeOptions, serve};
use argus_testkit::{Request, TestOptions, TestServer, Tweak};
use serde_json::json;

fn preview_tweak() -> Tweak {
    Arc::new(|opts: &mut ServeOptions| opts.preview = true)
}

fn preview_options() -> TestOptions {
    TestOptions {
        seed: true,
        tweak: Some(preview_tweak()),
        ..TestOptions::default()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_preview_accepts_an_owner_post_and_writes_the_board() {
    let t = TestServer::start(preview_options()).await;
    assert!(t.svc.preview && t.svc.board_writes && t.svc.shadow);
    let before = entry_count(&t, "alpha");
    let followup = json!({ "kind": "followup", "replyTo": "Q-1", "text": "From the preview" });
    let posted = cli(&t, &followup).await;
    assert_eq!(posted.status, 200, "{}", posted.text());
    assert_eq!(posted.json()["status"], "applied");
    assert_eq!(entry_count(&t, "alpha"), before + 1);
    let health = t.get("/api/health").await.json();
    assert_eq!(
        (&health["preview"], &health["shadow"]),
        (&json!(true), &json!(false))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_preview_marks_its_home_so_a_start_without_the_flag_is_a_preview_too() {
    let t = TestServer::start(preview_options()).await;
    assert!(argus_server::preview::marked(
        &t.dashboard_home.join("config.json")
    ));
    let (coord_home, dashboard_home) = t.stop(true).await;
    let again = TestServer::start(TestOptions {
        coord_home: Some(coord_home),
        dashboard_home: Some(dashboard_home),
        ..TestOptions::default()
    })
    .await;
    assert!(
        again.svc.preview,
        "the marker in config.json turns preview mode on"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_preview_refuses_play_and_every_run() {
    let r = Rig::with(TestOptions {
        tweak: Some(preview_tweak()),
        ..TestOptions::default()
    })
    .await;
    r.define(&[("job", json!({}))]);
    for (path, body) in [
        ("/api/p/alpha/runs", json!({ "name": "job" })),
        ("/api/p/alpha/runs/R-1/stop", json!({})),
        ("/api/p/alpha/runs/R-1/stdin", json!({ "text": "x" })),
    ] {
        let refused = r.page("POST", path, &body).await;
        assert_eq!(
            (refused.status, refused.code().as_str()),
            (403, "read_only"),
            "{path}"
        );
    }
    assert!(r.host.requests().is_empty(), "no run starts");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_preview_refuses_a_worktree_removal() {
    let t = TestServer::start(preview_options()).await;
    for path in [
        "/api/p/alpha/worktrees/remove",
        "/api/p/alpha/worktrees/prune",
        "/api/p/alpha/worktrees/leftovers/remove",
    ] {
        let request = Request::new("POST", path).json(&json!({ "id": "w1" }));
        let refused = t.send(t.page(request)).await;
        assert_eq!(
            (refused.status, refused.code().as_str()),
            (403, "read_only"),
            "{path}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_preview_refuses_a_process_kill() {
    let machine = Arc::new(Machine::with_self());
    let pid = machine.add(row(41_050, "victim", 50, 1_000));
    let wired = Arc::clone(&machine);
    let t = TestServer::start(TestOptions {
        tweak: Some(Arc::new(move |opts: &mut ServeOptions| {
            install(&wired, opts);
            opts.preview = true;
        })),
        ..TestOptions::default()
    })
    .await;
    for (path, body) in [
        (
            "/api/procs/kill",
            json!({ "pid": pid, "startedAt": "2026-01-01T00:00:00.000Z" }),
        ),
        ("/api/procs/kill-rule", json!({ "rule": "any" })),
    ] {
        let refused = t.send(t.page(Request::new("POST", path).json(&body))).await;
        assert_eq!(
            (refused.status, refused.code().as_str()),
            (403, "read_only"),
            "{path}"
        );
    }
    assert!(machine.alive(pid), "nothing was killed");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_preview_does_not_start_on_the_live_boards_and_writes_nothing_there() {
    let live = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let mut opts = ServeOptions::new(Paths::new(live.path(), Some(home.path().join("dash"))));
    opts.preview = true;
    opts.port = Some(0);
    opts.live_coord_home = Some(PathBuf::from(live.path()));
    let refused = serve(opts).await.err().expect("the start is refused");
    assert!(
        matches!(refused, ServeError::PreviewRefused(_)),
        "{refused}"
    );
    assert!(refused.to_string().contains("live boards"), "{refused}");
    assert!(
        !home.path().join("dash").exists(),
        "no folder, no marker, no database"
    );
}
