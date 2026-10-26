//! A rule path is the LITERAL parent folder. The incident this guards: a rule `.../worktrees/web-*` made the matched LIVE
//! worktree the "parent", its sub-folders (apps, packages...) were listed as leftovers, and `--all-leftovers` would have
//! deleted a live worktree's contents. Here: a wildcard rule is ignored and reported, include and exclude choose
//! direct child folder names, and a leftover is never a worktree root git lists, never holds one, never sits inside a
//! live work tree.

use crate::batch::BatchInput;
use crate::board::Caller;
use crate::doc::Status;
use crate::error::ErrorCode;
use crate::fs::RealFs;
use crate::jobs::JobState;
use crate::leftover_remove::git_within;
use crate::rule::Rule;
use crate::testkit::{Fixture, Harness, commit, exists_itself, git_in, put, rule_for, write};
use std::fs;

fn live_web_1(fx: &Fixture) {
    let path = fx.path("web-1");
    git_in(
        &fx.repo,
        &["worktree", "add", "-q", "-b", "web-1", &path],
        &[],
    );
    fs::create_dir_all(format!("{path}/apps")).unwrap();
    fs::create_dir_all(format!("{path}/packages")).unwrap();
    commit(&path, "apps/readme.txt", "app\n", 0.0);
}

fn page_input() -> BatchInput {
    BatchInput {
        older_than_days: Some(0.0),
        confirm_young: true,
        ..BatchInput::new("alpha", Caller::Page)
    }
}

fn page_batch(h: &Harness) -> crate::batch::BatchOutcome {
    h.svc.remove_leftovers(&page_input()).unwrap()
}

fn page_remove(h: &Harness, id: &str) -> crate::error::WtError {
    h.refused(id, |i| (i.caller, i.confirm_young) = (Caller::Page, true))
}

fn wildcard_setup() -> (Fixture, Harness) {
    let fx = Fixture::new();
    live_web_1(&fx);
    let h = Harness::new(
        &fx.root,
        vec![Rule::folder("W-1", &format!("{}/web-*", fx.wts), "worker")],
        |_| {},
    );
    h.scan();
    (fx, h)
}

#[test]
fn a_stored_wildcard_rule_is_invalid_no_rows_and_reported_as_invalid() {
    let (_fx, h) = wildcard_setup();
    assert!(
        h.docs().is_empty(),
        "no row at all: not web-1, not apps, not packages"
    );
    let meta = h.meta();
    let rule = &meta.rules[0];
    assert_eq!(
        (rule.invalid.as_deref(), rule.error.as_deref()),
        (Some("wildcard path"), Some("invalid: wildcard path"))
    );
    assert_eq!(
        (rule.matched.unwrap_or(0), rule.candidates.unwrap_or(0)),
        (0, 0)
    );
    assert_eq!((meta.leftovers, meta.leftover_old.count), (0, 0));
}

#[test]
fn nothing_is_offered_or_deleted_under_an_invalid_rule() {
    let (fx, h) = wildcard_setup();
    assert_eq!(page_batch(&h).started, 0);
    let web = fx.path("web-1");
    for dir in ["apps", "packages"] {
        let err = page_remove(&h, &h.id_of(&format!("{web}/{dir}")));
        assert_eq!(err.code, ErrorCode::NotFound, "{dir}");
        assert!(
            exists_itself(&format!("{web}/{dir}")),
            "{dir} is still there"
        );
    }
    assert_eq!(
        page_remove(&h, &h.id_of(&web)).code,
        ErrorCode::NotFound,
        "nor the worktree itself under an invalid rule"
    );
    assert!(exists_itself(&format!("{web}/.git")));
    assert!(
        h.svc.live_step("alpha").added.is_empty(),
        "the live check does not pick the folders up either"
    );
}

#[test]
fn the_same_worktree_under_a_literal_parent_rule_with_include_is_one_row_and_its_sub_folders_are_never_candidates()
 {
    let fx = Fixture::new();
    live_web_1(&fx);
    let h = Harness::new(
        &fx.root,
        vec![Rule::folder("W-1", &fx.wts, "worker").filtered(&["web-*"], &[])],
        |_| {},
    );
    h.scan();
    assert_eq!(h.docs().keys().cloned().collect::<Vec<_>>(), ["web-1"]);
    assert!(h.docs()["web-1"].leftover.is_none());
    assert_eq!(h.meta().leftovers, 0);
}

