//! Bundles and their launchers (build.verbs, the pure parts): close and reopen retire and
//! restore the launchers made for a bundle, the status of a bundle, and the rules for
//! handing a build to the owner.
//!
//! Keep rule: K1, K4.
// test-tag: critical

use argus_launchers::launcher::{
    Bundle, BundleStatus, CloseRequest, Launcher, LauncherBoard, LauncherPatch, WriteContext,
    check_hand, close_bundle, ensure_bundle, reopen_bundle, restore_made_for, retire_launcher,
    retire_made_for, set_bundle_status, write_launcher,
};

use crate::launcher_rules::{AT, board, launcher, lead, worker};

fn bundle(tag: &str, launcher: Option<&str>) -> Bundle {
    Bundle {
        tag: tag.to_owned(),
        thread: Some("worker".to_owned()),
        launcher: launcher.map(str::to_owned),
        ..Bundle::default()
    }
}

fn patch(cmd: &str) -> LauncherPatch {
    LauncherPatch {
        cmd: Some(Some(cmd.to_owned())),
        for_item: Some(Some("I-1".to_owned())),
        ..LauncherPatch::default()
    }
}

/// A launcher made for bundle `tag` (as `coord build add --cmd` does) and its bundle.
fn made_for_bundle(board: &mut LauncherBoard, tag: &str) {
    let ctx = WriteContext {
        cwd: "/p",
        bundle: Some(tag),
        at: AT,
    };
    write_launcher(board, &worker("worker"), tag, &patch("node j.js"), &ctx).unwrap();
    board.builds.insert(tag.to_owned(), bundle(tag, Some(tag)));
}

fn setup() -> LauncherBoard {
    let mut board = board();
    board.launchers.insert(
        "shared-1".to_owned(),
        Launcher {
            cmd: Some("game run --shared".to_owned()),
            use_every: Some("30d".to_owned()),
            ..launcher("shared-1")
        },
    );
    made_for_bundle(&mut board, "j-1");
    board
        .builds
        .insert("j-2".to_owned(), bundle("j-2", Some("shared-1")));
    board
}

fn close(board: &mut LauncherBoard, tag: &str) -> argus_launchers::launcher::CloseOutcome {
    close_bundle(board, &worker("worker"), tag, &CloseRequest::default(), AT).unwrap()
}

#[test]
fn closing_a_bundle_retires_the_launcher_made_for_it_and_only_that_one() {
    let mut board = setup();
    assert!(
        close(&mut board, "j-2").retired.is_empty(),
        "a shared launcher is never retired by a close"
    );
    assert!(board.launchers["shared-1"].retired_at.is_none());
    let closed = close(&mut board, "j-1");
    assert_eq!(closed.retired, ["j-1"]);
    assert_eq!(
        board.launchers["j-1"].retire_note.as_deref(),
        Some("bundle j-1 closed")
    );
    assert_eq!(
        board.launchers["j-1"].retired_by_bundle.as_deref(),
        Some("j-1")
    );
    assert!(!board.run_defs.contains_key("launcher-j-1"));
    assert!(board.launchers["master"].retired_at.is_none());
    assert!(
        board
            .audit
            .last()
            .unwrap()
            .text
            .contains("closed bundle j-1, retired launcher j-1")
    );
}

#[test]
fn reopen_brings_back_what_the_close_retired_and_its_play() {
    let mut board = setup();
    close(&mut board, "j-1");
    let back = reopen_bundle(&mut board, &worker("worker"), "j-1", Some("/p"), AT).unwrap();
    assert_eq!(back, Some(vec!["j-1".to_owned()]));
    assert!(board.launchers["j-1"].retired_at.is_none());
    assert!(board.run_defs.contains_key("launcher-j-1"));
    assert!(board.builds["j-1"].closed_at.is_none());
    assert_eq!(
        reopen_bundle(&mut board, &lead(), "j-1", None, AT).unwrap(),
        None,
        "not closed"
    );
    assert!(
        reopen_bundle(&mut board, &lead(), "nope", None, AT)
            .unwrap_err()
            .to_string()
            .contains("no bundle \"nope\"")
    );
}

#[test]
fn a_launcher_retired_some_other_way_stays_retired_through_a_close_and_a_reopen() {
    let mut board = setup();
    retire_launcher(&mut board, &worker("worker"), "j-1", Some("replaced"), AT).unwrap();
    assert!(close(&mut board, "j-1").retired.is_empty());
    let back = reopen_bundle(&mut board, &worker("worker"), "j-1", None, AT)
        .unwrap()
        .unwrap();
    assert!(back.is_empty());
    assert_eq!(
        board.launchers["j-1"].retire_note.as_deref(),
        Some("replaced")
    );
    assert!(board.launchers["j-1"].retired_at.is_some());
}

#[test]
fn a_launcher_another_open_bundle_links_is_not_retired_by_this_bundles_close() {
    let mut board = setup();
    board
        .builds
        .insert("j-3".to_owned(), bundle("j-3", Some("j-1")));
    assert!(
        close(&mut board, "j-1").retired.is_empty(),
        "j-3 still links it"
    );
    assert!(board.launchers["j-1"].retired_at.is_none());
    close(&mut board, "j-3");
    reopen_bundle(&mut board, &worker("worker"), "j-1", None, AT).unwrap();
}

