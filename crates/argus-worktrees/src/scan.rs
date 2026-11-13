//! What is on disk under a project's worktree rules, found with read-only git.
//!
//! A folder rule: every allowed sub-folder of its literal path is a candidate. A repo rule: the path is one git
//! repository and its `git worktree list` is the source. No clock, no board and no database in here.

use crate::deleting::{is_deleting, original_name};
use crate::fact::{Fact, missing_fact};
use crate::fs::Fs;
use crate::git::Git;
use crate::inspect::{RepoList, ScanCtx, inspect_dir, reuse_dir};
use crate::measure::Leftover;
use crate::parse::{ListedWorktree, parse_worktree_list};
use crate::paths::{base_name, clean_path, join_path, parent_of, path_key, same_path, worktree_id};
use crate::probe::{Fingerprint, Reuse, fingerprint};
use crate::rule::{Rule, RuleKind};
use std::collections::{HashMap, HashSet};

/// Folders one rule may give.
pub const MAX_CANDIDATES: usize = 200;

/// What the scan knew about one rule.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleInfo {
    pub id: String,
    pub exists: bool,
    pub matched: u32,
    pub candidates: u32,
    pub error: Option<String>,
    pub invalid: Option<String>,
}

/// A folder named `*.argus-deleting-<ms>` under a rule (a deletion a crash cut short).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeletingFolder {
    pub path: String,
    pub rule: String,
}

/// Everything one scan found.
#[derive(Clone, Debug, Default)]
pub struct ScanResult {
    pub rules: Vec<RuleInfo>,
    pub facts: Vec<Fact>,
    /// The path keys of every worktree any listing named.
    pub listed: HashSet<String>,
    pub deleting: Vec<DeletingFolder>,
    /// The fingerprint each candidate folder had right before it was read, by path.
    pub prints: HashMap<String, Fingerprint>,
}

/// What a scan reads with.
pub struct ScanEnv<'a> {
    pub git: &'a Git,
    pub fs: &'a dyn Fs,
    pub case_insensitive: bool,
    pub measure: &'a dyn Fn(&str) -> Leftover,
}

/// The candidate folders of a folder rule, from directory listings only (no git).
pub fn folder_candidates(rule: &Rule, fs: &dyn Fs, ci: bool) -> Vec<String> {
    if rule.problem().is_some() || !fs.is_dir(&clean_path(&rule.path)) {
        return Vec::new();
    }
    let root = clean_path(&rule.path);
    let mut names = fs.dirs(&root).unwrap_or_default();
    names.sort();
    names
        .into_iter()
        .filter(|n| !is_deleting(n) && rule.allows(n, ci))
        .map(|n| join_path(&root, &n))
        .take(MAX_CANDIDATES)
        .collect()
}

/// The folder (above `path`) that says `path` is part of a live work tree, or none: the folder that holds it has a
/// `.git` of its own (file or folder), or any folder above it has a `.git` FILE (a linked worktree's marker). The
/// `.git` FOLDER of a main repository further up is expected (worktrees live in `X/.claude/worktrees`).
pub fn inside_worktree(path: &str, fs: &dyn Fs) -> Option<String> {
    let mut dir = parent_of(path);
    let mut last = String::new();
    let mut first = true;
    while !dir.is_empty() && dir != last {
        let dot = join_path(&dir, ".git");
        if fs.exists(&dot) && (first || !fs.is_dir(&dot)) {
            return Some(dir);
        }
        first = false;
        let up = parent_of(&dir);
        last = std::mem::replace(&mut dir, up);
    }
    None
}

/// Does a worktree git lists sit inside this folder (its root is below `path`)?
pub fn contains_listed(path: &str, listed: &HashSet<String>, ci: bool) -> bool {
    let prefix = format!("{}/", path_key(path, ci));
    listed.iter().any(|key| key.starts_with(&prefix))
}