#[test]
fn rotp_web_include_and_rotp_exclude_share_a_parent_every_child_belongs_to_exactly_one() {
    let fx = Fixture::new();
    for d in ["web-1", "web-2"] {
        put(&fx.wts, &format!("{d}/x.txt"), "x");
    }
    let rules = vec![
        Rule::folder("W-1", &fx.wts, "worker").filtered(&["web-*"], &[]),
        Rule::folder("W-2", &fx.wts, "worker").filtered(&[], &["web-*"]),
    ];
    let h = Harness::new(&fx.root, rules, |_| {});
    h.scan();
    let d = h.docs();
    assert_eq!(
        (
            d["web-1"].rule.as_str(),
            d["web-1"].status,
            d["web-2"].rule.as_str()
        ),
        ("W-1", Status::Leftover, "W-1")
    );
    assert_eq!(
        (
            d["fresh"].rule.as_str(),
            d["old-merged"].rule.as_str(),
            d["plain"].rule.as_str(),
            d["plain"].status
        ),
        ("W-2", "W-2", "W-2", Status::Leftover)
    );
    let meta = h.meta();
    let (first, second) = (&meta.rules[0], &meta.rules[1]);
    assert_eq!(
        (first.include.clone(), first.exclude.len(), first.candidates),
        (vec!["web-*".to_string()], 0, Some(2))
    );
    assert_eq!(
        (
            second.include.len(),
            second.exclude.clone(),
            second.candidates
        ),
        (0, vec!["web-*".to_string()], Some(5))
    );
}

fn junk_setup(
    rules: impl FnOnce(&Fixture) -> Vec<Rule>,
    prep: impl FnOnce(&Fixture),
) -> (Fixture, Harness) {
    let fx = Fixture::new();
    prep(&fx);
    let h = Harness::new(&fx.root, rules(&fx), |_| {});
    h.scan();
    (fx, h)
}

fn plain_junk(fx: &Fixture) {
    put(&fx.wts, "junk/sub/a.txt", "x");
}

fn add_inner_worktree(fx: &Fixture) {
    git_in(
        &fx.repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "inner-wt",
            &format!("{}/junk/inner", fx.wts),
        ],
        &[],
    );
}

#[test]
fn a_folder_that_now_holds_a_worktree_git_lists_is_refused_live_work_not_a_leftover() {
    let (fx, h) = junk_setup(|fx| vec![rule_for(fx)], plain_junk);
    assert_eq!(
        h.docs()["junk"].status,
        Status::Leftover,
        "a plain leftover at scan time"
    );
    add_inner_worktree(&fx);
    let err = page_remove(&h, &h.id_of(&format!("{}/junk", fx.wts)));
    assert_eq!(err.code, ErrorCode::Conflict);
    assert!(
        err.message.contains("holds the working tree ")
            && err.message.contains("junk/inner (a worktree of ")
            && err
                .message
                .contains("it is live work, not a leftover; nothing was deleted"),
        "{}",
        err.message
    );
    assert!(
        exists_itself(&format!("{}/junk/inner/.git", fx.wts))
            && exists_itself(&format!("{}/junk/sub/a.txt", fx.wts))
    );
    h.scan();
    assert_ne!(
        h.docs()["junk"].status,
        Status::Leftover,
        "a folder holding a listed worktree is never a leftover row"
    );
    assert_eq!(h.meta().leftover_old.count, 0);
}

#[test]
fn the_bulk_path_applies_the_same_guards_per_folder_the_plain_leftover_beside_it_goes() {
    let (fx, h) = junk_setup(|fx| vec![rule_for(fx)], plain_junk);
    add_inner_worktree(&fx);
    put(&fx.wts, "plain-junk/b.txt", "x");
    h.scan();
    let out = page_batch(&h);
    h.idle();
    assert_eq!(out.started, 2, "{:?}", out.skipped);
    assert!(
        !h.svc
            .jobs("alpha")
            .iter()
            .any(|j| j.path.ends_with("/junk")),
        "junk was not even queued"
    );
    assert!(!exists_itself(&format!("{}/plain-junk", fx.wts)));
    assert!(
        exists_itself(&format!("{}/junk/inner/.git", fx.wts))
            && exists_itself(&format!("{}/junk/sub/a.txt", fx.wts))
    );
}