#[test]
fn a_removal_retires_for_good_and_no_reopen_brings_it_back() {
    let mut board = setup();
    let gone = retire_made_for(
        &mut board,
        &worker("worker"),
        "j-1",
        ("bundle j-1 removed", false),
        AT,
    )
    .unwrap();
    assert_eq!(gone, ["j-1"]);
    assert!(board.launchers["j-1"].retired_at.is_some());
    assert_eq!(board.launchers["j-1"].retired_by_bundle, None);
    assert!(
        restore_made_for(&mut board, &worker("worker"), "j-1", None, AT)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn closing_refuses_while_manual_tests_are_pending_unless_forced_and_unhands_the_handed_build() {
    let mut board = setup();
    let pending = vec!["T-1".to_owned(), "T-2".to_owned()];
    let request = CloseRequest {
        note: Some("landed in 1.4"),
        force: false,
        pending_tests: &pending,
    };
    let refused = close_bundle(&mut board, &worker("worker"), "j-1", &request, AT)
        .unwrap_err()
        .to_string();
    assert!(refused.starts_with("2 manual test(s) of j-1 are still to run: T-1, T-2."));
    assert!(board.builds["j-1"].closed_at.is_none());
    board.handed_build = Some("j-1".to_owned());
    let forced = close_bundle(
        &mut board,
        &worker("worker"),
        "j-1",
        &CloseRequest {
            force: true,
            ..request
        },
        AT,
    )
    .unwrap();
    assert!(forced.unhanded && !forced.already);
    assert_eq!(
        (board.handed_build.clone(), board.handed_at.clone()),
        (None, None)
    );
    let rec = &board.builds["j-1"];
    assert_eq!(
        (rec.closed_by.as_deref(), rec.close_note.as_deref()),
        (Some("worker"), Some("landed in 1.4"))
    );
    assert!(
        board
            .audit
            .last()
            .unwrap()
            .text
            .contains("(unhanded) with 2 test(s) still to run")
    );
    assert!(
        close_bundle(&mut board, &lead(), "j-1", &CloseRequest::default(), AT)
            .unwrap()
            .already
    );
}

#[test]
fn a_tag_with_no_record_is_made_by_the_close_and_ensure_bundle_makes_one_once() {
    let mut board = setup();
    close_bundle(&mut board, &lead(), "ghost-1", &CloseRequest::default(), AT).unwrap();
    assert_eq!(
        board.builds["ghost-1"].closed_by.as_deref(),
        Some("coordinator")
    );
    ensure_bundle(&mut board, "new-1", "worker", AT).title = Some("kept".to_owned());
    assert_eq!(
        ensure_bundle(&mut board, "new-1", "other", AT)
            .title
            .as_deref(),
        Some("kept")
    );
    assert_eq!(board.builds["new-1"].created_by.as_deref(), Some("worker"));
}

#[test]
fn ready_and_draft_a_closed_or_handed_bundle_cannot_go_back_and_an_unchanged_status_is_quiet() {
    let mut board = setup();
    board.builds.insert(
        "s-1".to_owned(),
        Bundle {
            tag: "s-1".to_owned(),
            ..Bundle::default()
        },
    );
    assert!(set_bundle_status(&mut board, &lead(), "s-1", BundleStatus::Ready, AT).unwrap());
    assert_eq!(board.builds["s-1"].status.as_deref(), Some("ready"));
    assert_eq!(board.audit.last().unwrap().text, "bundle s-1 is ready");
    let audits = board.audit.len();
    assert!(!set_bundle_status(&mut board, &lead(), "s-1", BundleStatus::Ready, AT).unwrap());
    assert_eq!(board.audit.len(), audits, "already ready: no audit line");
    board.handed_build = Some("s-1".to_owned());
    let handed = set_bundle_status(&mut board, &lead(), "s-1", BundleStatus::Draft, AT)
        .unwrap_err()
        .to_string();
    assert!(handed.contains("is the handed build: coord build unhand first"));
    board.handed_build = None;
    assert!(set_bundle_status(&mut board, &lead(), "s-1", BundleStatus::Draft, AT).unwrap());
    close(&mut board, "s-1");
    let closed = set_bundle_status(&mut board, &lead(), "s-1", BundleStatus::Ready, AT)
        .unwrap_err()
        .to_string();
    assert_eq!(
        closed,
        "bundle s-1 is closed: coord build reopen s-1 first."
    );
    assert!(
        set_bundle_status(&mut board, &lead(), "nope", BundleStatus::Ready, AT)
            .unwrap_err()
            .to_string()
            .contains("coord build add nope makes it")
    );
}

#[test]
fn hand_is_refused_for_a_closed_bundle_a_draft_and_a_project_without_a_master() {
    let mut board = setup();
    board.builds.insert(
        "x-1".to_owned(),
        Bundle {
            tag: "x-1".to_owned(),
            ..Bundle::default()
        },
    );
    let draft = check_hand(&board, "x-1").unwrap_err().to_string();
    assert!(
        draft.contains("bundle x-1 is a draft (not built yet)")
            && draft.contains("coord build ready x-1")
    );
    set_bundle_status(&mut board, &lead(), "x-1", BundleStatus::Ready, AT).unwrap();
    assert!(check_hand(&board, "x-1").is_ok());
    assert!(
        check_hand(&board, "no-record").is_ok(),
        "hand makes a missing record"
    );
    close(&mut board, "x-1");
    assert_eq!(
        check_hand(&board, "x-1").unwrap_err().to_string(),
        "bundle x-1 is closed: coord build reopen x-1 first."
    );
    board.launchers.remove("master");
    let no_master = check_hand(&board, "no-record").unwrap_err().to_string();
    assert!(no_master.contains("no master launcher, so nothing can be handed to the owner yet"));
    assert!(no_master.contains("Set master first: coord launcher set master --cmd"));
}
