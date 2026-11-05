//! Notifications to the owner end to end over HTTP: a server with a toaster that shows nothing,
//! the routes (a ping, the page's test button, the listing, the settings, the refusals), the
//! limits, the automatic pings from board writes (told once, across a restart), the activity
//! lines and the tray balloon. Ported from the HTTP cases of `notify.http.test.mjs`; the cases
//! that drive the Node `coord` and the MCP tool stay with their own slices.
//! K1, K2, K4: the notification routes, limits and automatic pings, with a manual clock and a worker that takes exports in order.
// test-tag: standard

use std::path::Path;
use std::sync::{Arc, PoisonError};

use crate::toasts::Toasts;
use crate::wait::until;
use argus_core::Coll;
use argus_core::clock::{ManualClock, now_ms};
use argus_core::fixture::test_doc;
use argus_core::ids::PORTFOLIO;
use argus_native::{Level, Notifier, TrayFiles};
use argus_server::ServeOptions;
use argus_server::adapters::notify::TrayBalloon;
use argus_testkit::{
    Reply, Request, TestOptions, TestServer, ago, beta_board, make_home, write_board,
};
use serde_json::{Value, json};
use tempfile::TempDir;

struct Rig {
    t: TestServer,
    toasts: Arc<Toasts>,
    clock: ManualClock,
    coord: TempDir,
    dash: TempDir,
}

fn overlord_board() -> Value {
    let mut board = beta_board();
    board["project"] = json!("portfolio");
    board
}

async fn start(
    (coord, dash): (&Path, &Path),
    toasts: &Arc<Toasts>,
    clock: &ManualClock,
    balloon: bool,
) -> TestServer {
    let toasts: Arc<dyn Notifier> = toasts.clone();
    let clock = clock.clone();
    let files = TrayFiles::new(dash);
    TestServer::start(TestOptions {
        coord_home: Some(coord.to_path_buf()),
        dashboard_home: Some(dash.to_path_buf()),
        tweak: Some(Arc::new(move |opts: &mut ServeOptions| {
            opts.notify.notifier = Some(Arc::clone(&toasts));
            opts.notify.clock = Some(Arc::new(clock.clone()));
            if balloon {
                opts.notify.fallback = Some(Arc::new(TrayBalloon::new(files.clone())));
            }
        })),
        ..TestOptions::default()
    })
    .await
}

async fn rig_with(balloon: bool) -> Rig {
    let coord = tempfile::tempdir().expect("coord home");
    make_home(coord.path());
    write_board(coord.path(), "portfolio", &overlord_board());
    let dash = tempfile::tempdir().expect("dashboard home");
    let toasts = Arc::new(Toasts::default());
    let clock = ManualClock::at(now_ms());
    let homes = (coord.path(), dash.path());
    let t = start(homes, &toasts, &clock, balloon).await;
    Rig {
        t,
        toasts,
        clock,
        coord,
        dash,
    }
}

async fn rig() -> Rig {
    rig_with(false).await
}

async fn restart(rig: Rig) -> Rig {
    let Rig {
        t,
        toasts,
        clock,
        coord,
        dash,
    } = rig;
    t.stop(false).await;
    let t = start((coord.path(), dash.path()), &toasts, &clock, false).await;
    Rig {
        t,
        toasts,
        clock,
        coord,
        dash,
    }
}

async fn ping(t: &TestServer, body: &Value) -> Reply {
    t.write("POST", "/api/notify", body).await
}

async fn as_page(t: &TestServer, path: &str, body: &Value) -> Reply {
    t.send(t.page(Request::new("POST", path).json(body))).await
}

