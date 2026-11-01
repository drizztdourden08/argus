//! Ports the `coord ping` cases of `notify.http.test.mjs` against an Argus test server whose
//! toaster shows nothing: the thread, the default title, the sentence, the limit after three a
//! minute, the levels, the refusals, and the Overlord pinging as the Overlord.

use std::path::Path;
use std::sync::Arc;

use crate::common::{ThreadSpec, board_with};
use crate::remote::{LEAD, MAIN, Rig, make_checkout, ok, refused};
use crate::toasts::Toasts;
use argus_core::clock::{ManualClock, now_ms};
use argus_native::{Level, Notifier};
use argus_server::ServeOptions;
use serde_json::{Value, json};

const HELPER: &str = "sess-helper";

fn seed(coord_home: &Path, base: &Path) {
    let alpha_root = make_checkout(base, "alpha");
    let main_root = make_checkout(base, "portfolio");
    let alpha: [ThreadSpec<'_>; 2] = [
        (
            "lead",
            "coordinator",
            "active",
            Some(LEAD),
            Some(&alpha_root),
        ),
        ("helper", "thread", "active", Some(HELPER), None),
    ];
    let mut board = board_with("alpha", &alpha);
    board["name"] = json!("Alpha");
    argus_testkit::write_board(coord_home, "alpha", &board);
    let portfolio = [(
        "coordinator",
        "coordinator",
        "active",
        Some(MAIN),
        Some(main_root.as_path()),
    )];
    argus_testkit::write_board(
        coord_home,
        "portfolio",
        &board_with("portfolio", &portfolio),
    );
}

async fn rig() -> (Rig, Arc<Toasts>) {
    let toasts = Arc::new(Toasts::default());
    let (shown, clock) = (Arc::clone(&toasts), ManualClock::at(now_ms()));
    let tweak = Arc::new(move |opts: &mut ServeOptions| {
        opts.notify.notifier = Some(Arc::clone(&shown) as Arc<dyn Notifier>);
        opts.notify.clock = Some(Arc::new(clock.clone()));
    });
    (Rig::start_with(json!({}), seed, Some(tweak)).await, toasts)
}

fn ping(rig: &Rig, words: &[&str], session: &str, folder: &str) -> crate::common::Ran {
    let mut argv = vec!["ping"];
    argv.extend(words);
    rig.run_in(&argv, Some(session), &rig.dir.path().join(folder))
}

async fn newest(rig: &Rig) -> Value {
    rig.t.get("/api/notify?limit=1").await.json()["notifications"][0].clone()
}

/// The first ping of the lead, as the dashboard recorded it.
fn assert_recorded(n: &Value) {
    let seen = json!([
        n["by"],
        n["project"],
        n["thread"],
        n["title"],
        n["text"],
        n["level"],
        n["toast"]
    ]);
    let want = json!([
        "thread",
        "alpha",
        "lead",
        "Alpha: lead",
        "The save-slot build is ready",
        "ping",
        "shown"
    ]);
    assert_eq!(seen, want);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_ping_names_the_thread_the_default_title_and_the_sentence_and_is_limited_after_three_a_minute()
 {
    let (rig, toasts) = rig().await;
    let first = ping(
        &rig,
        &["The save-slot build is ready"],
        LEAD,
        "checkout-alpha",
    );
    assert!(
        ok(&first).starts_with("pinged the owner: Alpha: lead (toast shown)."),
        "{}",
        first.out
    );
    assert_recorded(&newest(&rig).await);
    let titled = ping(
        &rig,
        &["second", "--title", "Custom title"],
        LEAD,
        "checkout-alpha",
    );
    assert!(
        ok(&titled).contains("pinged the owner: Custom title"),
        "{}",
        titled.out
    );
    ok(&ping(&rig, &["third"], LEAD, "checkout-alpha"));
    let before = toasts.count();
    let fourth = ping(&rig, &["fourth"], LEAD, "checkout-alpha");
    assert_eq!(
        ok(&fourth).trim(),
        "recorded, no toast: you pinged 4 times in a minute."
    );
    assert_eq!(newest(&rig).await["toast"], "limited");
    assert_eq!(toasts.count(), before + 1, "only the summary toast");
    assert!(
        toasts
            .last()
            .text
            .starts_with("1 more notification from alpha/lead: see Argus")
    );
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn levels_info_is_recorded_only_urgent_carries_its_warning_and_bad_input_is_refused() {
    let (rig, toasts) = rig().await;
    let info = ping(
        &rig,
        &["just so you know", "--level", "info"],
        HELPER,
        "checkout-alpha",
    );
    assert_eq!(
        ok(&info).trim(),
        "recorded, no toast: info is recorded only."
    );
    assert_eq!(toasts.count(), 0);
    let urgent = ping(
        &rig,
        &[
            "the server is on fire",
            "--level",
            "urgent",
            "--title",
            "Fire",
        ],
        HELPER,
        "checkout-alpha",
    );
    let out = ok(&urgent);
    assert!(out.contains("pinged the owner: Fire (toast shown). Urgent stays on his screen until he dismisses it: use it only when he must act now."), "{out}");
    assert_eq!(toasts.last().level, Level::Urgent);
    let long = "y".repeat(241);
    let long_title = "T".repeat(61);
    let cases: [(Vec<&str>, &str); 4] = [
        (
            vec!["x", "--level", "loud"],
            "--level must be one of info, ping, urgent",
        ),
        (vec![long.as_str()], "ping is 241 chars, the limit is 240"),
        (vec![], "ping text is required"),
        (
            vec!["x", "--title", long_title.as_str()],
            "title is 61 chars, the limit is 60",
        ),
    ];
    for (words, text) in cases {
        refused(&ping(&rig, &words, HELPER, "checkout-alpha"), 1, text);
    }
    refused(
        &ping(&rig, &["x"], "sess-unknown", "."),
        1,
        "cannot tell which project",
    );
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_overlord_pings_as_the_overlord_and_a_session_that_is_no_thread_is_told_to_register() {
    let (rig, _toasts) = rig().await;
    let ran = ping(
        &rig,
        &["the Overlord needs you", "--title", "Overlord"],
        MAIN,
        "checkout-portfolio",
    );
    assert!(
        ok(&ran).contains("pinged the owner: Overlord"),
        "{}",
        ran.out
    );
    let n = newest(&rig).await;
    assert_eq!(
        json!([n["by"], n["project"], n["thread"]]),
        json!(["overlord", "portfolio", "coordinator"])
    );
    refused(
        &ping(&rig, &["who am I"], "sess-nobody", "checkout-alpha"),
        1,
        "no thread registered for this session",
    );
    rig.stop().await;
}

#[test]
fn a_dashboard_that_cannot_come_up_is_an_error_not_a_silent_success() {
    let home = crate::common::Home::new();
    let checkout = make_checkout(home.path(), "alpha");
    argus_testkit::write_board(
        &home.coord_home(),
        "alpha",
        &crate::remote::lead_board("alpha", &checkout),
    );
    let missing = home
        .path()
        .join("no-install")
        .to_string_lossy()
        .into_owned();
    let env = crate::remote::env_plus(
        &home.env(),
        &[
            ("COORD_DASHBOARD_WAIT_MS", "300"),
            ("COORD_DASHBOARD_INSTALL", &missing),
            ("CLAUDE_CODE_SESSION_ID", ""),
        ],
    );
    let ran = crate::common::coord(&["ping", "hello"], env, &checkout);
    refused(
        &ran,
        1,
        "coord: the dashboard is not running (dashboard start): no dashboard install found",
    );
}
