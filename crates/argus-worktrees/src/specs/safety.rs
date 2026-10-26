//! The safety model of removing a worktree: removing the FOLDER of a clean worktree whose HEAD a branch (or another ref)
//! keeps loses nothing, so it needs no approval, merged or not. Deleting the BRANCH is its own decision: merged by
//! ancestry into origin's default or the LOCAL default, patch-equivalent, or its GitHub pull request merged; otherwise
//! kept. Only uncommitted changes and commits on a detached HEAD that no ref keeps are real losses. `gh` is never the
//! real one here: a fake runner answers, and absent or failing means "not merged".

use crate::board::Caller;
use crate::error::ErrorCode;
use crate::exec::GitOutput;
use crate::request::Discarded;
use crate::testkit::{
    Fixture, Harness, backdate_index, branches, commit, exists_itself, git_in, listed, owner_now,
    started, write,
};

fn setup(prep: impl FnOnce(&Fixture)) -> (Fixture, Harness) {
    started(prep)
}

fn id(h: &Harness, fx: &Fixture, name: &str) -> String {
    crate::testkit::id(h, fx, name)
}

fn tip_of(fx: &Fixture, branch: &str) -> String {
    fx.git(&["rev-parse", &format!("refs/heads/{branch}")])
        .trim()
        .to_string()
}

#[test]
fn a_clean_worktree_on_an_unmerged_branch_goes_with_no_approval_no_backup_and_the_branch_is_kept() {
    let (fx, h) = setup(|_| {});
    let row = h.docs()["old-unmerged"].clone();
    assert_eq!(
        (row.merged, row.referenced, row.dirty, row.unmerged),
        (false, Some(true), 0, Some(1))
    );
    let head = fx.git(&["rev-parse", "old-unmerged"]).trim().to_string();
    let out = h
        .rm_with(&id(&h, &fx, "old-unmerged"), |i| i.branch = true)
        .unwrap();
    assert_eq!(
        (
            out.removed,
            out.deleted_branch,
            out.branch_note.as_deref(),
            out.backup.is_none(),
            out.discarded
        ),
        (true, false, Some("not merged"), true, None)
    );
    assert!(
        out.text.starts_with("removed worktree ")
            && out
                .text
                .ends_with("old-unmerged, branch old-unmerged kept (not merged)"),
        "{}",
        out.text
    );
    assert!(
        !exists_itself(&fx.path("old-unmerged")),
        "the folder is gone"
    );
    assert!(!listed(&fx.repo).contains(&"old-unmerged".to_string()));
    assert!(
        branches(&fx.repo).contains(&"old-unmerged".to_string()),
        "the branch stays"
    );
    assert_eq!(
        fx.git(&["rev-parse", "old-unmerged"]).trim(),
        head,
        "every commit is still in the branch"
    );
    assert_eq!(h.notes.removals("removed worktree")[0].level, "info");
}

#[test]
fn without_branch_the_branch_is_simply_left() {
    let (fx, h) = setup(|_| {});
    let out = h.rm(&id(&h, &fx, "old-unmerged")).unwrap();
    assert_eq!(
        (
            out.removed,
            out.deleted_branch,
            out.branch_note,
            out.backup.is_none()
        ),
        (true, false, None, true)
    );
    assert!(out.text.ends_with("branch old-unmerged kept"));
}

#[test]
fn discarding_the_unmerged_branch_is_the_owners_decision_refused_before_anything_is_removed_then_a_bundle_first()
 {
    let (fx, h) = setup(|_| {});
    h.board().entries.insert(
        "O-n".into(),
        owner_now("O-n", "delete old-unmerged and its branch"),
    );
    let target = id(&h, &fx, "old-unmerged");
    let err = h.refused(&target, |i| (i.branch, i.discard) = (true, true));
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message
            .starts_with("discarding the commits of branch old-unmerged that are not merged in ")
            && err
                .message
                .contains("needs the owner: no owner entry was given"),
        "{}",
        err.message
    );
    assert!(
        exists_itself(&fx.path("old-unmerged")),
        "nothing was removed"
    );
    let out = h
        .rm_with(&target, |i| {
            (i.branch, i.discard, i.owner_entry) = (true, true, Some("O-n".into()));
        })
        .unwrap();
    assert_eq!((out.removed, out.deleted_branch), (true, true));
    let backup = out.backup.unwrap();
    assert!(backup.bundle.as_deref().is_some_and(exists_itself) && backup.patch.is_none());
    assert!(!branches(&fx.repo).contains(&"old-unmerged".to_string()));
}

