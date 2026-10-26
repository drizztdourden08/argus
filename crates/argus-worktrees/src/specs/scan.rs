//! The scanner: literal parent folders, name filters, and a scan of real repositories that only reads.

use crate::clock::{Clock, DAY_MS, SystemClock};
use crate::fact::Fact;
use crate::fs::RealFs;
use crate::git::{GIT_PREFIX, Git};
use crate::guard::assert_read_only;
use crate::paths::{base_name, path_key, worktree_id};
use crate::rule::Rule;
use crate::scan::{ScanEnv, folder_candidates, scan_rules};
use crate::status::{Judging, classify};
use crate::testkit::{
    Fixture, NotRepo, Recording, no_measure, put, real_measure, root_of, snapshot,
};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

fn by_name(facts: &[Fact]) -> HashMap<String, &Fact> {
    facts.iter().map(|f| (base_name(&f.path), f)).collect()
}

fn names(list: &[String]) -> Vec<String> {
    let mut out: Vec<String> = list.iter().map(|p| base_name(p)).collect();
    out.sort();
    out
}

fn env<'a>(
    git: &'a Git,
    ci: bool,
    measure: &'a dyn Fn(&str) -> crate::measure::Leftover,
) -> ScanEnv<'a> {
    ScanEnv {
        git,
        fs: &RealFs,
        case_insensitive: ci,
        measure,
    }
}

fn rule_lines(rules: &[crate::scan::RuleInfo]) -> Vec<String> {
    let line = |r: &crate::scan::RuleInfo| {
        format!(
            "{} {} {} {} {:?}",
            r.id,
            r.exists,
            r.matched,
            r.candidates,
            r.error.as_deref()
        )
    };
    rules.iter().map(line).collect()
}

fn tree(root: &str, dirs: &[&str]) {
    for d in dirs {
        fs::create_dir_all(format!("{root}/{d}")).unwrap();
    }
}

#[test]
fn a_folder_rule_takes_every_subfolder_of_its_literal_path_and_a_wildcard_rule_is_invalid() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    tree(
        &root,
        &[
            "a/.claude/worktrees/x",
            "b/.claude/worktrees/y",
            "c/other",
            "feat-1",
        ],
    );
    put(&root, "afile", "x");
    let git = Git::new(Arc::new(NotRepo));
    let rules = vec![
        Rule::folder("W-1", &format!("{root}/a/.claude/worktrees"), "lead"),
        Rule::folder("W-3", &format!("{root}/b/.claude/worktrees"), "lead"),
        Rule::folder("W-2", &format!("{root}/gone"), "lead"),
        Rule::folder("W-9", &format!("{root}/*/.claude/worktrees"), "lead"),
    ];
    let out = scan_rules(&rules, &env(&git, false, &no_measure));
    let mut found: Vec<String> = out.facts.iter().map(|f| base_name(&f.path)).collect();
    found.sort();
    assert_eq!(found, ["x", "y"]);
    assert!(
        out.facts.iter().all(|f| !f.is_git && f.error.is_none()),
        "not a git repository is an answer, not an error"
    );
    let shape = rule_lines(&out.rules);
    assert_eq!(
        shape,
        [
            "W-1 true 1 1 None",
            "W-3 true 1 1 None",
            "W-2 false 0 0 None",
            "W-9 false 0 0 Some(\"invalid: wildcard path\")"
        ]
    );
    assert_eq!(out.rules[3].invalid.as_deref(), Some("wildcard path"));
}

#[test]
fn folder_candidates_come_from_directory_listings_only_none_for_a_missing_path_or_a_wildcard() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    tree(&root, &["a/.claude/worktrees/x", "b/other"]);
    let rule = |path: String| Rule::folder("W-1", &path, "lead");
    assert_eq!(
        names(&folder_candidates(
            &rule(format!("{root}/a/.claude/worktrees")),
            &RealFs,
            false
        )),
        ["x"]
    );
    assert_eq!(
        folder_candidates(&rule(format!("{root}/gone")), &RealFs, false).len(),
        0
    );
    assert!(
        folder_candidates(&rule(format!("{root}/*/.claude/worktrees")), &RealFs, false).is_empty(),
        "a wildcard rule is invalid: no candidates"
    );
}

#[test]
fn include_and_exclude_choose_direct_child_folder_names() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    tree(
        &root,
        &[
            "pick/web-1",
            "pick/web-22",
            "pick/Web-3",
            "pick/site-a",
            "pick/agent-1",
        ],
    );
    let path = format!("{root}/pick");
    let with = |include: &[&str], exclude: &[&str], ci: bool| {
        names(&folder_candidates(
            &Rule::folder("W-1", &path, "t").filtered(include, exclude),
            &RealFs,
            ci,
        ))
    };
    assert_eq!(with(&["web-*"], &[], true), ["Web-3", "web-1", "web-22"]);
    assert_eq!(with(&["web-*"], &[], false), ["web-1", "web-22"]);
    assert_eq!(with(&["web-?", "site-?"], &[], false), ["site-a", "web-1"]);
    assert_eq!(with(&[], &["web-*"], false), ["Web-3", "agent-1", "site-a"]);
    assert_eq!(with(&[], &["web-*"], true), ["agent-1", "site-a"]);
    assert_eq!(with(&["web-*"], &["web-2*"], false), ["web-1"]);
    assert!(
        with(&["web-*", "web*/x"], &[], false).len() >= 2,
        "a pattern with a slash matches no folder name, the others still do"
    );
}

