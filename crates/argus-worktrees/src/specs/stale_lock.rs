//! I-163: a registered worktree whose folder lost its `.git` file and whose Claude Code lock is stale goes the way a
//! leftover goes; a live pid or a link in the folder keeps it.

// test-tag: critical

use crate::error::ErrorCode;
use crate::ports::ProcInfo;
use crate::testkit::{
    Fixture, Harness, backdate, exists_itself, listed, make_link, put, root_of, rule_for,
};
use std::fs;
use tempfile::tempdir;

const LOCK: &str = "claude agent agent-a37f1556 (pid 39176)";

fn world(link: bool) -> (Fixture, Harness) {
    let fx = Fixture::fake();
    let path = fx.add("agent-a37");
    fx.git(&["worktree", "lock", "--reason", LOCK, &path]);
    fs::remove_file(format!("{path}/.git")).unwrap();
    put(&path, "CLAUDE.md", "rendered\n");
    if link {
        let outside = tempdir().unwrap();
        put(&root_of(&outside), "keep.txt", "precious");
        make_link(&format!("{path}/lnk"), &root_of(&outside));
        std::mem::forget(outside);
    }
    backdate(&path, 10.0);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    (fx, h)
}

fn alive(h: &Harness) {
    *h.procs.0.lock().unwrap() = vec![ProcInfo {
        pid: 39176,
        name: "claude.exe".into(),
        ..ProcInfo::default()
    }];
}

#[test]
fn a_dead_pid_lock_on_a_folder_without_git_is_unlocked_deleted_and_pruned() {
    let (fx, h) = world(false);
    let path = fx.path("agent-a37");
    let out = h.rm(&h.id_of(&path)).unwrap();
    assert!(out.started, "{}", out.text);
    h.idle();
    assert!(!exists_itself(&path), "the folder is gone");
    assert_eq!(
        listed(&fx.repo)
            .iter()
            .filter(|n| n.contains("agent-a37"))
            .count(),
        0
    );
}

#[test]
fn a_live_pid_lock_refuses_and_names_the_pid() {
    let (fx, h) = world(false);
    alive(&h);
    let path = fx.path("agent-a37");
    let err = h.refused(&h.id_of(&path), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message.contains("pid 39176, which is running"),
        "{}",
        err.message
    );
    assert!(exists_itself(&path));
    assert!(
        fx.git(&["worktree", "list", "--porcelain"])
            .contains("locked claude agent")
    );
}

#[test]
fn a_folder_with_a_link_refuses_and_keeps_the_lock_and_the_target() {
    let (fx, h) = world(true);
    let path = fx.path("agent-a37");
    let err = h.refused(&h.id_of(&path), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message.contains("holds a link or junction"),
        "{}",
        err.message
    );
    assert!(exists_itself(&format!("{path}/lnk")));
    assert!(
        fx.git(&["worktree", "list", "--porcelain"])
            .contains("locked claude agent")
    );
}