#[test]
fn a_dirty_worktree_is_not_removable_without_the_discard_rules_even_on_a_merged_branch() {
    let (fx, h) = setup(|fx| write(&fx.path("old-unmerged"), "scratch.txt", "not committed\n"));
    let err = h.refused(&id(&h, &fx, "old-unmerged"), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message
            .contains("has 1 uncommitted change: removing it discards them")
    );
    let merged = h.refused(&id(&h, &fx, "dirty-old"), |_| {});
    assert!(merged.message.contains("has 1 uncommitted change"));
    assert!(exists_itself(&fx.path("old-unmerged")) && exists_itself(&fx.path("dirty-old")));
    let page = h
        .rm_with(&id(&h, &fx, "old-unmerged"), |i| {
            (i.discard, i.caller) = (true, Caller::Page);
        })
        .unwrap();
    let backup = page.backup.unwrap();
    assert!(
        backup.patch.is_some() && backup.bundle.is_none(),
        "the uncommitted file is what is lost"
    );
    assert!(branches(&fx.repo).contains(&"old-unmerged".to_string()));
}

fn detached_cases(fx: &Fixture) {
    git_in(
        &fx.repo,
        &[
            "worktree",
            "add",
            "-q",
            "--detach",
            &fx.path("at-main"),
            "main",
        ],
        &[],
    );
    let loose = fx.path("loose");
    git_in(
        &fx.repo,
        &["worktree", "add", "-q", "--detach", &loose, "main"],
        &[],
    );
    commit(&loose, "loose.txt", "on no branch\n", 10.0);
    backdate_index(&loose, 10.0);
    let tagged = fx.path("tagged");
    git_in(
        &fx.repo,
        &["worktree", "add", "-q", "--detach", &tagged, "main"],
        &[],
    );
    commit(&tagged, "tagged.txt", "tagged\n", 10.0);
    let sha = git_in(&tagged, &["rev-parse", "HEAD"], &[]);
    git_in(&fx.repo, &["tag", "keep-me", sha.trim()], &[]);
}

#[test]
fn a_detached_head_a_branch_contains_is_safe_one_with_commits_no_ref_keeps_is_not_even_when_clean()
{
    let (fx, h) = setup(detached_cases);
    let d = h.docs();
    assert_eq!(
        (
            d["at-main"].referenced,
            d["loose"].referenced,
            d["tagged"].referenced
        ),
        (Some(true), Some(false), Some(true))
    );
    let err = h.refused(&id(&h, &fx, "loose"), |_| {});
    assert!(err.message.contains(
        "has 1 commit on a detached HEAD that no branch keeps: removing it discards them"
    ));
    assert!(exists_itself(&fx.path("loose")));
    assert!(h.rm(&id(&h, &fx, "at-main")).unwrap().removed);
    assert!(
        h.rm(&id(&h, &fx, "tagged")).unwrap().removed,
        "a tag keeps it"
    );
    let page = h
        .rm_with(&id(&h, &fx, "loose"), |i| {
            (i.discard, i.caller) = (true, Caller::Page);
        })
        .unwrap();
    assert!(
        page.backup
            .unwrap()
            .bundle
            .as_deref()
            .is_some_and(exists_itself)
    );
    assert_eq!(
        page.discarded,
        Some(Discarded {
            dirty: 0,
            unmerged: Some(1)
        })
    );
}

fn squashed(name: &'static str, commits: usize) -> impl FnOnce(&Fixture) {
    move |fx| {
        let path = fx.path(name);
        git_in(&fx.repo, &["worktree", "add", "-q", "-b", name, &path], &[]);
        for i in 0..commits {
            commit(
                &path,
                &format!("{name}-{i}.txt"),
                &format!("{name} work {i}\n"),
                5.0,
            );
        }
        git_in(&fx.repo, &["merge", "--squash", name], &[]);
        git_in(
            &fx.repo,
            &["commit", "-q", "-m", &format!("squash {name}")],
            &[],
        );
    }
}

#[test]
fn one_commit_squashed_the_worktree_goes_and_branch_deletes_the_branch_because_its_changes_are_already_in_main()
 {
    let (fx, h) = setup(squashed("sq", 1));
    let row = h.docs()["sq"].clone();
    assert_eq!(
        (row.merged, row.referenced, row.unmerged),
        (false, Some(true), Some(1)),
        "unmerged by ancestry"
    );
    let out = h.rm_with(&id(&h, &fx, "sq"), |i| i.branch = true).unwrap();
    assert_eq!(
        (
            out.removed,
            out.deleted_branch,
            out.branch_why.as_deref(),
            out.backup.is_none()
        ),
        (
            true,
            true,
            Some("its changes are already in local main (not pushed)"),
            true
        )
    );
    assert!(
        out.text
            .ends_with("branch sq deleted: its changes are already in local main (not pushed)")
    );
    assert!(!branches(&fx.repo).contains(&"sq".to_string()));
    assert!(
        h.gh.calls.lock().unwrap().is_empty(),
        "no remote, so gh was never asked"
    );
}