#[test]
fn a_worktree_that_appears_inside_a_queued_leftover_before_its_turn_makes_that_job_refused_and_nothing_is_deleted()
 {
    let (fx, h) = junk_setup(|fx| vec![rule_for(fx)], plain_junk);
    let hold = h.svc.one_at_a_time();
    let input = page_input();
    h.svc.remove_leftovers(&input).unwrap();
    add_inner_worktree(&fx);
    drop(hold);
    h.idle();
    let job = h
        .svc
        .jobs("alpha")
        .into_iter()
        .find(|j| j.path.ends_with("/junk"))
        .unwrap();
    assert_eq!(job.state, JobState::Refused);
    assert!(job.error.unwrap().contains("holds the working tree "));
    assert!(
        exists_itself(&format!("{}/junk/inner/.git", fx.wts))
            && exists_itself(&format!("{}/junk/sub/a.txt", fx.wts)),
        "nothing was deleted"
    );
}

#[test]
fn a_folder_inside_a_folder_with_a_git_of_its_own_is_refused() {
    let (fx, h) = junk_setup(|fx| vec![rule_for(fx)], plain_junk);
    put(&fx.wts, ".git", "gitdir: /somewhere/else\n");
    let err = page_remove(&h, &h.id_of(&format!("{}/junk", fx.wts)));
    assert_eq!(err.code, ErrorCode::Conflict);
    assert!(err.message.contains(", a folder with a .git of its own: it is part of a live work tree, not a leftover; nothing was deleted"), "{}", err.message);
    assert!(exists_itself(&format!("{}/junk/sub/a.txt", fx.wts)));
}

#[test]
fn a_git_file_higher_up_is_refused_but_a_main_repositorys_git_folder_higher_up_is_the_normal_layout()
 {
    let (fx, h) = junk_setup(
        |fx| {
            vec![Rule::folder(
                "W-1",
                &format!("{}/outer/wts", fx.root),
                "worker",
            )]
        },
        |fx| write(&fx.root, "outer/wts/junk/a.txt", "x"),
    );
    assert_eq!(h.docs()["junk"].status, Status::Leftover);
    put(&fx.root, "outer/.git", "gitdir: /x\n");
    let err = page_remove(&h, &h.id_of(&format!("{}/outer/wts/junk", fx.root)));
    assert_eq!(err.code, ErrorCode::Conflict);
    assert!(
        err.message
            .contains("outer, a folder with a .git of its own"),
        "{}",
        err.message
    );
    assert!(exists_itself(&format!("{}/outer/wts/junk/a.txt", fx.root)));
    let (ok_fx, ok) = junk_setup(
        |fx| {
            vec![Rule::folder(
                "W-1",
                &format!("{}/.claude/worktrees", fx.repo),
                "worker",
            )]
        },
        |fx| write(&fx.repo, ".claude/worktrees/junk/a.txt", "x"),
    );
    assert_eq!(ok.docs()["junk"].status, Status::Leftover);
    let out = ok
        .rm_with(
            &ok.id_of(&format!("{}/.claude/worktrees/junk", ok_fx.repo)),
            |i| (i.caller, i.confirm_young) = (Caller::Page, true),
        )
        .unwrap();
    assert!(out.started);
    ok.idle();
}

#[test]
fn a_git_within_three_levels_is_found_one_deeper_than_three_is_not_asked_for() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::testkit::root_of(&dir);
    fs::create_dir_all(format!("{root}/junk/a/b/c/.git")).unwrap();
    let found = git_within(&RealFs, &format!("{root}/junk")).unwrap();
    assert!(found.ends_with("junk/a/b/c/.git"), "{found}");
    fs::create_dir_all(format!("{root}/deep/a/b/c/d/.git")).unwrap();
    assert!(git_within(&RealFs, &format!("{root}/deep")).is_none());
    put(&root, "node/node_modules/x/.git/HEAD", "x");
    assert!(
        git_within(&RealFs, &format!("{root}/node")).is_none(),
        "node_modules is not entered"
    );
}

#[test]
fn a_rule_that_is_invalid_or_a_child_its_lists_skip_is_refused_by_the_delete_guards_too() {
    let (fx, h) = junk_setup(|fx| vec![rule_for(fx)], plain_junk);
    let target = h.id_of(&format!("{}/junk", fx.wts));
    h.board().rules[0].path = format!("{}/*", fx.wts);
    assert_eq!(page_remove(&h, &target).code, ErrorCode::NotFound);
    h.board().rules[0].path = fx.wts.clone();
    h.board().rules[0].exclude = vec!["junk".to_string()];
    let skipped = page_remove(&h, &target);
    assert!(matches!(
        skipped.code,
        ErrorCode::Forbidden | ErrorCode::NotFound
    ));
    if skipped.code == ErrorCode::Forbidden {
        assert!(
            skipped
                .message
                .contains("outside the project's worktree rules")
        );
    }
    assert!(exists_itself(&format!("{}/junk/sub/a.txt", fx.wts)));
}