#[test]
fn every_child_belongs_to_exactly_one_of_two_rules_sharing_a_parent() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    tree(&root, &["pick/web-1", "pick/site-a", "pick/agent-1"]);
    let path = format!("{root}/pick");
    let git = Git::new(Arc::new(NotRepo));
    let rules = vec![
        Rule::folder("W-1", &path, "t").filtered(&["web-*"], &[]),
        Rule::folder("W-2", &path, "t").filtered(&[], &["web-*"]),
    ];
    let out = scan_rules(
        &rules,
        &env(&git, true, &|_| crate::measure::Leftover::default()),
    );
    let map = by_name(&out.facts);
    assert_eq!(
        (
            map["web-1"].rule.as_str(),
            map["site-a"].rule.as_str(),
            map["agent-1"].rule.as_str()
        ),
        ("W-1", "W-2", "W-2")
    );
    assert!(map["web-1"].leftover.is_some() && map["site-a"].leftover.is_some());
    assert!(!out.facts.iter().any(|f| f.rule == "W-1" && (f.path.ends_with("site-a") || f.path.ends_with("agent-1"))));
    assert!(
        !out.facts
            .iter()
            .any(|f| f.rule == "W-2" && f.path.contains("/web-"))
    );
}

fn near(value: Option<i64>, expected: i64) -> bool {
    value.is_some_and(|v| (v - expected).abs() < 60_000)
}

fn scan_fixture(fx: &Fixture) -> crate::scan::ScanResult {
    let git = Git::system();
    scan_rules(
        &[Rule::folder("W-1", &fx.wts, "lead")],
        &env(&git, cfg!(windows), &real_measure),
    )
}

#[test]
fn a_folder_rule_finds_every_folder_and_the_worktree_git_lists_without_a_folder() {
    let fx = Fixture::new();
    let out = scan_fixture(&fx);
    let mut all: Vec<String> = by_name(&out.facts).keys().cloned().collect();
    all.sort();
    assert_eq!(
        all,
        [
            "dirty-old",
            "fresh",
            "gone",
            "old-merged",
            "old-unmerged",
            "plain"
        ]
    );
}

#[test]
fn a_folder_rule_reads_the_branch_the_commit_and_the_dirty_count() {
    let fx = Fixture::new();
    let out = scan_fixture(&fx);
    let f = by_name(&out.facts);
    let now = SystemClock.now_ms();
    let fresh = f["fresh"];
    assert_eq!(
        (fresh.is_git, fresh.branch.as_deref(), fresh.main),
        (true, Some("fresh"), false)
    );
    assert_eq!(
        (fresh.dirty, fresh.subject.as_deref()),
        (0, Some("add fresh.txt"))
    );
    assert_eq!(fresh.head.as_deref().map(str::len), Some(40));
    assert!(near(fresh.commit_ms, now), "the commit time is now");
    let old = f["old-unmerged"];
    assert!(
        near(old.commit_ms, now - 10 * DAY_MS),
        "the backdated commit"
    );
    assert!(near(old.index_ms, now - 10 * DAY_MS), "the backdated index");
    assert_eq!((f["old-merged"].dirty, f["dirty-old"].dirty), (0, 1));
}

#[test]
fn a_folder_rule_reads_what_is_merged_and_what_the_default_branch_lacks() {
    let fx = Fixture::new();
    let out = scan_fixture(&fx);
    let f = by_name(&out.facts);
    let old = f["old-unmerged"];
    assert_eq!(
        (old.merged, old.merged_into.as_deref()),
        (false, Some("main"))
    );
    assert_eq!(
        (f["old-merged"].merged, f["dirty-old"].merged),
        (true, true)
    );
    assert_eq!(
        (old.unmerged, old.unmerged_subjects.clone()),
        (Some(1), vec!["add old.txt".to_string()])
    );
    assert_eq!(
        (f["old-merged"].unmerged, f["fresh"].unmerged),
        (Some(0), Some(1))
    );
    assert_eq!(
        (f["fresh"].referenced, old.referenced),
        (Some(true), Some(true))
    );
    assert_eq!(f["old-merged"].default_names, ["main"]);
}

