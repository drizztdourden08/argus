//! Builds that serve an item (I-78): the `for` link takes the usage window away, the master role
//! is one flag handed over by the coordinator, and `link_launcher` moves a build to an item.
//!
//! Keep rule: K1, K4.
// test-tag: critical

use argus_launchers::launcher::{
    LauncherBoard, LauncherPatch, WriteContext, has_master, is_master, is_standalone,
    link_launcher, retire_launcher, usage_of, write_launcher,
};

use crate::launcher_rules::{AT, board, launcher, lead, worker};

fn ctx() -> WriteContext<'static> {
    WriteContext {
        cwd: "/somewhere",
        bundle: None,
        at: AT,
    }
}

fn serving(cmd: &str, item: &str) -> LauncherPatch {
    LauncherPatch {
        cmd: Some(Some(cmd.to_owned())),
        for_item: Some(Some(item.to_owned())),
        ..LauncherPatch::default()
    }
}

fn master_patch() -> LauncherPatch {
    LauncherPatch {
        master: Some(true),
        ..LauncherPatch::default()
    }
}

fn with_build(item: &str) -> LauncherBoard {
    let mut board = board();
    write_launcher(
        &mut board,
        &lead(),
        "build-2",
        &serving("game run --two", item),
        &ctx(),
    )
    .unwrap();
    board
}

#[test]
fn a_build_that_serves_an_item_has_no_usage_window_and_drops_a_use_every() {
    let mut board = board();
    let patch = LauncherPatch {
        use_every: Some(Some("7d".to_owned())),
        ..serving("game run --two", "I-5")
    };
    let made = write_launcher(&mut board, &lead(), "build-2", &patch, &ctx()).unwrap();
    assert_eq!(made.ignored, ["useEvery"]);
    let rec = &board.launchers["build-2"];
    assert_eq!(rec.for_item.as_deref(), Some("I-5"));
    assert_eq!(rec.use_every, None);
    assert!(!is_standalone(&board, rec));
    assert!(usage_of(&board, rec, 1_790_000_000_000, 0).is_none());
}

#[test]
fn a_new_build_with_no_link_and_no_window_is_still_refused_on_an_old_board() {
    let mut board = board();
    let patch = LauncherPatch {
        cmd: Some(Some("game run --two".to_owned())),
        ..LauncherPatch::default()
    };
    let refused = write_launcher(&mut board, &lead(), "build-2", &patch, &ctx()).unwrap_err();
    assert!(refused.0.contains("--use-every"), "{refused}");
}

#[test]
fn the_master_role_is_handed_over_by_the_coordinator_and_the_old_master_waits_for_a_link() {
    let mut board = with_build("I-5");
    let res = write_launcher(&mut board, &lead(), "build-2", &master_patch(), &ctx()).unwrap();
    assert_eq!(res.changed, ["master"]);
    assert_eq!(res.previous_master.as_deref(), Some("master"));
    assert!(board.launchers["build-2"].master);
    assert!(!board.launchers["master"].master);
    assert!(board.launchers["master"].relink);
    assert!(is_master(&board, "build-2") && !is_master(&board, "master"));
    assert!(has_master(&board));
}

#[test]
fn a_thread_cannot_hand_the_master_role_over() {
    let mut board = with_build("I-5");
    let refused = write_launcher(
        &mut board,
        &worker("worker"),
        "build-2",
        &master_patch(),
        &ctx(),
    )
    .unwrap_err();
    assert!(refused.0.contains("coordinator"), "{refused}");
    assert!(!board.launchers["build-2"].master);
}

#[test]
fn the_flagged_master_is_never_retired_and_the_name_master_counts_only_without_a_flag() {
    let mut board = with_build("I-5");
    write_launcher(&mut board, &lead(), "build-2", &master_patch(), &ctx()).unwrap();
    let refused = retire_launcher(&mut board, &lead(), "build-2", None, AT).unwrap_err();
    assert!(
        refused.0.starts_with("build-2 is never retired"),
        "{refused}"
    );
    retire_launcher(&mut board, &lead(), "master", None, AT).unwrap();
    assert!(board.launchers["master"].retired_at.is_some());
}

#[test]
fn a_new_build_made_as_master_needs_no_window_and_takes_the_role() {
    let mut board = board();
    let patch = LauncherPatch {
        cmd: Some(Some("game run --main".to_owned())),
        master: Some(true),
        ..LauncherPatch::default()
    };
    let made = write_launcher(&mut board, &lead(), "main", &patch, &ctx()).unwrap();
    assert_eq!(made.previous_master.as_deref(), Some("master"));
    assert!(board.launchers["main"].master && board.launchers["main"].use_every.is_none());
}

#[test]
fn link_launcher_sets_the_item_and_clears_a_pending_relink() {
    let mut board = board();
    board.launchers.insert("tool".to_owned(), {
        let mut rec = launcher("tool");
        rec.relink = true;
        rec
    });
    link_launcher(&mut board, &lead(), ("tool", "I-9"), AT).unwrap();
    let rec = &board.launchers["tool"];
    assert_eq!(rec.for_item.as_deref(), Some("I-9"));
    assert!(!rec.relink);
    assert!(
        board
            .audit
            .iter()
            .any(|l| l.text == "linked launcher tool to I-9")
    );
}

#[test]
fn link_launcher_refuses_a_retired_build_and_a_build_of_another_thread() {
    let mut board = board();
    board.launchers.insert("tool".to_owned(), launcher("tool"));
    let other = link_launcher(&mut board, &worker("other"), ("tool", "I-9"), AT).unwrap_err();
    assert!(other.0.contains("only"), "{other}");
    retire_launcher(&mut board, &lead(), "tool", None, AT).unwrap();
    let retired = link_launcher(&mut board, &lead(), ("tool", "I-9"), AT).unwrap_err();
    assert!(retired.0.contains("is retired"), "{retired}");
}

#[test]
fn a_write_that_brings_a_build_back_forgets_the_item_that_closed_it() {
    let mut board = with_build("I-5");
    if let Some(rec) = board.launchers.get_mut("build-2") {
        rec.retired_at = Some(AT.to_owned());
        rec.retired_by = Some("coordinator".to_owned());
        rec.retired_by_item = Some("I-5".to_owned());
    }
    let patch = LauncherPatch {
        cmd: Some(Some("game run --two".to_owned())),
        ..LauncherPatch::default()
    };
    let back = write_launcher(&mut board, &lead(), "build-2", &patch, &ctx()).unwrap();
    assert!(back.revived);
    assert_eq!(board.launchers["build-2"].retired_by_item, None);
}
