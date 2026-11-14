//! KEEP: K3, K4.
//! The class a worktree doc shows, and the two removals that lose nothing: a worktree that is merged and clean goes
//! after the debounce, so does one on a detached HEAD whose commit is merged into main. The class is display only: a
//! merged worktree with untracked files, a locked one and one holding work still wait for their lead.

// test-tag: standard

use crate::class::{Class, LockKind};
use crate::hygiene::HygieneState;
use crate::specs::hygiene_kit::{
    agent, agents_rule, hygienic, kind, left_why, through_debounce, world, world_alive,
};
use crate::testkit::{Fixture, Harness, commit, put, rule_for};

/// The class and the lock kind of a doc.
fn class_of(h: &Harness, name: &str) -> (Class, Option<LockKind>) {
    let doc = &h.docs()[name];
    (doc.class, doc.lock_kind)
}

/// A world whose only worktree, `agent-a1`, is on a detached HEAD at `main`, with its agent done.
fn detached_world(prep: impl FnOnce(&str)) -> (Fixture, Harness) {
    let fx = Fixture::fake();
    let path = fx.path("agent-a1");
    fx.git(&["worktree", "add", "-q", "--detach", &path, "main"]);
    prep(&path);
    let h = hygienic(&fx, agents_rule(&fx));
    h.board().agents = vec![agent("1", "lead", "done", h.clock_now() - 1000)];
    h.scan();
    h.svc.start();
    (fx, h)
}

#[test]
fn the_doc_shows_the_class_of_each_worktree_of_the_fixture() {
    let fx = Fixture::fake();
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    let class = |name| class_of(&h, name).0;
    assert_eq!(class("fresh"), Class::Clean);
    assert_eq!(class("old-unmerged"), Class::Clean);
    assert_eq!(class("old-merged"), Class::Merged);
    assert_eq!(class("dirty-old"), Class::MergedUntracked);
    assert_eq!(class("plain"), Class::NotWorktree);
    let dirty = &h.docs()["dirty-old"];
    assert_eq!((dirty.dirty, dirty.untracked), (1, 1));
}

#[test]
fn a_lock_shows_whose_it_is_and_still_waits_for_the_lead_unless_it_is_brocks() {
    let lock = |reason: &'static str| {
        world_alive(&[4242], move |fx| {
            fx.git(&["worktree", "lock", "--reason", reason, &fx.path("agent-a1")]);
        })
    };
    let (_fx, h) = lock("owner: hands off");
    assert_eq!(
        class_of(&h, "agent-a1"),
        (Class::Locked, Some(LockKind::Other))
    );
    assert_eq!(left_why(&h, "agent-a1"), "it is locked (owner: hands off)");
    let (_fx, h) = lock("claude agent agent-a1 (pid 4242)");
    assert_eq!(
        class_of(&h, "agent-a1"),
        (Class::Locked, Some(LockKind::Claude))
    );
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::AwaitingLead));
    let (_fx, h) = lock("cloned node_modules from X:/brock");
    assert_eq!(
        class_of(&h, "agent-a1"),
        (Class::Locked, Some(LockKind::Brock))
    );
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
}

#[test]
fn a_merged_worktree_with_untracked_files_is_merged_untracked_and_still_asks_the_lead() {
    let (fx, h) = world(|fx| {
        put(&fx.path("agent-a1"), "notes.txt", "not committed\n");
    });
    let doc = &h.docs()["agent-a1"];
    assert_eq!(
        (doc.class, doc.merged, doc.dirty, doc.untracked),
        (Class::MergedUntracked, true, 1, 1)
    );
    assert!(left_why(&h, "agent-a1").contains("1 uncommitted change"));
    assert!(!through_debounce(&h, &fx.path("agent-a1")));
}

/// The worktree `agent-a1` is merged, will be removed, and goes through the debounce.
fn assert_merged_and_removable(fx: &Fixture, h: &Harness) {
    assert_eq!(class_of(h, "agent-a1"), (Class::Merged, None));
    assert_eq!(kind(h, "agent-a1"), Some(HygieneState::Removing));
    assert!(through_debounce(h, &fx.path("agent-a1")));
}

#[test]
fn merged_and_clean_is_removable_after_the_debounce() {
    let (fx, h) = world(|_| {});
    assert_merged_and_removable(&fx, &h);
}

#[test]
fn a_detached_head_merged_into_main_is_merged_and_removable() {
    let (fx, h) = detached_world(|_| {});
    let doc = &h.docs()["agent-a1"];
    assert_eq!((doc.detached, doc.merged), (true, true));
    assert_merged_and_removable(&fx, &h);
}

#[test]
fn a_detached_head_with_a_commit_no_ref_keeps_is_work_left_and_stays() {
    let (fx, h) = detached_world(|path| commit(path, "d.txt", "detached work\n", 0.0));
    assert_eq!(class_of(&h, "agent-a1"), (Class::WorkLeft, None));
    assert!(left_why(&h, "agent-a1").contains("no branch or ref keeps"));
    assert!(!through_debounce(&h, &fx.path("agent-a1")));
}
