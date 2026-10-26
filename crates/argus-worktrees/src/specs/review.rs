//! What the safety review found: each case fails without its fix. A batch job never deletes what a scan turned into a
//! worktree, a prune forgets only the records Argus vetted, a nested repository or a huge untracked tree is not lost
//! silently, a moved branch tip is never deleted, and a merged pull request with no head proves nothing.

use crate::batch::BatchInput;
use crate::board::Caller;
use crate::error::ErrorCode;
use crate::exec::GitOutput;
use crate::inspect::{ScanCtx, inspect_dir};
use crate::jobs::JobState;
use crate::rule::Rule;
use crate::testkit::{
    Fixture, Harness, RemoveMode, board_with, branches, commit, exists_itself, git_in, id, listed,
    make_link, owner_now, rule_for, started, write,
};
use std::fs;

fn become_worktree(h: &Harness, fx: &Fixture, name: &str) {
    let rule = rule_for(fx);
    let mut ctx = ScanCtx {
        git: &h.svc.deps.git,
        fs: h.svc.deps.fs.as_ref(),
        lists: Vec::new(),
        case_insensitive: cfg!(windows),
    };
    let fresh = inspect_dir(&fx.path(name), &rule, &mut ctx);
    let mut state = h.svc.st();
    let facts = &mut state.scans.get_mut("alpha").unwrap().facts;
    let at = facts.iter().position(|f| f.id == fresh.id).unwrap();
    facts[at] = fresh;
}

#[test]
fn a_batch_job_whose_row_a_scan_turned_into_a_git_worktree_is_refused_and_nothing_is_deleted() {
    let (fx, h) = started(|fx| fs::create_dir_all(fx.path("junk")).unwrap());
    assert_eq!(h.docs()["junk"].status, crate::doc::Status::Leftover);
    let hold = h.svc.one_at_a_time();
    let input = BatchInput {
        older_than_days: Some(0.0),
        confirm_young: true,
        ..BatchInput::new("alpha", Caller::Page)
    };
    h.svc.remove_leftovers(&input).unwrap();
    fx.git(&["worktree", "add", "-q", "--detach", &fx.path("junk")]);
    become_worktree(&h, &fx, "junk");
    drop(hold);
    h.idle();
    let job = h.job_ending("junk");
    assert_eq!(job.state, JobState::Refused);
    assert!(
        job.error
            .unwrap()
            .contains("is not a leftover folder any more")
    );
    assert!(
        exists_itself(&format!("{}/.git", fx.path("junk"))),
        "the worktree is still there"
    );
    assert!(listed(&fx.repo).contains(&"junk".to_string()));
}

#[test]
fn a_finish_of_a_crashed_deletion_under_a_folder_that_is_a_rules_own_folder_is_refused() {
    let fx = Fixture::new();
    write(&fx.wts, "crashed.argus-deleting-1730000000001/a.txt", "x");
    let rules = vec![
        rule_for(&fx),
        Rule::folder("W-2", &fx.path("crashed"), "worker"),
    ];
    let h = Harness::new(&fx.root, rules, |_| {});
    h.scan();
    h.idle();
    assert!(
        exists_itself(&fx.path("crashed.argus-deleting-1730000000001")),
        "it is the folder of another rule: not deleted"
    );
    assert_eq!(h.svc.jobs("alpha").len(), 0);
}

#[test]
fn a_gone_worktree_on_a_detached_head_no_ref_keeps_needs_discard_and_the_owners_say_to_prune() {
    let (fx, h) = {
        let fx = Fixture::new();
        let path = fx.path("loose");
        fx.git(&["worktree", "add", "-q", "--detach", &path, "main"]);
        commit(&path, "loose.txt", "on no branch\n", 10.0);
        fs::remove_dir_all(&path).unwrap();
        let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
        h.scan();
        (fx, h)
    };
    let target = id(&h, &fx, "loose");
    let plain = h.refused(&target, |_| {});
    assert_eq!(plain.code, ErrorCode::Forbidden);
    assert!(
        plain.message.contains(
            "is kept by no branch, tag or remote: pruning forgets the only record of those commits"
        ),
        "{}",
        plain.message
    );
    let no_entry = h.refused(&target, |i| i.discard = true);
    assert!(
        no_entry.message.contains("needs the owner"),
        "{}",
        no_entry.message
    );
    assert!(listed(&fx.repo).contains(&"loose".to_string()));
    h.board()
        .entries
        .insert("O-n".into(), owner_now("O-n", "delete loose"));
    let out = h
        .rm_with(&target, |i| {
            (i.discard, i.owner_entry) = (true, Some("O-n".into()));
        })
        .unwrap();
    assert!(out.pruned_only);
    assert!(!listed(&fx.repo).contains(&"loose".to_string()));
}

