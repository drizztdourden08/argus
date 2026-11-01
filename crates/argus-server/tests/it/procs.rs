//! The process monitor end to end over HTTP, with a fake machine: the listing, the rules, the
//! kills (the page and the Overlord may write, any other CLI caller may not) and the activity
//! lines they leave. Ported from the HTTP cases of `procs.http.test.mjs`; the `coord proc`
//! verbs stay in the Node `coord` tests.

use crate::machine::{Machine, row};
use argus_core::Coll;
use argus_core::clock::{iso_from_ms, now_ms};
use argus_core::ids::PORTFOLIO;
use argus_testkit::{Reply, Request, TestOptions, TestServer, beta_board, write_board};
use serde_json::{Value, json};
use std::sync::Arc;

const MIN: i64 = 60_000;

struct Rig {
    t: TestServer,
    machine: Arc<Machine>,
    _home: tempfile::TempDir,
}

fn overlord_board() -> Value {
    let mut b = beta_board();
    b["project"] = json!("portfolio");
    b["threads"]["api"]["subject"] = json!("helper");
    b
}

/// A server over the fake machine and the dashboard home `home`, with the board of project `project` written.
async fn rig_on(
    home: tempfile::TempDir,
    sync: bool,
    config: Value,
    project: &str,
    board: &Value,
) -> Rig {
    let machine = Arc::new(Machine::with_self());
    let wired = Arc::clone(&machine);
    let t = TestServer::start(TestOptions {
        sync,
        config,
        dashboard_home: Some(home.path().to_path_buf()),
        tweak: Some(Arc::new(move |opts| crate::machine::install(&wired, opts))),
        ..TestOptions::default()
    })
    .await;
    write_board(&t.coord_home, project, board);
    Rig {
        t,
        machine,
        _home: home,
    }
}

async fn rig_with(sync: bool) -> Rig {
    let home = tempfile::tempdir().expect("dashboard home");
    let rules = json!({ "rules": [{ "name": "test-kids", "match": { "cmd": "procs-http-test" }, "killable": true, "maxMemMB": 1000 }] });
    std::fs::write(home.path().join("process-rules.json"), rules.to_string()).expect("rules file");
    rig_on(home, sync, json!({}), "portfolio", &overlord_board()).await
}

async fn rig() -> Rig {
    rig_with(false).await
}

async fn post_as_page(t: &TestServer, path: &str, body: &Value) -> Reply {
    t.send(t.page(Request::new("POST", path).json(body))).await
}

async fn post(t: &TestServer, path: &str, body: &Value) -> Reply {
    t.write("POST", path, body).await
}

async fn procs(t: &TestServer) -> Value {
    t.get("/api/procs").await.json()
}

async fn activity_texts(t: &TestServer, kind: &str) -> Vec<String> {
    let rows = t.get(&format!("/api/activity?kind={kind}")).await.json();
    let rows = rows["activity"].as_array().cloned().unwrap_or_default();
    rows.iter()
        .map(|r| r["text"].as_str().unwrap_or_default().to_owned())
        .collect()
}