fn gh_says(h: &Harness, code: i32, stdout: &str) {
    *h.gh.answer.lock().unwrap() = GitOutput {
        code: Some(code),
        stdout: stdout.into(),
        ..GitOutput::default()
    };
}

fn try_again(
    h: &Harness,
    fx: &Fixture,
    seq: &mut u32,
    more: bool,
) -> crate::request::RemoveOutcome {
    *seq += 1;
    let name = format!("try-{seq}");
    git_in(
        &fx.repo,
        &["worktree", "add", "-q", &fx.path(&name), "sq2"],
        &[],
    );
    if more {
        commit(&fx.path(&name), &format!("{name}.txt"), "one more\n", 0.0);
    }
    h.scan();
    h.rm_with(&id(h, fx, &name), |i| i.branch = true).unwrap()
}

fn github_sq2(fx: &Fixture) {
    squashed("sq2", 2)(fx);
    fx.git(&[
        "remote",
        "add",
        "origin",
        "https://github.com/acme/proj.git",
    ]);
}

fn asked_of_gh() -> Vec<String> {
    [
        "pr",
        "list",
        "--state",
        "merged",
        "--head",
        "sq2",
        "--json",
        "number,headRefOid",
        "--limit",
        "1",
    ]
    .map(String::from)
    .to_vec()
}

fn merged_pr(oid: &str) -> String {
    format!("[{{\"number\":12,\"headRefOid\":\"{oid}\"}}]")
}

#[test]
fn several_commits_squashed_into_one_are_not_patch_equivalent_and_an_absent_gh_means_not_merged() {
    let (fx, h) = setup(github_sq2);
    let kept = h.rm_with(&id(&h, &fx, "sq2"), |i| i.branch = true).unwrap();
    assert_eq!((kept.removed, kept.deleted_branch), (true, false));
    assert_eq!(kept.branch_note.as_deref(), Some("not merged"));
    assert!(kept.text.ends_with("branch sq2 kept (not merged)"));
    assert!(branches(&fx.repo).contains(&"sq2".to_string()));
    assert_eq!(h.gh.calls.lock().unwrap()[0], asked_of_gh());
}

#[test]
fn a_failing_gh_garbage_from_gh_and_new_work_after_the_merged_pull_request_are_not_merged() {
    let (fx, h) = setup(github_sq2);
    h.rm_with(&id(&h, &fx, "sq2"), |i| i.branch = true).unwrap();
    let mut seq = 0;
    gh_says(&h, 1, "");
    assert!(
        !try_again(&h, &fx, &mut seq, true).deleted_branch,
        "a failing gh"
    );
    gh_says(&h, 0, "not json");
    assert!(
        !try_again(&h, &fx, &mut seq, true).deleted_branch,
        "garbage from gh"
    );
    gh_says(&h, 0, &merged_pr(&tip_of(&fx, "sq2")));
    assert!(
        !try_again(&h, &fx, &mut seq, true).deleted_branch,
        "new work after the merged pull request"
    );
}

#[test]
fn gh_is_asked_once_per_branch_and_tip_and_the_answer_is_cached() {
    let (fx, h) = setup(github_sq2);
    h.rm_with(&id(&h, &fx, "sq2"), |i| i.branch = true).unwrap();
    let mut seq = 0;
    gh_says(&h, 0, "[]");
    let before = h.gh.calls.lock().unwrap().len();
    assert!(!try_again(&h, &fx, &mut seq, true).deleted_branch);
    assert_eq!(h.gh.calls.lock().unwrap().len(), before + 1);
    gh_says(&h, 0, &merged_pr(&tip_of(&fx, "sq2")));
    assert!(
        !try_again(&h, &fx, &mut seq, false).deleted_branch,
        "still the cached \"not merged\" for that tip"
    );
    assert_eq!(
        h.gh.calls.lock().unwrap().len(),
        before + 1,
        "one gh call per branch and tip"
    );
}

#[test]
fn a_new_tip_that_gh_says_was_merged_deletes_the_branch_with_the_reason() {
    let (fx, h) = setup(|fx| {
        squashed("sq2", 2)(fx);
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
        fx.git(&["worktree", "remove", "--force", &fx.path("sq2")]);
        let path = fx.path("final");
        git_in(&fx.repo, &["worktree", "add", "-q", &path, "sq2"], &[]);
        commit(&path, "final.txt", "tip again\n", 0.0);
    });
    let tip = tip_of(&fx, "sq2");
    gh_says(
        &h,
        0,
        &format!("[{{\"number\":12,\"headRefOid\":\"{tip}\"}}]"),
    );
    let out = h
        .rm_with(&id(&h, &fx, "final"), |i| i.branch = true)
        .unwrap();
    assert_eq!(
        (out.removed, out.deleted_branch, out.branch_why.as_deref()),
        (true, true, Some("its pull request is merged"))
    );
    assert!(
        out.text
            .ends_with("branch sq2 deleted: its pull request is merged")
    );
    assert!(!branches(&fx.repo).contains(&"sq2".to_string()));
}