#[test]
fn a_prune_refuses_when_git_would_also_forget_a_record_argus_did_not_vet() {
    let fx = Fixture::new();
    let outside = format!("{}/elsewhere/other", fx.root);
    fx.git(&["worktree", "add", "-q", "-b", "other", &outside]);
    fs::remove_dir_all(&outside).unwrap();
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    let err = h.refused(&id(&h, &fx, "gone"), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message
            .contains("would also forget 1 other worktree record that Argus has not vetted"),
        "{}",
        err.message
    );
    assert!(
        listed(&fx.repo).contains(&"gone".to_string())
            && listed(&fx.repo).contains(&"other".to_string())
    );
}

#[test]
fn a_worktree_folder_that_is_a_link_is_refused() {
    let (fx, h) = started(|_| {});
    let target = id(&h, &fx, "old-merged");
    let real = fx.path("old-merged");
    let moved = format!("{}/moved-away", fx.root);
    fs::rename(&real, &moved).unwrap();
    make_link(&real, &moved);
    let err = h.refused(&target, |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message
            .contains("is a link or junction, not a worktree folder"),
        "{}",
        err.message
    );
    assert!(exists_itself(&format!("{moved}/.git")));
}

fn nested_repo_in_an_ignored_folder(fx: &Fixture) {
    fs::write(format!("{}/.git/info/exclude", fx.repo), "vendor\n").unwrap();
    write(
        &fx.path("old-merged"),
        "vendor/lib/.git/HEAD",
        "ref: refs/heads/main\n",
    );
    write(&fx.path("old-merged"), "vendor/lib/a.txt", "real work\n");
}

#[test]
fn a_nested_repository_is_not_safe_it_needs_discard_and_is_copied_into_the_backup() {
    let (fx, h) = started(nested_repo_in_an_ignored_folder);
    assert_eq!(
        h.docs()["old-merged"].dirty,
        0,
        "ignored: git sees nothing to commit"
    );
    let target = id(&h, &fx, "old-merged");
    let err = h.refused(&target, |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message.contains("a nested git repository inside it ("),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("old-merged")));
    let out = h
        .rm_with(&target, |i| (i.discard, i.caller) = (true, Caller::Page))
        .unwrap();
    let backup = out.backup.unwrap();
    let copy = backup.nested_copy.unwrap();
    assert_eq!(
        fs::read_to_string(format!("{copy}/a.txt")).unwrap(),
        "real work\n"
    );
}

#[test]
fn untracked_files_the_backup_can_only_list_are_said_so_in_the_loss_sentence() {
    let (fx, h) = started(|fx| {
        for i in 0..2001 {
            write(&fx.path("old-unmerged"), &format!("many/f{i}.txt"), "x");
        }
    });
    let err = h.refused(&id(&h, &fx, "old-unmerged"), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message.contains(
            "its untracked files (2001 files, 0 MB) are only listed in the backup, not copied"
        ),
        "{}",
        err.message
    );
}

#[test]
fn a_branch_whose_tip_moved_after_the_decision_is_not_deleted() {
    let fx = Fixture::new();
    let h = Harness::with_mode(
        &fx.root,
        board_with(vec![rule_for(&fx)]),
        RemoveMode::Leaves,
        |_| {},
    );
    h.scan();
    *h.exec.on_remove.lock().unwrap() = Some(Box::new(move |p: &str| {
        commit(p, "late.txt", "work added after the decision\n", 0.0);
    }));
    let out = h
        .rm_with(&id(&h, &fx, "old-merged"), |i| i.branch = true)
        .unwrap();
    assert!(out.removed && !out.deleted_branch, "{}", out.text);
    assert!(
        out.branch_note
            .unwrap()
            .contains("it is no longer the commit that was judged")
    );
    assert!(
        branches(&fx.repo).contains(&"old-merged".to_string()),
        "the new commit is still in its branch"
    );
}

#[test]
fn a_merged_pull_request_that_names_no_head_is_not_proof_of_a_merge() {
    let fx = Fixture::new();
    git_in(
        &fx.repo,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/acme/proj.git",
        ],
        &[],
    );
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    *h.gh.answer.lock().unwrap() = GitOutput {
        code: Some(0),
        stdout: "[{\"number\":7}]".into(),
        ..GitOutput::default()
    };
    let out = h
        .rm_with(&id(&h, &fx, "old-unmerged"), |i| i.branch = true)
        .unwrap();
    assert!(!out.deleted_branch, "{}", out.text);
    assert!(branches(&fx.repo).contains(&"old-unmerged".to_string()));
    assert!(!exists_itself(&fx.path("old-unmerged")));
}
