//! Reading one folder with read-only git: is it a work tree, what is checked out, is it merged.

use crate::exec::{GitOutput, first_line};
use crate::fact::{Fact, blank_fact};
use crate::fs::Fs;
use crate::git::Git;
use crate::parse::{ListedWorktree, parse_status, parse_worktree_list};
use crate::paths::{clean_path, join_path, path_key};
use crate::rule::Rule;
use std::path::Path;

/// The default branches of a repository: where `origin/HEAD` points, and the local branch merges land in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Defaults {
    pub remote: Option<String>,
    pub local: Option<String>,
}

impl Defaults {
    /// The remote default first, then the local one.
    pub fn bases(&self) -> Vec<String> {
        [self.remote.clone(), self.local.clone()]
            .into_iter()
            .flatten()
            .collect()
    }
}

/// The listing of one repository, kept for the scan.
#[derive(Clone, Debug, Default)]
pub struct RepoList {
    pub items: Vec<ListedWorktree>,
    pub defaults: Option<Defaults>,
}

/// What a scan reads with: git, the file system, the listings already asked for.
pub struct ScanCtx<'a> {
    pub git: &'a Git,
    pub fs: &'a dyn Fs,
    pub lists: Vec<RepoList>,
    pub case_insensitive: bool,
}

struct Hit {
    list: usize,
    index: usize,
}

fn unfit(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    [
        "not a git repository",
        "not a work tree",
        "cannot change to",
        "no such file",
    ]
    .iter()
    .any(|s| lower.contains(s))
}

fn find_in(list: &RepoList, key: &str, ci: bool) -> Option<usize> {
    list.items.iter().position(|x| path_key(&x.path, ci) == key)
}

fn list_for(dir: &str, path: &str, ctx: &mut ScanCtx<'_>) -> Option<Hit> {
    let key = path_key(path, ctx.case_insensitive);
    for (list, known) in ctx.lists.iter().enumerate() {
        if let Some(index) = find_in(known, &key, ctx.case_insensitive) {
            return Some(Hit { list, index });
        }
    }
    let out = ctx.git.read(dir, &["worktree", "list", "--porcelain"]);
    if !out.ok() {
        return None;
    }
    ctx.lists.push(RepoList {
        items: parse_worktree_list(&out.stdout),
        defaults: None,
    });
    let list = ctx.lists.len() - 1;
    find_in(&ctx.lists[list], &key, ctx.case_insensitive).map(|index| Hit { list, index })
}

struct Probe {
    lines: Vec<String>,
    unborn: bool,
}

enum Probed {
    Tree(Probe),
    NotTree,
    Failed(String),
}

fn probe_repo(git: &Git, dir: &str) -> Probed {
    let mut out = git.read(
        dir,
        &[
            "rev-parse",
            "--show-toplevel",
            "--abbrev-ref",
            "HEAD",
            "--git-dir",
        ],
    );
    let mut unborn = false;
    if !out.ok() && !out.timed_out && !unfit(&out.stderr) {
        out = git.read(dir, &["rev-parse", "--show-toplevel", "--git-dir"]);
        unborn = out.ok();
    }
    if out.ok() {
        let lines = out
            .stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(ToString::to_string)
            .collect();
        return Probed::Tree(Probe { lines, unborn });
    }
    if unfit(&out.stderr) {
        return Probed::NotTree;
    }
    let text = out
        .error
        .clone()
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| first_line(&out.stderr));
    Probed::Failed(if text.is_empty() {
        "git failed".to_string()
    } else {
        text
    })
}

fn apply_status(fact: &mut Fact, out: &GitOutput) {
    if out.ok() {
        let status = parse_status(&out.stdout);
        (fact.dirty, fact.ahead, fact.behind, fact.upstream) =
            (status.dirty, status.ahead, status.behind, status.upstream);
        return;
    }
    let text = if out.timed_out {
        out.error.clone().unwrap_or_default()
    } else {
        out.first_line()
    };
    fact.error = Some(if text.is_empty() {
        out.error
            .clone()
            .unwrap_or_else(|| "git status failed".to_string())
    } else {
        text
    });
}

