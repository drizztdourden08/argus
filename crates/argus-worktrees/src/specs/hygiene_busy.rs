//! KEEP: K3, K4.
//! Worktree hygiene, I-162: a command that creates a worktree and works in it for over a minute (an install and an
//! upgrade, 75 s) is never taken for an unattached one. A process counts when its working directory or its command
//! line names the folder, and a worktree whose files changed in the last 10 minutes is busy.
//! I-178: a commit or `git add` writes the git index and files deep in the tree, so a recent index or last commit
//! keeps a worktree busy even when its top-level files are older than the grace.

// test-tag: critical

use crate::hygiene::HygieneState;
use crate::ports::ProcInfo;
use crate::specs::hygiene_kit::{
    agent, agents_rule, attached_why, hygiene_on, kind, step, through_debounce, world,
};
use crate::sync::locked;
use crate::testkit::{Fixture, Harness, exists_itself};

const GRACE: i64 = 10 * 60_000;
const UPGRADE_MS: i64 = 75_000;

/// A worktree of a done agent, the hygiene on with the busy grace, scanned and started.
fn world_with_grace() -> (Fixture, Harness) {
    let fx = Fixture::fake_bare();
    fx.add("agent-a1");
    let h = Harness::new(&fx.root, vec![agents_rule(&fx)], |o| {
        hygiene_on(o);
        o.busy_grace_ms = GRACE;
    });
    let ended = h.clock_now() - 1000;
    h.board().agents = vec![agent("1", "lead", "done", ended)];
    h.scan();
    h.svc.start();
    (fx, h)
}

fn running(h: &Harness, cmd: &str, cwd: Option<&str>) {
    locked(&h.procs.0).push(ProcInfo {
        pid: 4242,
        name: "bun.exe".into(),
        cmd: cmd.into(),
        cwd: cwd.map(String::from),
        ..ProcInfo::default()
    });
}

#[test]
fn a_75_second_upgrade_in_a_fresh_worktree_is_not_removed() {
    let (fx, h) = world_with_grace();
    let path = fx.path("agent-a1");
    step(&h, UPGRADE_MS);
    h.svc.requests_idle();
    assert!(exists_itself(&path), "75 s into the upgrade it must stay");
    let why = attached_why(&h, "agent-a1");
    assert!(why.contains("busy"), "{why}");
}

#[test]
fn the_debounce_starts_after_the_grace() {
    let (fx, h) = world_with_grace();
    let path = fx.path("agent-a1");
    step(&h, GRACE + 1000);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    assert!(exists_itself(&path));
    assert!(through_debounce(&h, &path));
}

#[test]
fn a_recent_git_index_or_commit_keeps_a_worktree_busy() {
    let (_fx, h) = world_with_grace();
    step(&h, GRACE + 1000);
    let mut fact = h.fact("agent-a1");
    (fact.index_ms, fact.commit_ms) = (None, None);
    assert_eq!(h.svc.busy_why(&fact), None, "files past the grace");
    fact.index_ms = Some(h.clock_now() - 7000);
    let why = h.svc.busy_why(&fact).expect("a recent index is busy");
    assert!(why.contains("busy") && why.contains("git index"), "{why}");
    (fact.index_ms, fact.commit_ms) = (None, Some(h.clock_now() - 7000));
    let why = h.svc.busy_why(&fact).expect("a recent commit is busy");
    assert!(why.contains("busy") && why.contains("last commit"), "{why}");
}

#[test]
fn a_process_counts_by_its_command_line_when_its_cwd_cannot_be_read() {
    let (fx, h) = world(|_| {});
    let upper = fx.path("agent-a1").replace('/', "\\").to_uppercase();
    h.clock.advance(10_001);
    running(&h, &format!("bun upgrade --dir {upper}\\web"), None);
    h.scan();
    assert!(attached_why(&h, "agent-a1").contains("4242"));
    assert!(!through_debounce(&h, &fx.path("agent-a1")));
}

#[test]
fn a_process_counts_by_its_cwd_when_its_command_line_names_nothing() {
    let (fx, h) = world(|_| {});
    let inside = format!("{}/packages/web", fx.path("agent-a1")).replace('/', "\\");
    h.clock.advance(10_001);
    running(&h, "pnpm install", Some(&inside));
    h.scan();
    assert!(attached_why(&h, "agent-a1").contains("4242"));
    assert!(!through_debounce(&h, &fx.path("agent-a1")));
}
