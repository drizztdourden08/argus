//! The cheap look: what changed on disk since the last scan, from file times and sizes only. No git process runs here.
//!
//! A worktree's fingerprint is made of the files git rewrites when its state changes. A scan keeps the fingerprint of
//! each worktree it read; the next look compares, and only a changed (new, gone) worktree is read with git again. The
//! shape of a project says whether folders came or went under its rules. A dirty file that git has not staged changes
//! none of these files, so a scan also runs on a fixed period (`Options::scan_ms`, `SAFETY_SCAN_MS` by default).

use crate::fact::Fact;
use crate::fs::Fs;
use crate::paths::{clean_path, join_path};
use crate::rule::{Rule, RuleKind};
use crate::scan::folder_candidates;
use std::collections::HashMap;

/// How often everything is read with git again whatever the fingerprints say (30 minutes; the plan allows 30 to 60).
pub const SAFETY_SCAN_MS: i64 = 30 * 60_000;

/// The modification time and the size of one file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    mtime_ms: Option<i64>,
    size: u64,
}

fn stamp(fs: &dyn Fs, path: &str) -> Option<Stamp> {
    let stat = fs.lstat(path).ok()?;
    Some(Stamp {
        mtime_ms: stat.mtime_ms,
        size: stat.size,
    })
}

/// What a look at one folder saw: it is there, its `.git`, its HEAD and index, the loose ref of its branch, the loose
/// ref of the branch the main checkout has, and the packed refs of the repository.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fingerprint {
    folder: bool,
    dot_git: Option<Stamp>,
    head: Option<Stamp>,
    index: Option<Stamp>,
    branch_ref: Option<Stamp>,
    main_ref: Option<Stamp>,
    packed_refs: Option<Stamp>,
}

fn is_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with('/') || (bytes.len() >= 3 && bytes[1] == b':' && bytes[2] == b'/')
}

/// `rel` against `base` (both clean paths), the `..` and `.` segments applied.
fn resolve(base: &str, rel: &str) -> String {
    let rel = clean_path(rel);
    if is_absolute(&rel) {
        return rel;
    }
    let mut parts: Vec<&str> = base.split('/').collect();
    for segment in rel.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if parts.len() > 1 {
                    parts.pop();
                }
            }
            name => parts.push(name),
        }
    }
    parts.join("/")
}

/// The folder with the worktree's own HEAD and index: `.git` itself, or what the `gitdir:` line of a `.git` file names.
fn git_dir_of(fs: &dyn Fs, dir: &str) -> Option<String> {
    let dot = join_path(dir, ".git");
    if fs.is_dir(&dot) {
        return Some(dot);
    }
    let text = fs.read_text(&dot).ok()?;
    let line = text.lines().find_map(|l| l.strip_prefix("gitdir:"))?;
    Some(resolve(dir, line.trim()))
}

/// The folder with the refs: the git dir of a main checkout, the one its `commondir` file names for a linked worktree.
fn common_dir(fs: &dyn Fs, git_dir: &str) -> String {
    fs.read_text(&join_path(git_dir, "commondir"))
        .map_or_else(|_| git_dir.to_string(), |t| resolve(git_dir, t.trim()))
}

fn head_text(fs: &dyn Fs, git_dir: &str) -> String {
    fs.read_text(&join_path(git_dir, "HEAD"))
        .unwrap_or_default()
}

/// The loose ref file a HEAD names (`ref: refs/heads/x`), none for a detached HEAD.
fn ref_stamp(fs: &dyn Fs, common: &str, head: &str) -> Option<Stamp> {
    let name = head.trim().strip_prefix("ref:")?.trim();
    stamp(fs, &join_path(common, name))
}

/// Look at one folder.
pub fn fingerprint(fs: &dyn Fs, dir: &str) -> Fingerprint {
    let dir = clean_path(dir);
    let mut print = Fingerprint {
        folder: fs.is_dir(&dir),
        ..Fingerprint::default()
    };
    if !print.folder {
        return print;
    }
    print.dot_git = stamp(fs, &join_path(&dir, ".git"));
    let Some(git_dir) = git_dir_of(fs, &dir) else {
        return print;
    };
    let common = common_dir(fs, &git_dir);
    print.head = stamp(fs, &join_path(&git_dir, "HEAD"));
    print.index = stamp(fs, &join_path(&git_dir, "index"));
    print.branch_ref = ref_stamp(fs, &common, &head_text(fs, &git_dir));
    print.main_ref = ref_stamp(fs, &common, &head_text(fs, &common));
    print.packed_refs = stamp(fs, &join_path(&common, "packed-refs"));
    print
}

/// The folder where a repository registers its linked worktrees.
fn registrations(fs: &dyn Fs, root: &str) -> Option<Stamp> {
    let dot = join_path(root, ".git");
    let common = if fs.is_dir(&dot) {
        dot
    } else {
        root.to_string()
    };
    stamp(fs, &join_path(&common, "worktrees"))
}

/// How the rules of a project sit on disk: whether each root is there, the candidate folders of a folder rule, and the
/// registration folder of a repo rule (a worktree added or removed with git changes it). A change is a folder that is
/// new or gone.
pub fn shape_of(rules: &[Rule], fs: &dyn Fs, case_insensitive: bool) -> String {
    let lines: Vec<String> = rules
        .iter()
        .filter(|rule| rule.problem().is_none())
        .map(|rule| {
            let root = clean_path(&rule.path);
            let seen = match rule.kind {
                RuleKind::Folder => folder_candidates(rule, fs, case_insensitive).join(";"),
                RuleKind::Repo => format!("{:?}", registrations(fs, &root)),
            };
            format!("{}|{}|{seen}", rule.id, fs.is_dir(&root))
        })
        .collect();
    lines.join("\n")
}

/// What an earlier scan of a project read: each worktree's facts and the fingerprint it had when it was read.
pub struct Reuse {
    /// When the scan that read everything with git ran.
    pub full_ms: i64,
    prints: HashMap<String, Fingerprint>,
    facts: HashMap<String, Fact>,
}

impl Reuse {
    pub fn new(full_ms: i64, prints: HashMap<String, Fingerprint>, facts: &[Fact]) -> Self {
        Self {
            full_ms,
            prints,
            facts: facts
                .iter()
                .map(|fact| (fact.path.clone(), fact.clone()))
                .collect(),
        }
    }

    /// The facts of the worktree at `path`, when it is a settled git worktree whose fingerprint did not change.
    pub fn keeps(&self, path: &str, now: &Fingerprint) -> Option<&Fact> {
        let fact = self.facts.get(path)?;
        let settled = fact.exists && fact.is_git && fact.error.is_none() && fact.leftover.is_none();
        (settled && now.head.is_some() && self.prints.get(path) == Some(now)).then_some(fact)
    }
}
