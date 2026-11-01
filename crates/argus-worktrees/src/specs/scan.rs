//! KEEP: K1 (which folders a rule takes, and that each belongs to exactly one rule).
//!
//! The scanner on temporary folders and a git that says every folder is not a repository.

// test-tag: critical

use crate::git::Git;
use crate::paths::base_name;
use crate::rule::Rule;
use crate::scan::scan_rules;
use crate::specs::scan_kit::{by_name, env, rule_lines};
use crate::testkit::{NotRepo, no_measure, put, root_of};
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

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
