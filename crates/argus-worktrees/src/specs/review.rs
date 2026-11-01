//! KEEP: K3, K4.
//! What the safety review found: each case fails without its fix. A batch job never deletes what a scan turned into a
//! worktree, a prune forgets only the records Argus vetted, a nested repository or a huge untracked tree is not lost
//! silently, a moved branch tip is never deleted, and a merged pull request with no head proves nothing.

// test-tag: standard

use crate::batch::BatchInput;
use crate::board::Caller;
use crate::error::ErrorCode;
use crate::inspect::{ScanCtx, inspect_dir};
use crate::jobs::JobState;
use crate::rule::Rule;
use crate::testkit::{
    Fixture, Harness, RemoveMode, board_with, branches, commit, exists_itself, id, listed,
    make_link, pull_request_answer_keeps_branch, rule_for, started, write,
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
    let hold = h.svc.one_at_a_time("alpha", "a test", None);
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
    let fx = Fixture::fake();
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
    let fx = Fixture::fake();
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
    let fx = pull_request_answer_keeps_branch("[{\"number\":7}]");
    assert!(!exists_itself(&fx.path("old-unmerged")));
}