fn apply_log(fact: &mut Fact, out: &GitOutput) {
    if !out.ok() {
        return;
    }
    let text = out.stdout.trim_end_matches(['\r', '\n']);
    let mut parts = text.split('\x1f');
    let sha = parts.next().unwrap_or_default();
    let seconds: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let subject: String = parts
        .collect::<Vec<_>>()
        .join("\x1f")
        .chars()
        .take(120)
        .collect();
    fact.head = (!sha.is_empty()).then(|| sha.to_string());
    fact.commit_ms = (seconds > 0).then_some(seconds * 1000);
    fact.subject = (!subject.is_empty()).then_some(subject);
}

const STATUS: &[&str] = &["status", "--porcelain=v1", "-b"];
const LAST_COMMIT: &[&str] = &["log", "-1", "--format=%H%x1f%ct%x1f%s"];

/// Two independent reads of one folder at once, so a scan uses both git slots as the Node scanner did.
fn read_both(git: &Git, dir: &str, first: &[&str], second: &[&str]) -> (GitOutput, GitOutput) {
    std::thread::scope(|scope| {
        let other = scope.spawn(|| git.read(dir, second));
        let one = git.read(dir, first);
        let two = other
            .join()
            .unwrap_or_else(|_| GitOutput::refused("git read panicked"));
        (one, two)
    })
}

fn index_mtime(fs: &dyn Fs, dir: &str, git_dir: &str) -> Option<i64> {
    if git_dir.is_empty() {
        return None;
    }
    let folder = Path::new(dir).join(git_dir);
    fs.mtime_ms(&join_path(&clean_path(&folder.to_string_lossy()), "index"))
}

/// The facts of one candidate folder (at most six reads, never a write).
pub fn inspect_dir(dir: &str, rule: &Rule, ctx: &mut ScanCtx<'_>) -> Fact {
    let path = clean_path(dir);
    let mut fact = blank_fact(&path, rule, ctx.case_insensitive);
    if !ctx.fs.exists(&join_path(&path, ".git")) {
        return fact;
    }
    let probe = match probe_repo(ctx.git, dir) {
        Probed::Tree(probe) => probe,
        Probed::NotTree => return fact,
        Probed::Failed(text) => {
            fact.error = Some(text);
            return fact;
        }
    };
    fact.is_git = true;
    if !probe.unborn {
        let name = probe.lines.get(1).cloned().unwrap_or_default();
        fact.detached = name == "HEAD";
        fact.branch = (!fact.detached).then_some(name);
    }
    if probe.unborn {
        apply_status(&mut fact, &ctx.git.read(dir, STATUS));
    } else {
        let (status, log) = read_both(ctx.git, dir, STATUS, LAST_COMMIT);
        apply_status(&mut fact, &status);
        apply_log(&mut fact, &log);
    }
    let git_dir = probe
        .lines
        .get(if probe.unborn { 1 } else { 2 })
        .cloned()
        .unwrap_or_default();
    fact.index_ms = index_mtime(ctx.fs, dir, &git_dir);
    if let Some(hit) = list_for(dir, &path, ctx) {
        apply_entry(&mut fact, &hit, ctx);
        if !probe.unborn && !fact.main && fact.head.is_some() {
            mark_merged(dir, hit.list, &mut fact, ctx);
        }
    }
    fact
}

fn apply_entry(fact: &mut Fact, hit: &Hit, ctx: &ScanCtx<'_>) {
    let list = &ctx.lists[hit.list];
    let item = &list.items[hit.index];
    fact.main = hit.index == 0;
    fact.registered = true;
    fact.repo = list.items.first().map(|i| i.path.clone());
    fact.prunable.clone_from(&item.prunable);
    fact.locked.clone_from(&item.locked);
}

fn first_word(text: &str) -> &str {
    text.trim()
}