#[test]
fn the_repository_of_each_worktree_and_the_folders_that_are_not_work_trees() {
    let fx = Fixture::new();
    let git = Git::system();
    let ci = cfg!(windows);
    let out = scan_rules(
        &[Rule::folder("W-1", &fx.wts, "lead")],
        &env(&git, ci, &real_measure),
    );
    let f = by_name(&out.facts);
    for name in ["fresh", "gone"] {
        assert!(f[name].registered);
        assert_eq!(
            path_key(f[name].repo.as_deref().unwrap(), ci),
            path_key(&fx.repo, ci),
            "{name}"
        );
    }
    assert_eq!(
        (
            f["plain"].is_git,
            f["plain"].exists,
            f["plain"].error.clone()
        ),
        (false, true, None)
    );
    assert_eq!(
        (
            f["gone"].exists,
            f["gone"].is_git,
            f["gone"].branch.as_deref()
        ),
        (false, true, Some("gone"))
    );
    assert!(f["gone"].prunable.is_some());
    assert_eq!(f["fresh"].id, worktree_id(&f["fresh"].path, ci));
    let shape = rule_lines(&out.rules);
    assert_eq!(
        shape,
        ["W-1 true 1 5 None"],
        "five folders on disk (the deleted one comes from git)"
    );
}

#[test]
fn the_statuses_with_the_clock_at_now() {
    let fx = Fixture::new();
    let git = Git::system();
    let rule = Rule::folder("W-1", &fx.wts, "lead");
    let out = scan_rules(
        std::slice::from_ref(&rule),
        &env(&git, cfg!(windows), &real_measure),
    );
    let judging = Judging {
        rule: Some(&rule),
        now_ms: SystemClock.now_ms(),
        ..Judging::default()
    };
    let mut status: Vec<(String, &str)> = out
        .facts
        .iter()
        .map(|x| (base_name(&x.path), classify(x, &judging).status.as_str()))
        .collect();
    status.sort();
    let want = [
        ("dirty-old", "dirty-stale"),
        ("fresh", "active"),
        ("gone", "missing"),
        ("old-merged", "merged"),
        ("old-unmerged", "stale"),
        ("plain", "leftover"),
    ];
    assert_eq!(status, want.map(|(n, s)| (n.to_string(), s)));
}

#[test]
fn a_repo_rule_uses_the_repository_listing_and_never_lists_its_main_checkout() {
    let fx = Fixture::new();
    let git = Git::system();
    let out = scan_rules(
        &[Rule::folder("W-1", &fx.repo, "lead").repo()],
        &env(&git, cfg!(windows), &real_measure),
    );
    assert_eq!(
        names(&out.facts.iter().map(|f| f.path.clone()).collect::<Vec<_>>()),
        ["dirty-old", "fresh", "gone", "old-merged", "old-unmerged"]
    );
    assert!(!by_name(&out.facts)["gone"].exists);
    assert!(!out.facts.iter().any(|x| x.main));
    let wrong = scan_rules(
        &[Rule::folder("W-1", &fx.wts, "lead").repo()],
        &env(&git, cfg!(windows), &real_measure),
    );
    assert_eq!(wrong.facts.len(), 0);
    assert!(
        wrong.rules[0]
            .error
            .as_deref()
            .unwrap()
            .contains("not a git repository")
    );
}

#[test]
fn the_main_checkout_reached_by_a_folder_rule_is_flagged_main_and_never_merged() {
    let fx = Fixture::new();
    let git = Git::system();
    let rule = Rule::folder("W-1", &fx.root, "lead");
    let out = scan_rules(
        std::slice::from_ref(&rule),
        &env(&git, cfg!(windows), &real_measure),
    );
    let main = by_name(&out.facts)["proj"];
    assert!(main.main);
    let judging = Judging {
        rule: Some(&rule),
        now_ms: SystemClock.now_ms() + 30 * DAY_MS,
        ..Judging::default()
    };
    assert_eq!(classify(main, &judging).status.as_str(), "stale");
    fs::create_dir(format!("{}/inner", fx.repo)).unwrap();
    let inside = scan_rules(
        &[Rule::folder("W-2", &fx.repo, "lead")],
        &env(&git, cfg!(windows), &real_measure),
    );
    assert!(
        !by_name(&inside.facts)["inner"].is_git,
        "a sub-folder of a repository is not a work tree of its own"
    );
}

#[test]
fn the_scan_only_reads_every_command_is_a_reading_one_and_no_file_changes() {
    let fx = Fixture::new();
    let before = snapshot(&fx.root);
    let exec = Arc::new(Recording::default());
    let git = Git::new(exec.clone());
    scan_rules(
        &[Rule::folder("W-1", &fx.wts, "lead")],
        &env(&git, cfg!(windows), &real_measure),
    );
    let calls = exec.calls.lock().unwrap().clone();
    assert!(
        calls.len() > 20,
        "a real scan ran many commands: {}",
        calls.len()
    );
    for args in &calls {
        assert_eq!(args[..3], GIT_PREFIX.map(String::from), "{args:?}");
        let rest: Vec<&str> = args[3..].iter().map(String::as_str).collect();
        assert!(assert_read_only(&rest).is_ok(), "{rest:?}");
    }
    assert_eq!(
        snapshot(&fx.root),
        before,
        "sizes and modification times of every file are unchanged"
    );
}
