//! Worktree safety round 2 (Node 62e66c7): a prune forgets only vetted records, a merged pull request needs its head, a
//! nested repository is unsafe and copied into the backup, and a git status that failed is unknown, never clean.

use crate::board::Caller;
use crate::error::{ErrorCode, WtError};
use crate::exec::GitOutput;
use crate::request::RemoveInput;
use crate::testkit::{
    Fixture, Harness, branches, commit, exists_itself, git_in, id, listed, owner_now, records,
    rule_for, started, unplugged, write,
};
use std::fs;

fn prune_request() -> RemoveInput {
    RemoveInput::new("alpha", "", Caller::Thread("lead".into()))
}

fn prune_all_refused(h: &Harness) -> WtError {
    h.svc.prune(&prune_request()).unwrap_err()
}

#[test]
fn a_prunable_record_no_rule_vetted_stops_the_prune_by_id_and_for_all_nothing_pruned() {
    let (fx, h) = started(unplugged(true));
    let before = records(&fx);
    assert!(before.contains(&"drive-x".to_string()) && before.contains(&"gone".to_string()));
    let one = h.refused(&id(&h, &fx, "gone"), |_| {});
    assert_eq!(one.code, ErrorCode::Forbidden);
    assert!(
        one.message
            .contains("would also forget 1 other worktree record")
            && one.message.contains("drive-x")
            && one.message.contains("nothing was pruned"),
        "{}",
        one.message
    );
    let all = prune_all_refused(&h);
    assert_eq!(all.code, ErrorCode::Forbidden);
    assert!(all.message.contains("drive-x"), "{}", all.message);
    assert_eq!(
        records(&fx),
        before,
        "git still knows every record, the detached commit stays reachable"
    );
}

#[test]
fn with_only_vetted_records_the_prune_still_works() {
    let (fx, h) = started(|_| {});
    let out = h.svc.prune(&prune_request()).unwrap();
    assert_eq!((out.pruned_only, out.pruned.len()), (true, 1));
    assert!(!records(&fx).contains(&"gone".to_string()));
}

fn gone_loose(fx: &Fixture) {
    let path = fx.path("lost-gone");
    fx.git(&["worktree", "add", "-q", "--detach", &path, "main"]);
    commit(&path, "lost.txt", "only here\n", 4.0);
    fs::remove_dir_all(&path).unwrap();
}

#[test]
fn a_gone_worktree_on_a_detached_head_no_ref_keeps_needs_discard_and_the_owners_say_by_id_and_for_all()
 {
    let (fx, h) = started(gone_loose);
    let target = id(&h, &fx, "lost-gone");
    let before = records(&fx);
    let bare = h.refused(&target, |_| {});
    assert_eq!(bare.code, ErrorCode::Forbidden);
    assert!(
        bare.message
            .contains("is on a detached HEAD that no branch keeps"),
        "{}",
        bare.message
    );
    let no_owner = h.refused(&target, |i| i.discard = true);
    assert!(
        no_owner
            .message
            .contains("needs the owner: no owner entry was given"),
        "{}",
        no_owner.message
    );
    assert_eq!(prune_all_refused(&h).code, ErrorCode::Forbidden);
    assert_eq!(records(&fx), before, "nothing was pruned");
    h.board().entries.insert(
        "O-p".into(),
        owner_now("O-p", "delete lost-gone, I want it gone"),
    );
    let out = h
        .rm_with(&target, |i| {
            (i.discard, i.owner_entry) = (true, Some("O-p".into()));
        })
        .unwrap();
    assert!(out.removed && out.pruned_only);
    assert!(!records(&fx).contains(&"lost-gone".to_string()));
}

fn nested_at(rel: &'static str) -> impl FnOnce(&Fixture) {
    move |fx| {
        fs::write(format!("{}/.git/info/exclude", fx.repo), "vendor/\ndeep/\n").unwrap();
        let folder = fx.path("old-merged");
        write(
            &folder,
            &format!("{rel}/.git/HEAD"),
            "ref: refs/heads/main\n",
        );
        write(
            &folder,
            &format!("{rel}/precious.txt"),
            "a repository somebody cloned in here\n",
        );
    }
}

#[test]
fn an_ignored_nested_repository_is_refused_without_discard_then_removed_with_the_pages_say_and_copied()
 {
    let (fx, h) = started(nested_at("vendor/lib"));
    assert_eq!(
        h.docs()["old-merged"].dirty,
        0,
        "git calls it clean: it is ignored"
    );
    let target = id(&h, &fx, "old-merged");
    let err = h.refused(&target, |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message.contains("a nested git repository (vendor/lib)")
            && err.message.contains("removing it discards them"),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("old-merged")), "nothing was removed");
    let out = h
        .rm_with(&target, |i| (i.discard, i.caller) = (true, Caller::Page))
        .unwrap();
    assert!(out.removed);
    let copy = out.backup.unwrap().nested_copy.unwrap();
    assert!(exists_itself(&format!("{copy}/precious.txt")));
    assert!(!exists_itself(&fx.path("old-merged")));
}

#[test]
fn three_levels_down_still_counts_and_a_thread_needs_the_owners_entry() {
    let (fx, h) = started(nested_at("deep/a/b"));
    let err = h.refused(&id(&h, &fx, "old-merged"), |i| i.discard = true);
    assert!(
        err.message.contains("a nested git repository (deep/a/b)")
            && err.message.contains("needs the owner"),
        "{}",
        err.message
    );
}

#[test]
fn without_any_nested_repository_a_clean_merged_worktree_goes_with_no_backup() {
    let (fx, h) = started(|_| {});
    let out = h.rm(&id(&h, &fx, "old-merged")).unwrap();
    assert!(out.removed && out.backup.is_none());
}

#[test]
fn a_failed_git_status_is_unknown_never_clean_the_removal_is_refused_and_the_folder_stays() {
    let (fx, h) = started(|_| {});
    *h.exec.fail_status.lock().unwrap() = true;
    let err = h.refused(&id(&h, &fx, "old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::GitFailed);
    assert!(
        err.message.contains("could not read the state of")
            && err.message.contains("unable to read the index")
            && err.message.contains("nothing was removed"),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("old-merged")));
    assert!(listed(&fx.repo).contains(&"old-merged".to_string()));
}

#[test]
fn a_merged_pull_request_whose_head_is_not_a_commit_id_proves_nothing() {
    let fx = Fixture::new();
    let remote = [
        "remote",
        "add",
        "origin",
        "https://github.com/acme/proj.git",
    ];
    git_in(&fx.repo, &remote, &[]);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    *h.gh.answer.lock().unwrap() = GitOutput {
        code: Some(0),
        stdout: "[{\"number\":7,\"headRefOid\":\"--oops\"}]".into(),
        ..GitOutput::default()
    };
    let out = h
        .rm_with(&id(&h, &fx, "old-unmerged"), |i| i.branch = true)
        .unwrap();
    assert!(!out.deleted_branch, "{}", out.text);
    assert!(branches(&fx.repo).contains(&"old-unmerged".to_string()));
}