async fn notes(t: &TestServer) -> Vec<Value> {
    let found = t.get("/api/notify").await.json();
    found["notifications"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

async fn activity(t: &TestServer, query: &str) -> Vec<Value> {
    let found = t.get(&format!("/api/activity?{query}")).await.json();
    found["activity"].as_array().cloned().unwrap_or_default()
}

async fn texts(t: &TestServer, query: &str) -> Vec<String> {
    let rows = activity(t, query).await;
    rows.iter()
        .map(|r| r["text"].as_str().unwrap_or_default().to_owned())
        .collect()
}

fn count_of(list: &[Value], word: &str) -> usize {
    list.iter()
        .filter(|n| n["by"] == "auto" && n["text"].as_str().is_some_and(|t| t.contains(word)))
        .count()
}

/// Waits for the worker that tells the owner of an export.
async fn told(t: &TestServer, word: &str, count: usize) -> Vec<Value> {
    until("the owner to be told", || async {
        let list = notes(t).await;
        (count_of(&list, word) >= count).then_some(list)
    })
    .await
}

/// Waits until the worker has taken the first export of the server: its markers are saved.
async fn first_export(t: &TestServer) {
    until("the first export to be taken", || async {
        let seen = t.svc.db.kv_get("notify:seen").ok().flatten();
        seen.filter(|text| text.contains("alpha")).map(|_| ())
    })
    .await;
}

/// The worker takes the exports in order: once the owner is told of a build handed after the
/// others, every export before it has been taken and any ping it owed has been sent.
async fn flush(t: &TestServer, tag: &str) {
    edit_alpha(t, |b| hand_build(b, tag, tag)).await;
    told(t, tag, 1).await;
}

async fn edit_alpha(t: &TestServer, change: impl FnOnce(&mut Value)) {
    let mut board = t.board("alpha");
    change(&mut board);
    write_board(&t.coord_home, "alpha", &board);
    let done = t
        .write("POST", "/api/admin/resync", &json!({ "project": "alpha" }))
        .await;
    assert_eq!(done.status, 200, "{}", done.text());
}

fn set_notify(t: &TestServer, change: impl FnOnce(&mut Value)) {
    let mut config = t.svc.config.write().unwrap_or_else(PoisonError::into_inner);
    let notify = config.extra.get_mut("notify").expect("the notify key");
    change(notify);
}

fn question(id: &str, text: &str, needs: &str) -> Value {
    argus_core::fixture::doc(
        id,
        "ui",
        json!({ "kind": "question", "item": null, "thread": "ui", "author": "ui", "text": text, "needs": needs, "options": [], "status": "open", "replyTo": null, "deliveredAt": null }),
        4096,
    )
}

fn config_notify(dash: &Path) -> Value {
    let text = std::fs::read_to_string(dash.join("config.json")).expect("config.json");
    serde_json::from_str::<Value>(&text).expect("json")["notify"].clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_ping_is_recorded_and_toasted_and_the_refusals_are_plain_400s() {
    let r = rig().await;
    let body = json!({ "title": "Alpha: helper", "text": "build is ready", "project": "alpha", "thread": "helper", "by": "thread" });
    let sent = ping(&r.t, &body).await;
    assert_eq!(sent.status, 200, "{}", sent.text());
    let out = sent.json();
    assert_eq!(
        (&out["recorded"], &out["toast"], &out["level"], &out["why"]),
        (&json!(true), &json!("shown"), &json!("ping"), &Value::Null)
    );
    assert_eq!(r.toasts.count(), 1);
    let url = format!("http://127.0.0.1:{}/p/alpha/", r.t.port);
    assert_eq!(r.toasts.last().url.as_deref(), Some(url.as_str()));
    let id = out["id"].as_str().expect("an id");
    let doc = r.t.svc.db.docs().get(PORTFOLIO, Coll::Notifications, id);
    let data = doc.expect("read").expect("the doc").data;
    for (field, want) in [("by", "thread"), ("project", "alpha"), ("thread", "helper")] {
        assert_eq!(data[field], want, "{field}");
    }
    assert_eq!(
        (&data["title"], &data["toast"]),
        (&json!("Alpha: helper"), &json!("shown"))
    );
    let long = "y".repeat(241);
    for bad in [
        json!({ "title": "x" }),
        json!({ "text": "x", "level": "loud" }),
        json!({ "text": "x", "project": "Bad Key!" }),
        json!({ "text": long }),
    ] {
        assert_eq!(ping(&r.t, &bad).await.status, 400, "{bad}");
    }
    let refused = ping(&r.t, &json!({ "text": long })).await.json();
    let message = refused["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("text is 241 chars, the limit is 240"),
        "{message}"
    );
    assert_eq!(r.toasts.count(), 1, "a refusal shows nothing");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_page_may_send_a_text_over_the_limit_and_it_is_cut() {
    let r = rig().await;
    let sent = as_page(&r.t, "/api/notify", &json!({ "text": "z".repeat(300) })).await;
    assert_eq!(sent.status, 200, "{}", sent.text());
    let top = notes(&r.t).await.remove(0);
    assert_eq!(top["by"], "page");
    assert!(top["text"].as_str().unwrap_or_default().chars().count() <= 240);
}

async fn by_of(t: &TestServer, body: Value, page: bool) -> Value {
    let sent = if page {
        as_page(t, "/api/notify", &body).await
    } else {
        ping(t, &body).await
    };
    assert_eq!(sent.status, 200, "{}", sent.text());
    notes(t).await.remove(0)["by"].clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn by_is_the_page_for_the_page_the_overlord_only_when_the_board_says_so_never_auto() {
    let r = rig().await;
    let t = &r.t;
    let thread = |text, thread, project, by| json!({ "text": text, "thread": thread, "project": project, "by": by });
    assert_eq!(
        by_of(t, thread("a", "coordinator", "alpha", "overlord"), false).await,
        "thread"
    );
    assert_eq!(
        by_of(t, thread("b", "api", "portfolio", "overlord"), false).await,
        "thread"
    );
    assert_eq!(
        by_of(
            t,
            thread("c", "coordinator", "portfolio", "overlord"),
            false
        )
        .await,
        "overlord"
    );
    assert_eq!(
        by_of(t, thread("d", "coordinator", "alpha", "auto"), false).await,
        "thread"
    );
    assert_eq!(
        by_of(t, json!({ "text": "e", "by": "overlord" }), true).await,
        "page"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_overlord_opens_the_notifications_tab() {
    let r = rig().await;
    let body = json!({ "text": "the Overlord needs you", "title": "Overlord", "thread": "coordinator", "project": "portfolio", "by": "overlord" });
    assert_eq!(ping(&r.t, &body).await.status, 200);
    let top = notes(&r.t).await.remove(0);
    assert_eq!(
        (&top["by"], &top["project"], &top["thread"]),
        (
            &json!("overlord"),
            &json!("portfolio"),
            &json!("coordinator")
        )
    );
    let tab = format!("http://127.0.0.1:{}/#notifications", r.t.port);
    assert_eq!(top["url"], tab);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_test_button_is_the_owners_page_only() {
    let r = rig().await;
    let cli = r.t.write("POST", "/api/notify/test", &json!({})).await;
    assert_eq!(cli.status, 403);
    assert_eq!(r.toasts.count(), 0);
    let page = as_page(&r.t, "/api/notify/test", &json!({})).await;
    assert_eq!(page.status, 200, "{}", page.text());
    assert_eq!(page.json()["toast"], "shown");
    assert_eq!(r.toasts.last().title, "Argus: test notification");
    let top = notes(&r.t).await.remove(0);
    assert_eq!(
        (&top["by"], &top["title"], &top["level"]),
        (
            &json!("page"),
            &json!("Argus: test notification"),
            &json!("ping")
        )
    );
}

fn settings(info: bool, ping: bool, sound_ping: bool) -> Value {
    json!({ "os": true, "toast": { "info": info, "ping": ping, "urgent": true }, "sound": { "info": false, "ping": sound_ping, "urgent": true } })
}

async fn current(t: &TestServer) -> Value {
    t.get("/api/notify/settings").await.json()
}

#[tokio::test(flavor = "multi_thread")]
async fn settings_are_the_pages_to_change_and_a_bad_body_is_refused() {
    let r = rig().await;
    let t = &r.t;
    assert_eq!(current(t).await, settings(false, true, false));
    let cli = t
        .write(
            "POST",
            "/api/notify/settings",
            &json!({ "toast": { "ping": false } }),
        )
        .await;
    assert_eq!(cli.status, 403, "the CLI and threads cannot change them");
    assert_eq!(current(t).await["toast"]["ping"], true);
    for bad in [
        json!({ "toast": { "ping": "no" } }),
        json!({ "sound": { "loud": true } }),
        json!({ "toast": [] }),
        json!({ "toast": null }),
    ] {
        assert_eq!(
            as_page(t, "/api/notify/settings", &bad).await.status,
            400,
            "{bad}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_change_of_settings_is_merged_written_to_config_json_and_obeyed_at_once() {
    let r = rig().await;
    let t = &r.t;
    let ok = as_page(
        t,
        "/api/notify/settings",
        &json!({ "toast": { "info": true }, "sound": { "ping": true } }),
    )
    .await;
    assert_eq!(ok.status, 200, "{}", ok.text());
    assert_eq!(ok.json(), settings(true, true, true));
    let saved = config_notify(r.dash.path());
    assert_eq!(saved["toastInfo"], true);
    assert_eq!(
        saved["sound"],
        json!({ "info": false, "ping": true, "urgent": true })
    );
    assert_eq!(
        saved["auto"],
        json!({ "handed": true, "ask": true, "assigned": true, "verify": true, "run": true }),
        "the rest of notify is kept"
    );
    let before = r.toasts.count();
    for (text, level, thread) in [
        ("info with a toast", "info", "sett"),
        ("ping with sound", "ping", "sett2"),
    ] {
        ping(
            t,
            &json!({ "text": text, "level": level, "thread": thread }),
        )
        .await;
    }
    let shown = r.toasts.shown();
    let got: Vec<(Level, bool)> = shown[before..].iter().map(|x| (x.level, x.sound)).collect();
    assert_eq!(got, [(Level::Info, false), (Level::Ping, true)]);
    let off = as_page(
        t,
        "/api/notify/settings",
        &json!({ "toast": { "ping": false } }),
    )
    .await;
    assert_eq!(off.json()["toast"]["ping"], false);
    assert_eq!(config_notify(r.dash.path())["quietLevels"], json!(["ping"]));
    let quiet = ping(t, &json!({ "text": "quiet ping", "thread": "sett3" })).await;
    assert_eq!(quiet.json()["toast"], "skipped");
    let lines = texts(t, "kind=note&limit=10").await;
    let line = lines
        .iter()
        .find(|l| l.starts_with("notification settings"));
    assert_eq!(
        line.map(String::as_str),
        Some(
            "notification settings changed from the page: info toast silent, ping no toast, urgent toast with sound"
        )
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_list_is_newest_first_and_filters_by_level_and_project() {
    let r = rig().await;
    let t = &r.t;
    for (n, (level, project)) in [
        ("ping", "alpha"),
        ("info", "alpha"),
        ("ping", "beta"),
        ("urgent", "alpha"),
    ]
    .into_iter()
    .enumerate()
    {
        let body = json!({ "text": format!("note {n}"), "level": level, "project": project, "thread": format!("t{n}") });
        assert_eq!(ping(t, &body).await.status, 200);
    }
    let all = notes(t).await;
    let numbers: Vec<i64> = all.iter().filter_map(|n| n["n"].as_i64()).collect();
    assert_eq!(numbers, [4, 3, 2, 1]);
    let alpha = t.get("/api/notify?project=alpha").await.json();
    assert_eq!(alpha["notifications"].as_array().map(Vec::len), Some(3));
    let urgent = t.get("/api/notify?level=urgent").await.json();
    assert_eq!(urgent["notifications"][0]["text"], "note 3");
    let two = t.get("/api/notify?limit=2").await.json();
    assert_eq!(two["notifications"].as_array().map(Vec::len), Some(2));
    assert_eq!(t.get("/api/notify?level=loud").await.status, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_newest_fifty_are_kept_as_rows_and_as_docs() {
    let r = rig().await;
    for n in 1..=55 {
        let body = json!({ "text": format!("note {n}"), "level": "info" });
        assert_eq!(ping(&r.t, &body).await.status, 200);
    }
    let all = notes(&r.t).await;
    let numbers: Vec<i64> = all.iter().filter_map(|n| n["n"].as_i64()).collect();
    assert_eq!((numbers.len(), numbers[0], numbers[49]), (50, 55, 6));
    let docs = r.t.svc.db.docs().list(PORTFOLIO, Coll::Notifications);
    assert_eq!(docs.expect("docs").len(), 50);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_thread_is_limited_after_three_a_minute_and_one_summary_toast_says_so() {
    let r = rig().await;
    let lead = |text: &str| json!({ "text": text, "project": "alpha", "thread": "lead" });
    let first = ping(&r.t, &lead("The save-slot build is ready")).await;
    assert_eq!(first.json()["toast"], "shown");
    let top = notes(&r.t).await.remove(0);
    assert_eq!(
        (&top["title"], &top["by"]),
        (&json!("Alpha: lead"), &json!("thread"))
    );
    ping(&r.t, &lead("second")).await;
    ping(&r.t, &lead("third")).await;
    assert_eq!(r.toasts.count(), 3);
    let fourth = ping(&r.t, &lead("fourth")).await.json();
    assert_eq!(fourth["toast"], "limited");
    assert_eq!(fourth["why"], "you pinged 4 times in a minute");
    assert_eq!(r.toasts.count(), 4, "only the summary toast");
    assert_eq!(
        r.toasts.last().text,
        "1 more notification from alpha/lead: see Argus"
    );
    r.clock.advance(61_000);
    assert_eq!(
        ping(&r.t, &lead("a minute later")).await.json()["toast"],
        "shown"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn info_is_recorded_only_urgent_keeps_its_level_and_a_bad_ping_is_refused() {
    let r = rig().await;
    let helper = |text: &str, level: &str| json!({ "text": text, "level": level, "project": "alpha", "thread": "helper" });
    let info = ping(&r.t, &helper("just so you know", "info")).await.json();
    assert_eq!(
        (&info["toast"], &info["why"]),
        (&json!("skipped"), &json!("info is recorded only"))
    );
    assert_eq!(r.toasts.count(), 0);
    let urgent = ping(&r.t, &json!({ "text": "the server is on fire", "level": "urgent", "title": "Fire", "thread": "helper" })).await;
    assert_eq!(urgent.json()["toast"], "shown");
    assert_eq!(r.toasts.last().level, Level::Urgent);
    assert_eq!(ping(&r.t, &helper("x", "loud")).await.status, 400);
    assert_eq!(ping(&r.t, &json!({ "text": "   " })).await.status, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn every_ping_has_an_activity_line_info_has_none_and_a_failing_toast_is_a_warning() {
    let r = rig().await;
    let lead = |text: &str| json!({ "text": text, "project": "alpha", "thread": "lead" });
    ping(&r.t, &lead("The save-slot build is ready")).await;
    ping(&r.t, &json!({ "text": "the server is on fire", "level": "urgent", "title": "Fire", "project": "alpha", "thread": "helper" })).await;
    ping(
        &r.t,
        &json!({ "text": "just so you know", "level": "info" }),
    )
    .await;
    r.toasts.refuse("no toast host");
    let failed = ping(
        &r.t,
        &json!({ "text": "x", "project": "alpha", "thread": "failing" }),
    )
    .await
    .json();
    assert_eq!(
        (&failed["toast"], &failed["why"]),
        (&json!("failed"), &json!("no toast host"))
    );
    let lines = texts(&r.t, "kind=note&limit=200").await;
    for want in [
        "ping from alpha/lead: Alpha: lead: The save-slot build is ready (toast shown)",
        "urgent ping from alpha/helper: Fire: the server is on fire (toast shown)",
        "ping from alpha/failing: Alpha: failing: x (no toast: failed, no toast host)",
    ] {
        assert!(lines.iter().any(|l| l == want), "{want} in {lines:?}");
    }
    assert!(
        !lines.iter().any(|l| l.contains("just so you know")),
        "info writes no line"
    );
    let warns = texts(&r.t, "kind=note&level=warn").await;
    assert_eq!(warns.len(), 1, "{warns:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_failed_toast_becomes_a_tray_balloon_when_a_tray_runs() {
    let r = rig_with(true).await;
    let alive = format!(
        "{{\"pid\":{},\"startedAt\":\"2026-10-26T12:00:00.000Z\"}}\n",
        std::process::id()
    );
    std::fs::write(r.dash.path().join("tray.json"), alive).expect("tray.json");
    r.toasts.refuse("no toast host");
    let sent = ping(
        &r.t,
        &json!({ "text": "build failed", "title": "Alpha", "project": "alpha", "thread": "lead" }),
    )
    .await
    .json();
    assert_eq!(sent["toast"], "shown");
    let why = sent["why"].as_str().unwrap_or_default();
    assert!(
        why.starts_with("shown as a tray balloon (") && why.contains("no toast host"),
        "{why}"
    );
    let left =
        std::fs::read_to_string(r.dash.path().join("tray-notify.json")).expect("balloon file");
    assert!(
        left.contains("\"title\":\"Alpha\"") && left.contains("build failed"),
        "{left}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_hard_resync_writes_the_docs_again() {
    let r = rig().await;
    let id = ping(&r.t, &json!({ "text": "keep me" })).await.json()["id"]
        .as_str()
        .expect("an id")
        .to_owned();
    let done =
        r.t.write("POST", "/api/admin/resync", &json!({ "hard": true }))
            .await;
    assert_eq!(done.status, 200, "{}", done.text());
    let doc = r.t.svc.db.docs().get(PORTFOLIO, Coll::Notifications, &id);
    assert!(doc.expect("read").is_some());
}

fn hand_build(board: &mut Value, tag: &str, title: &str) {
    board["launchers"][tag] = json!({ "name": tag });
    board["items"][tag] = json!({
        "id": tag, "type": "test-run", "title": title, "thread": "ui", "stage": "ready",
        "assignee": "owner", "updatedAt": ago(0), "fields": { "plan": "I-77", "build": tag }
    });
    for (n, text) in [(3, "Slots persist"), (4, "Slots survive a crash")] {
        let id = format!("T-{tag}-{n}");
        let fields = json!({ "title": text, "mode": "manual" });
        board["tests"][id.as_str()] = test_doc(&id, "I-77", "ui", fields, 4096);
    }
}

fn hand_b2(board: &mut Value) {
    hand_build(board, "b2", "Save slots");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_build_handed_pings_the_owner_once_and_a_resync_tells_nothing_more() {
    let r = rig().await;
    first_export(&r.t).await;
    assert_eq!(
        count_of(&notes(&r.t).await, ""),
        0,
        "the first export only sets the markers"
    );
    edit_alpha(&r.t, hand_b2).await;
    told(&r.t, "ready to test", 1).await;
    edit_alpha(&r.t, |_| {}).await;
    flush(&r.t, "b3").await;
    let list = notes(&r.t).await;
    assert_eq!(count_of(&list, "Save slots"), 1);
    let hand = list
        .iter()
        .find(|n| n["by"] == "auto" && n["text"].as_str().is_some_and(|t| t.contains("Save slots")))
        .expect("the ping");
    let got = (
        &hand["text"],
        &hand["level"],
        &hand["project"],
        &hand["thread"],
        &hand["toast"],
        &hand["title"],
    );
    let want = (
        &json!("Alpha: Save slots is ready to test (2 tests)"),
        &json!("ping"),
        &json!("alpha"),
        &json!("ui"),
        &json!("shown"),
        &json!("Alpha: ready to test"),
    );
    assert_eq!(got, want);
    let lines = texts(&r.t, "kind=note&limit=50").await.join("\n");
    assert!(lines.contains("automatic ping: Alpha: ready to test: Alpha: Save slots is ready to test (2 tests) (toast shown)"), "{lines}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_question_pings_once_with_the_first_100_characters_and_a_notice_does_not() {
    let r = rig().await;
    first_export(&r.t).await;
    let said = format!(
        "Should the slots be per profile or shared across the whole install? {}",
        "Think about it. ".repeat(10)
    );
    edit_alpha(&r.t, |b| {
        b["entries"]["Q-2"] = question("Q-2", &said, "answer");
        b["entries"]["N-2"] = argus_core::fixture::doc("N-2", "ui", json!({ "kind": "notice", "item": null, "thread": "ui", "author": "ui", "text": "just a notice", "level": "info", "status": "open", "replyTo": null, "deliveredAt": null }), 5120);
    })
    .await;
    told(&r.t, "asks:", 1).await;
    flush(&r.t, "b3").await;
    let list = notes(&r.t).await;
    let asks: Vec<&Value> = list
        .iter()
        .filter(|n| n["by"] == "auto" && !n["text"].as_str().unwrap_or_default().contains("b3"))
        .collect();
    assert_eq!(asks.len(), 1);
    let head: String = said.chars().take(100).collect();
    assert_eq!(asks[0]["text"], format!("Alpha asks: {head}\u{2026}"));
    assert_eq!(
        (&asks[0]["title"], &asks[0]["thread"], &asks[0]["level"]),
        (&json!("Alpha has a question"), &json!("ui"), &json!("ping"))
    );
    edit_alpha(&r.t, |b| {
        b["entries"]["Q-3"] = question("Q-3", "Which of these two?", "decision");
    })
    .await;
    let list = told(&r.t, "asks:", 2).await;
    let newest = list
        .iter()
        .find(|n| n["by"] == "auto")
        .expect("the decision");
    assert_eq!(newest["title"], "Alpha needs a decision");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_restart_does_not_tell_the_owner_again_what_he_was_told_but_news_still_comes() {
    let r = rig().await;
    first_export(&r.t).await;
    edit_alpha(&r.t, hand_b2).await;
    edit_alpha(&r.t, |b| {
        b["entries"]["Q-2"] = question("Q-2", "Per profile?", "answer");
    })
    .await;
    let before = told(&r.t, "asks:", 1).await;
    let before: Vec<Value> = before.into_iter().filter(|n| n["by"] == "auto").collect();
    let shown = r.toasts.count();
    let r = restart(r).await;
    r.t.write("POST", "/api/admin/resync", &json!({})).await;
    flush(&r.t, "b3").await;
    let after = notes(&r.t).await;
    let told_texts = |list: &[Value]| -> Vec<Value> {
        list.iter()
            .filter(|n| n["by"] == "auto" && !n["text"].as_str().unwrap_or_default().contains("b3"))
            .map(|n| n["text"].clone())
            .collect()
    };
    assert_eq!(told_texts(&after), told_texts(&before));
    assert_eq!(
        r.toasts.count(),
        shown + 1,
        "only the toast of the last build"
    );
    edit_alpha(&r.t, |b| {
        b["entries"]["Q-4"] = question("Q-4", "One more thing after the restart?", "answer");
    })
    .await;
    let list = told(&r.t, "One more thing after the restart", 1).await;
    assert_eq!(count_of(&list, "One more thing after the restart"), 1);
    assert_eq!(r.toasts.count(), shown + 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn notify_os_false_records_a_question_without_a_toast_and_auto_ask_false_does_not_record_it()
{
    let r = rig().await;
    first_export(&r.t).await;
    set_notify(&r.t, |n| n["os"] = json!(false));
    edit_alpha(&r.t, |b| {
        b["entries"]["Q-2"] = question("Q-2", "Question with the OS toasts off?", "answer");
    })
    .await;
    let list = told(&r.t, "Question with the OS toasts off", 1).await;
    let n = list.iter().find(|n| n["by"] == "auto").expect("recorded");
    assert_eq!(
        (&n["toast"], &n["why"]),
        (
            &json!("skipped"),
            &json!("OS notifications are off (config notify.os)")
        )
    );
    set_notify(&r.t, |n| {
        n["os"] = json!(true);
        n["auto"]["ask"] = json!(false);
    });
    edit_alpha(&r.t, |b| {
        b["entries"]["Q-3"] = question("Q-3", "Question with auto ask off?", "answer");
    })
    .await;
    flush(&r.t, "b3").await;
    assert_eq!(
        count_of(&notes(&r.t).await, "Question with auto ask off"),
        0
    );
}

fn owned_item(id: &str, title: &str) -> Value {
    argus_core::fixture::doc(
        id,
        "ui",
        json!({ "title": title, "thread": "ui", "status": "doing", "assignee": "owner" }),
        8192,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn items_a_thread_assigns_to_the_owner_ping_him_once_inside_the_toast_limits() {
    let r = rig().await;
    first_export(&r.t).await;
    let base = r.toasts.count();
    edit_alpha(&r.t, |b| {
        b["items"]["I-1"]["assignee"] = json!("owner");
        b["items"]["I-1"]["rev"] = json!(2);
        for n in 3..=6 {
            let id = format!("I-{n}");
            b["items"][id.as_str()] = owned_item(&id, &format!("Chore {n}"));
        }
    })
    .await;
    let list = told(&r.t, "was assigned to you by ui", 5).await;
    let states: Vec<&str> = list
        .iter()
        .filter(|n| n["by"] == "auto")
        .filter(|n| {
            n["text"]
                .as_str()
                .is_some_and(|t| t.contains("assigned to you"))
        })
        .filter_map(|n| n["toast"].as_str())
        .collect();
    assert_eq!(states.iter().filter(|s| **s == "shown").count(), 3);
    assert_eq!(states.iter().filter(|s| **s == "limited").count(), 2);
    assert_eq!(r.toasts.count() - base, 4, "three toasts and one summary");
    let r = restart(r).await;
    r.t.write("POST", "/api/admin/resync", &json!({})).await;
    flush(&r.t, "b3").await;
    assert_eq!(count_of(&notes(&r.t).await, "was assigned to you by ui"), 5);
}

#[tokio::test(flavor = "multi_thread")]
async fn auto_assigned_false_keeps_the_ping_off_and_a_bug_to_verify_pings_the_owner() {
    let r = rig().await;
    first_export(&r.t).await;
    set_notify(&r.t, |n| n["auto"]["assigned"] = json!(false));
    edit_alpha(&r.t, |b| {
        b["items"]["I-7"] = owned_item("I-7", "Switched off");
    })
    .await;
    flush(&r.t, "b3").await;
    assert_eq!(count_of(&notes(&r.t).await, "Switched off"), 0);
    edit_alpha(&r.t, |b| {
        let mut bug = owned_item("I-8", "Login loses the user");
        bug["assignee"] = json!("ui");
        bug["type"] = json!("bug");
        bug["stage"] = json!("verifying");
        bug["status"] = json!("review");
        bug["fields"] = json!({ "reporter": "owner" });
        b["items"]["I-8"] = bug;
    })
    .await;
    let list = told(&r.t, "is ready to verify", 1).await;
    assert_eq!(count_of(&list, "Login loses the user"), 1);
}