/// A folder with no `.git` that no repository lists is a leftover: measure it and put the facts on it. A folder git
/// still lists stays `not-git`. Returns true when it is one.
pub fn mark_leftover(fact: &mut Fact, listed: &HashSet<String>, env: &ScanEnv<'_>) -> bool {
    if fact.is_git || fact.error.is_some() || !fact.exists || fact.leftover.is_some() {
        return fact.leftover.is_some();
    }
    if env.fs.exists(&join_path(&fact.path, ".git"))
        || listed.contains(&path_key(&fact.path, env.case_insensitive))
    {
        return false;
    }
    if contains_listed(&fact.path, listed, env.case_insensitive)
        || inside_worktree(&fact.path, env.fs).is_some()
    {
        return false;
    }
    fact.leftover = Some((env.measure)(&fact.path));
    true
}

struct Scanner<'a, 'e> {
    ctx: ScanCtx<'a>,
    env: &'e ScanEnv<'a>,
    seen: HashSet<String>,
    facts: Vec<Fact>,
    folder_facts: Vec<usize>,
    deleting: Vec<DeletingFolder>,
    reuse: Option<&'e Reuse>,
    prints: HashMap<String, Fingerprint>,
}

struct Found {
    paths: Vec<String>,
    missing: Vec<(ListedWorktree, Option<String>)>,
}

impl Scanner<'_, '_> {
    fn repo_candidates(&mut self, root: &str, info: &mut RuleInfo, found: &mut Found) {
        let fs = self.env.fs;
        if !fs.exists(&join_path(root, ".git")) && !fs.exists(&join_path(root, "HEAD")) {
            info.error = Some("not a git repository".to_string());
            return;
        }
        let out = self
            .env
            .git
            .read(root, &["worktree", "list", "--porcelain"]);
        if !out.ok() {
            let text = if out.timed_out {
                out.error.clone().unwrap_or_default()
            } else {
                out.first_line()
            };
            info.error = Some(if text.is_empty() {
                out.error
                    .clone()
                    .unwrap_or_else(|| "not a git repository".to_string())
            } else {
                text
            });
            return;
        }
        let items = parse_worktree_list(&out.stdout);
        let repo = items.first().map(|i| i.path.clone());
        for item in items.iter().skip(1).filter(|i| !i.bare) {
            if item.prunable.is_some() || !fs.is_dir(&item.path) {
                found.missing.push((item.clone(), repo.clone()));
            } else {
                found.paths.push(item.path.clone());
            }
        }
        self.ctx.lists.push(RepoList {
            items,
            defaults: None,
        });
    }

    fn folder_names(&mut self, root: &str, rule: &Rule, info: &mut RuleInfo, found: &mut Found) {
        let mut names = match self.env.fs.dirs(root) {
            Ok(names) => names,
            Err(err) => {
                info.error = Some(crate::error::io_code(&err));
                Vec::new()
            }
        };
        names.sort();
        for name in names {
            let deleting = is_deleting(&name);
            let base = if deleting {
                original_name(&name)
            } else {
                name.clone()
            };
            let path = join_path(root, &name);
            let may_be_listed = !deleting && self.env.fs.exists(&join_path(&path, ".git"));
            if !rule.allows(&base, self.env.case_insensitive) && !may_be_listed {
                continue;
            }
            if deleting {
                self.deleting.push(DeletingFolder {
                    path,
                    rule: rule.id.clone(),
                });
            } else {
                found.paths.push(path);
            }
        }
    }

    /// A folder an earlier rule already read is skipped before any git runs: it is listed once, for that rule.
    fn inspect_found(&mut self, rule: &Rule, paths: &[String]) {
        for dir in paths.iter().take(MAX_CANDIDATES) {
            let id = worktree_id(&clean_path(dir), self.env.case_insensitive);
            if !self.seen.insert(id) {
                continue;
            }
            let fact = self.read_dir(dir, rule);
            let ci = self.env.case_insensitive;
            if rule.kind == RuleKind::Folder {
                if !rule.admits(&base_name(dir), ci, fact.registered && !fact.main) {
                    self.seen.remove(&id);
                    continue;
                }
                self.folder_facts.push(self.facts.len());
            }
            self.facts.push(fact);
        }
    }