#[test]
fn gh_is_only_asked_when_the_remote_is_github() {
    let (fx, h) = setup(|fx| {
        squashed("sq3", 2)(fx);
        git_in(
            &fx.repo,
            &[
                "remote",
                "add",
                "origin",
                "https://git.example.test/acme/proj.git",
            ],
            &[],
        );
    });
    gh_says(&h, 0, "[{\"number\":1}]");
    let out = h.rm_with(&id(&h, &fx, "sq3"), |i| i.branch = true).unwrap();
    assert_eq!(
        (
            out.deleted_branch,
            out.branch_note.as_deref(),
            h.gh.calls.lock().unwrap().len()
        ),
        (false, Some("not merged"), 0)
    );
}

fn like_rotp(fx: &Fixture) {
    fx.git(&["branch", "-m", "main", "master"]);
    let bare = format!("{}/origin.git", fx.root);
    git_in(&fx.root, &["init", "-q", "--bare", &bare], &[]);
    fx.git(&["remote", "add", "origin", &bare]);
    fx.git(&["push", "-q", "origin", "master"]);
    fx.git(&["remote", "set-head", "origin", "master"]);
    fx.git(&[
        "merge",
        "-q",
        "--no-ff",
        "-m",
        "merge old-unmerged (local only)",
        "old-unmerged",
    ]);
}

#[test]
fn a_branch_merged_into_the_local_default_only_says_so_and_one_merged_into_origin_says_that() {
    let (_fx, h) = setup(like_rotp);
    let d = h.docs();
    let shape = |n: &str| {
        (
            d[n].merged,
            d[n].merged_into.clone().unwrap(),
            d[n].status.as_str(),
        )
    };
    assert_eq!(
        shape("old-unmerged"),
        (true, "master".to_string(), "merged")
    );
    assert_eq!(
        shape("old-merged"),
        (true, "origin/master".to_string(), "merged")
    );
    assert_eq!(
        (d["fresh"].merged, d["fresh"].merged_into.as_deref()),
        (false, Some("origin/master"))
    );
}

#[test]
fn the_branch_reason_says_which_default_it_was_merged_into_and_an_unmerged_one_is_kept() {
    let (fx, h) = setup(like_rotp);
    let delete = |name: &str| h.rm_with(&id(&h, &fx, name), |i| i.branch = true).unwrap();
    let local = delete("old-unmerged");
    assert_eq!(
        (local.removed, local.deleted_branch, local.backup.is_none()),
        (true, true, true)
    );
    assert_eq!(
        local.branch_why.as_deref(),
        Some("merged into local master (not pushed)")
    );
    assert!(
        local
            .text
            .ends_with("branch old-unmerged deleted: merged into local master (not pushed)")
    );
    let pushed = delete("old-merged");
    assert_eq!(
        (pushed.deleted_branch, pushed.branch_why.as_deref()),
        (true, Some("merged into origin/master"))
    );
    let now = branches(&fx.repo);
    assert!(!now.contains(&"old-unmerged".to_string()) && !now.contains(&"old-merged".to_string()));
    let fresh = delete("fresh");
    assert_eq!((fresh.removed, fresh.deleted_branch), (true, false));
    assert_eq!(fresh.branch_note.as_deref(), Some("not merged"));
}

#[test]
fn the_default_branch_itself_is_never_deleted_from_its_own_worktree() {
    let (fx, h) = setup(|fx| {
        fx.git(&["checkout", "-q", "-b", "dev"]);
        fx.git(&["worktree", "add", "-q", &fx.path("on-main"), "main"]);
    });
    let out = h
        .rm_with(&id(&h, &fx, "on-main"), |i| i.branch = true)
        .unwrap();
    assert_eq!(
        (out.removed, out.deleted_branch, out.branch_note.as_deref()),
        (
            true,
            false,
            Some("main is the default branch: never deleted")
        )
    );
    assert!(branches(&fx.repo).contains(&"main".to_string()));
}

#[test]
fn with_no_origin_at_all_the_local_main_is_the_default_and_merged_into_is_its_name() {
    let (_fx, h) = setup(|_| {});
    let d = h.docs();
    assert_eq!(
        (
            d["old-merged"].merged_into.as_deref(),
            d["old-merged"].merged
        ),
        (Some("main"), true)
    );
    assert_eq!(
        (
            d["old-unmerged"].merged_into.as_deref(),
            d["old-unmerged"].merged
        ),
        (Some("main"), false)
    );
}