/// The default branches of a repository: the remote one where `origin/HEAD` points (when that branch exists), and the
/// local one a merge lands in. The local one is the remote default's own name when it exists locally, else what the main
/// checkout has checked out when that is main, master or `init.defaultBranch`, else `init.defaultBranch`, `main`, `master`,
/// else whatever the main checkout has checked out.
pub fn find_defaults(git: &Git, dir: &str, main_branch: Option<&str>) -> Defaults {
    let sym = git.read(
        dir,
        &["symbolic-ref", "-q", "--short", "refs/remotes/origin/HEAD"],
    );
    let remote = Some(first_word(&sym.stdout).to_string())
        .filter(|name| sym.ok() && !name.is_empty())
        .filter(|name| {
            git.read(
                dir,
                &[
                    "rev-parse",
                    "--verify",
                    "-q",
                    &format!("refs/remotes/{name}"),
                ],
            )
            .ok()
        });
    let cfg = git.read(dir, &["config", "--get", "init.defaultBranch"]);
    let init =
        Some(first_word(&cfg.stdout).to_string()).filter(|name| cfg.ok() && !name.is_empty());
    let remote_short = remote.as_ref().map(|r| {
        r.split_once('/')
            .map_or(r.as_str(), |(_, rest)| rest)
            .to_string()
    });
    let usual =
        main_branch.filter(|b| *b == "main" || *b == "master" || init.as_deref() == Some(*b));
    let mut tried: Vec<String> = Vec::new();
    let order = [
        remote_short,
        usual.map(String::from),
        init,
        Some("main".to_string()),
        Some("master".to_string()),
        main_branch.map(String::from),
    ];
    for name in order.into_iter().flatten() {
        if !tried.contains(&name) {
            tried.push(name);
        }
    }
    let local = tried.into_iter().find(|name| {
        git.read(
            dir,
            &["rev-parse", "--verify", "-q", &format!("refs/heads/{name}")],
        )
        .ok()
    });
    Defaults { remote, local }
}

fn defaults_of(dir: &str, list: usize, ctx: &mut ScanCtx<'_>) -> Defaults {
    if let Some(known) = &ctx.lists[list].defaults {
        return known.clone();
    }
    let main_branch = ctx.lists[list].items.first().and_then(|i| i.branch.clone());
    let found = find_defaults(ctx.git, dir, main_branch.as_deref());
    ctx.lists[list].defaults = Some(found.clone());
    found
}

/// Is HEAD kept by a branch, a remote branch or a tag? (A stash does not count.) None when git could not say.
pub fn head_kept(git: &Git, dir: &str) -> Option<bool> {
    rev_kept(git, dir, "HEAD")
}

/// Is this commit kept by a branch, a remote branch or a tag? None when git could not say.
pub fn rev_kept(git: &Git, dir: &str, rev: &str) -> Option<bool> {
    let args = [
        "for-each-ref",
        "--contains",
        rev,
        "--count=1",
        "--format=%(refname)",
        "refs/heads",
        "refs/remotes",
        "refs/tags",
    ];
    let out = git.read(dir, &args);
    out.ok().then(|| !out.stdout.trim().is_empty())
}

fn is_default_branch(branch: Option<&str>, bases: &[String]) -> bool {
    branch.is_some_and(|b| {
        bases
            .iter()
            .any(|base| base == b || *base == format!("origin/{b}"))
    })
}

fn mark_merged(dir: &str, list: usize, fact: &mut Fact, ctx: &mut ScanCtx<'_>) {
    fact.referenced = if fact.branch.is_some() && !fact.detached {
        Some(true)
    } else {
        head_kept(ctx.git, dir)
    };
    let bases = defaults_of(dir, list, ctx).bases();
    fact.default_names.clone_from(&bases);
    let Some(base) = bases.first().cloned() else {
        return;
    };
    fact.merged_into = Some(base.clone());
    if is_default_branch(fact.branch.as_deref(), &bases) {
        fact.unmerged = Some(0);
        return;
    }
    for candidate in &bases {
        if ctx
            .git
            .read(dir, &["merge-base", "--is-ancestor", "HEAD", candidate])
            .ok()
        {
            fact.merged = true;
            fact.merged_into = Some(candidate.clone());
            fact.unmerged = Some(0);
            return;
        }
    }
    mark_unmerged(dir, &base, fact, ctx);
}

fn mark_unmerged(dir: &str, base: &str, fact: &mut Fact, ctx: &ScanCtx<'_>) {
    let range = format!("{base}..HEAD");
    let (count, subjects) = read_both(
        ctx.git,
        dir,
        &["rev-list", "--count", &range],
        &["log", "-3", "--format=%s", &range],
    );
    let digits = count.stdout.trim();
    if count.ok() && !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
        fact.unmerged = digits.parse().ok();
    }
    if subjects.ok() {
        fact.unmerged_subjects = subjects
            .stdout
            .lines()
            .map(|s| s.trim().chars().take(120).collect::<String>())
            .filter(|s| !s.is_empty())
            .take(3)
            .collect();
    }
}