    /// One candidate folder: its fingerprint first (so a change during the read shows at the next look), then git, or
    /// the facts of the last scan when the fingerprint is the same.
    fn read_dir(&mut self, dir: &str, rule: &Rule) -> Fact {
        let path = clean_path(dir);
        let print = fingerprint(self.env.fs, &path);
        let kept = self.reuse.and_then(|r| r.keeps(&path, &print)).cloned();
        self.prints.insert(path, print);
        match kept {
            Some(kept) => reuse_dir(&kept, dir, rule, &mut self.ctx),
            None => inspect_dir(dir, rule, &mut self.ctx),
        }
    }

    fn folder_missing(&self, rule: &Rule, found: &mut Found) {
        let ci = self.env.case_insensitive;
        for list in &self.ctx.lists {
            for item in list.items.iter().skip(1) {
                let ours = same_path(&parent_of(&item.path), &rule.path, ci);
                if !item.bare && !self.env.fs.is_dir(&item.path) && ours {
                    found
                        .missing
                        .push((item.clone(), list.items.first().map(|i| i.path.clone())));
                }
            }
        }
    }

    fn add_missing(&mut self, rule: &Rule, missing: &[(ListedWorktree, Option<String>)]) {
        for (item, repo) in missing {
            let fact = missing_fact(item, rule, repo.as_deref(), self.env.case_insensitive);
            if self.seen.insert(fact.id.clone()) {
                self.facts.push(fact);
            }
        }
    }

    fn scan_rule(&mut self, rule: &Rule) -> RuleInfo {
        let mut info = RuleInfo {
            id: rule.id.clone(),
            ..RuleInfo::default()
        };
        if let Some(problem) = rule.problem() {
            info.invalid = Some(problem.to_string());
            info.error = Some(format!("invalid: {problem}"));
            return info;
        }
        let root = clean_path(&rule.path);
        let mut found = Found {
            paths: Vec::new(),
            missing: Vec::new(),
        };
        info.exists = self.env.fs.is_dir(&root);
        info.matched = u32::from(info.exists);
        if info.exists {
            match rule.kind {
                RuleKind::Repo => self.repo_candidates(&root, &mut info, &mut found),
                RuleKind::Folder => self.folder_names(&root, rule, &mut info, &mut found),
            }
        }
        info.candidates = u32::try_from(found.paths.len().min(MAX_CANDIDATES)).unwrap_or(u32::MAX);
        self.inspect_found(rule, &found.paths);
        if rule.kind == RuleKind::Folder {
            self.folder_missing(rule, &mut found);
        }
        self.add_missing(rule, &found.missing);
        info
    }

    fn finish(mut self, rules: Vec<RuleInfo>) -> ScanResult {
        let ci = self.env.case_insensitive;
        let listed: HashSet<String> = self
            .ctx
            .lists
            .iter()
            .flat_map(|l| l.items.iter().map(|i| path_key(&i.path, ci)))
            .collect();
        for index in std::mem::take(&mut self.folder_facts) {
            mark_leftover(&mut self.facts[index], &listed, self.env);
        }
        ScanResult {
            rules,
            facts: self.facts,
            listed,
            deleting: self.deleting,
            prints: self.prints,
        }
    }
}

/// Scan every rule. A path two rules reach is listed once, for the first rule. A stored rule with a wildcard path is
/// invalid: no candidates, no leftovers, reported as `invalid: wildcard path`.
#[cfg(test)]
pub fn scan_rules(rules: &[Rule], env: &ScanEnv<'_>) -> ScanResult {
    scan_rules_with(rules, env, None)
}

/// [`scan_rules`] that keeps the facts of a worktree whose fingerprint is the one `reuse` remembers, and reads the others.
pub fn scan_rules_with(rules: &[Rule], env: &ScanEnv<'_>, reuse: Option<&Reuse>) -> ScanResult {
    let ctx = ScanCtx {
        git: env.git,
        fs: env.fs,
        lists: Vec::new(),
        case_insensitive: env.case_insensitive,
    };
    let mut scanner = Scanner {
        ctx,
        env,
        seen: HashSet::new(),
        facts: Vec::new(),
        folder_facts: Vec::new(),
        deleting: Vec::new(),
        reuse,
        prints: HashMap::new(),
    };
    let infos: Vec<RuleInfo> = rules.iter().map(|rule| scanner.scan_rule(rule)).collect();
    scanner.finish(infos)
}