fn pids(list: &Value) -> Vec<i64> {
    let rows = list.as_array().cloned().unwrap_or_default();
    rows.iter().filter_map(|p| p["pid"].as_i64()).collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn nothing_before_a_sample_then_the_listing_the_biggest_first_with_this_process_protected() {
    let r = rig().await;
    let empty = procs(&r.t).await;
    assert_eq!(empty["meta"], Value::Null);
    assert_eq!(empty["procs"], json!([]));
    let a = r.machine.add(row(41_001, "a", 300, now_ms() - 30 * MIN));
    let sampled = post(&r.t, "/api/procs/sample", &json!({})).await;
    assert_eq!(sampled.json()["meta"]["count"], 2);
    let out = procs(&r.t).await;
    let own = i64::from(std::process::id());
    assert_eq!(
        pids(&out["procs"]),
        [i64::from(a), own],
        "the biggest first"
    );
    let me = &out["procs"][1];
    assert_eq!(
        (&me["protected"], &me["protectedWhy"], &me["killable"]),
        (&json!(true), &json!("dashboard"), &json!(false))
    );
    assert_eq!(out["procs"][0]["rule"], "test-kids");
    let none = r.t.get("/api/procs?rule=nope").await.json();
    assert_eq!(none["procs"], json!([]));
    let id = out["procs"][0]["id"].as_str().unwrap().to_owned();
    let doc = r.t.svc.db.docs().get(PORTFOLIO, Coll::Procs, &id).unwrap();
    assert!(doc.is_some());
    let meta =
        r.t.svc
            .db
            .docs()
            .get(PORTFOLIO, Coll::Meta, "procs")
            .unwrap();
    assert_eq!(meta.unwrap().data["count"], 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_portfolio_export_keeps_the_meta_doc_of_the_monitor() {
    let r = rig_with(true).await;
    r.machine.add(row(41_002, "a", 100, now_ms() - 30 * MIN));
    post(&r.t, "/api/procs/sample", &json!({})).await;
    let sync = r.t.svc.sync.clone().expect("sync is on");
    sync.resync(Some(PORTFOLIO), false).await.expect("resync");
    let meta =
        r.t.svc
            .db
            .docs()
            .get(PORTFOLIO, Coll::Meta, "procs")
            .unwrap();
    assert_eq!(
        meta.expect("kept by the portfolio exporter").data["count"],
        2
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_rules_and_the_names_that_are_always_protected() {
    let r = rig().await;
    let rules = r.t.get("/api/procs/rules").await.json();
    let names: Vec<&str> = rules["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x["name"].as_str())
        .collect();
    assert_eq!(names, ["test-kids"]);
    let protected = rules["protectedNames"].as_array().unwrap();
    assert!(protected.contains(&json!("svchost")));
}

#[tokio::test(flavor = "multi_thread")]
async fn the_page_and_the_overlord_may_change_rules_and_no_other_caller() {
    let r = rig().await;
    let t = &r.t;
    let body =
        json!({ "rule": { "name": "extra", "match": { "name": "^python" }, "maxMemMB": 900 } });
    let path = "/api/procs/rules";
    let by = |who: &str| json!({ "rule": body["rule"], "by": who });
    assert_eq!(
        post(t, path, &body).await.status,
        403,
        "a CLI caller without by"
    );
    assert_eq!(
        post(t, path, &by("api")).await.status,
        403,
        "not the coordinator"
    );
    assert_eq!(
        post(t, path, &by("lead")).await.status,
        403,
        "no such thread"
    );
    let page = post_as_page(t, path, &body).await;
    assert_eq!(page.status, 200, "{}", page.text());
    assert_eq!(page.json()["created"], true);
    let idle = json!({ "rule": { "name": "extra", "idleMin": 30 }, "by": "coordinator" });
    let cli = post(t, path, &idle).await;
    assert_eq!(cli.status, 200, "{}", cli.text());
    let want =
        json!({ "name": "extra", "match": { "name": "^python" }, "maxMemMB": 900, "idleMin": 30 });
    assert_eq!(cli.json()["rule"], want);
    let regex =
        json!({ "rule": { "name": "extra", "match": { "cmd": "(" } }, "by": "coordinator" });
    assert_eq!(post(t, path, &regex).await.status, 400);
    assert_eq!(post(t, path, &json!({})).await.status, 400);
    let gone = json!({ "name": "extra", "by": "coordinator" });
    assert_eq!(post(t, "/api/procs/rules/remove", &gone).await.status, 200);
    assert_eq!(post(t, "/api/procs/rules/remove", &gone).await.status, 404);
    let acts = activity_texts(t, "process-rule").await;
    assert_eq!(
        acts,
        [
            "rule extra removed by coordinator",
            "rule extra changed by coordinator",
            "rule extra added from the page"
        ]
    );
}

fn started_of(list: &Value, pid: u32) -> String {
    let rows = list["procs"].as_array().unwrap();
    let doc = rows.iter().find(|p| p["pid"] == pid).expect("listed");
    doc["startedAt"].as_str().unwrap().to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_kill_refuses_what_is_not_listed_reused_protected_or_not_asked_by_the_overlord() {
    let r = rig().await;
    let t = &r.t;
    let start = now_ms() - 30 * MIN;
    let b = r.machine.add(row(41_003, "b", 50, start + 1000));
    post(t, "/api/procs/sample", &json!({})).await;
    let kill = "/api/procs/kill";
    let stale = json!({ "pid": 999_999, "startedAt": iso_from_ms(start) });
    assert_eq!(post_as_page(t, kill, &stale).await.status, 404);
    let reused = json!({ "pid": b, "startedAt": iso_from_ms(start) });
    assert_eq!(post_as_page(t, kill, &reused).await.status, 409);
    let own = std::process::id();
    let listed = procs(t).await;
    let mine = json!({ "pid": own, "startedAt": started_of(&listed, own) });
    assert_eq!(post_as_page(t, kill, &mine).await.status, 403);
    let ok_start = json!({ "pid": b, "startedAt": started_of(&listed, b) });
    assert_eq!(
        post(t, kill, &ok_start).await.status,
        403,
        "a CLI caller that is not the Overlord"
    );
    assert!(r.machine.alive(b));
    let none = post_as_page(t, kill, &json!({ "startedAt": iso_from_ms(start) })).await;
    assert_eq!(none.status, 400);
    let no_time = post_as_page(t, kill, &json!({ "pid": b })).await;
    assert_eq!(no_time.status, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_kill_from_the_page_and_one_from_the_overlord_end_the_process_and_say_so() {
    let r = rig().await;
    let t = &r.t;
    let start = now_ms() - 30 * MIN;
    let b = r.machine.add(row(41_004, "b", 50, start + 1000));
    let e = r.machine.add(row(41_005, "e", 70, start + 2000));
    post(t, "/api/procs/sample", &json!({})).await;
    let listed = procs(t).await;
    let reason = "left over by a test";
    let body = json!({ "pid": b, "startedAt": started_of(&listed, b), "reason": reason });
    let done = post_as_page(t, "/api/procs/kill", &body).await;
    assert_eq!(done.status, 200, "{}", done.text());
    let want = json!({ "killed": true, "pid": b, "name": "node.exe", "memMB": 50 });
    assert_eq!(done.json(), want);
    assert!(!r.machine.alive(b));
    let by = json!({ "pid": e, "startedAt": started_of(&listed, e), "reason": "a test child", "by": "coordinator" });
    let cli = post(t, "/api/procs/kill", &by).await;
    assert_eq!(cli.status, 200, "{}", cli.text());
    let texts = activity_texts(t, "process-kill").await;
    let page_line = format!("killed node {b} (procs-http-test, 50 MB) from the page: {reason}");
    let cli_line = format!(
        "killed node {e} (procs-http-test, 70 MB) from the CLI (coordinator): a test child"
    );
    assert!(texts.contains(&page_line), "{texts:?}");
    assert!(texts.contains(&cli_line), "{texts:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_refused_kill_is_a_warning_line_with_its_reason() {
    let r = rig().await;
    let t = &r.t;
    let stale = json!({ "pid": 999_999, "startedAt": iso_from_ms(now_ms()) });
    assert_eq!(post_as_page(t, "/api/procs/kill", &stale).await.status, 404);
    assert_eq!(post(t, "/api/procs/kill", &stale).await.status, 403);
    let texts = activity_texts(t, "process-kill").await;
    let listing = texts.iter().any(|x| {
        x.starts_with("refused to kill 999999 from the page: process 999999 is not in the listing")
    });
    assert!(listing, "{texts:?}");
    let cli = texts
        .iter()
        .any(|x| x.starts_with("refused to kill a process from the CLI: only the Overlord"));
    assert!(cli, "{texts:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn kill_rule_ends_the_old_ones_and_skips_the_young() {
    let r = rig().await;
    let t = &r.t;
    let c = r.machine.add(row(41_006, "c", 50, now_ms() - 300 * MIN));
    let d = r.machine.add(row(41_007, "d", 50, now_ms() - MIN));
    let path = "/api/procs/kill-rule";
    let done = post_as_page(
        t,
        path,
        &json!({ "rule": "test-kids", "olderThanMin": 120 }),
    )
    .await;
    assert_eq!(done.status, 200, "{}", done.text());
    let out = done.json();
    assert_eq!(pids(&out["killed"]), [i64::from(c)]);
    let skipped = out["skipped"].as_array().unwrap();
    let young = skipped
        .iter()
        .any(|s| s["pid"] == d && s["why"].as_str().unwrap().contains("younger"));
    assert!(young, "{skipped:?}");
    assert!(!r.machine.alive(c));
    assert!(r.machine.alive(d));
    assert_eq!(
        post_as_page(t, path, &json!({ "rule": "nope" }))
            .await
            .status,
        404
    );
    assert_eq!(post_as_page(t, path, &json!({})).await.status, 400);
    let bad = json!({ "rule": "test-kids", "olderThanMin": -1 });
    assert_eq!(post_as_page(t, path, &bad).await.status, 400);
    let cli = post(t, path, &json!({ "rule": "test-kids" })).await;
    assert_eq!(cli.status, 403);
}

#[tokio::test(flavor = "multi_thread")]
async fn without_the_monitor_its_routes_answer_503() {
    let t = TestServer::start(TestOptions::default()).await;
    for path in ["/api/procs", "/api/procs/rules"] {
        let reply = t.get(path).await;
        assert_eq!(
            (reply.status, reply.code().as_str()),
            (503, "unavailable"),
            "{path}"
        );
    }
    assert_eq!(post(&t, "/api/procs/sample", &json!({})).await.status, 503);
}

/// A rig whose project `beta` has its root at `/work/beta`, and the beta board.
async fn own_rig() -> Rig {
    let home = tempfile::tempdir().expect("dashboard home");
    let config = json!({ "projectRoots": { "beta": "/work/beta" } });
    rig_on(home, false, config, "beta", &beta_board()).await
}

fn message(reply: &Reply) -> String {
    reply.json()["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_thread_kills_its_own_process_through_its_project_route_and_nothing_else() {
    let r = own_rig().await;
    let t = &r.t;
    let start = now_ms() - MIN;
    let mine = r.machine.add(row(42_001, "/work/beta/run.js", 40, start));
    let other = r.machine.add(row(42_002, "/work/gamma/run.js", 40, start));
    let by = json!({ "by": "api", "reason": "its run hung" });
    let path = |pid: u32| format!("/api/p/beta/procs/{pid}/kill");
    let ghost = post(t, &path(mine), &json!({ "by": "ghost" })).await;
    assert_eq!(ghost.status, 403);
    assert!(message(&ghost).starts_with("only a live thread of project beta"));
    let outside = post(t, &path(other), &by).await;
    assert_eq!(outside.status, 403);
    let said = message(&outside);
    assert!(
        said.contains("is not inside a folder of project beta"),
        "{said}"
    );
    let server = post(t, &path(std::process::id()), &by).await;
    assert_eq!(server.status, 403);
    assert!(message(&server).contains("is a process of the dashboard itself"));
    assert_eq!(post(t, "/api/p/beta/procs/abc/kill", &by).await.status, 400);
    let done = post(t, &path(mine), &by).await;
    assert_eq!(done.status, 200, "{}", done.json());
    assert_eq!(done.json()["killed"], json!(true));
    assert!(!r.machine.alive(mine) && r.machine.alive(other));
    let texts = activity_texts(t, "proc-kill").await;
    let killed =
        format!("thread api of beta killed C:\\nvm4w\\nodejs\\node.exe {mine}: its run hung");
    assert!(texts.contains(&killed), "{texts:?}");
    let refused = texts
        .iter()
        .any(|x| x.starts_with("refused a process kill for project beta from ghost"));
    assert!(refused, "{texts:?}");
}
