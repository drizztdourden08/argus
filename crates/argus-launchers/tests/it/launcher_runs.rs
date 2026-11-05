//! Builds made for a test run: the write rules (a build serves its run, so it has no usage
//! window and is never stale), retire, and the master rule for handing a run to the owner.
//!
//! Keep rule: K1, K4.
// test-tag: critical

use argus_launchers::launcher::{
    Launcher, LauncherBoard, LauncherPatch, WriteContext, is_standalone, require_master,
    retire_launcher, write_launcher,
};

use crate::launcher_rules::{AT, board, worker};

fn for_run(cmd: &str) -> LauncherPatch {
    LauncherPatch {
        cmd: Some(Some(cmd.to_owned())),
        for_item: Some(Some("I-9".to_owned())),
        ..LauncherPatch::default()
    }
}

fn made_for_run(board: &mut LauncherBoard, name: &str) {
    let ctx = WriteContext {
        cwd: "/p",
        bundle: None,
        at: AT,
    };
    write_launcher(board, &worker("worker"), name, &for_run("node r.js"), &ctx).unwrap();
}

#[test]
fn a_build_made_for_a_run_serves_it_and_has_no_usage_window() {
    let mut board = board();
    made_for_run(&mut board, "r-1");
    let rec: &Launcher = &board.launchers["r-1"];
    assert_eq!(rec.for_item.as_deref(), Some("I-9"));
    assert_eq!(rec.bundle, None);
    assert!(rec.use_every.is_none());
    assert!(!is_standalone(&board, rec));
    assert!(board.run_defs.contains_key("launcher-r-1"));
}

#[test]
fn retiring_a_build_made_for_a_run_drops_its_play_and_leaves_master_alone() {
    let mut board = board();
    made_for_run(&mut board, "r-1");
    retire_launcher(&mut board, &worker("worker"), "r-1", Some("run ended"), AT).unwrap();
    let rec = &board.launchers["r-1"];
    assert_eq!(rec.retire_note.as_deref(), Some("run ended"));
    assert!(rec.retired_at.is_some() && !rec.current);
    assert!(!board.run_defs.contains_key("launcher-r-1"));
    assert!(board.launchers["master"].retired_at.is_none());
}

#[test]
fn a_run_is_handed_only_while_the_project_has_a_master() {
    let mut board = board();
    assert!(require_master(&board).is_ok());
    board.launchers.remove("master");
    let refused = require_master(&board).unwrap_err().to_string();
    assert!(refused.contains("no master launcher, so nothing can be handed to the owner yet"));
    assert!(refused.contains("Set master first: coord launcher set master --cmd"));
}
