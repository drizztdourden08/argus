//! What the fake git remembers: repositories, commits, branches and the worktree records of each.

use crate::paths::path_key;
use std::collections::{BTreeMap, HashMap, HashSet};

/// How a command ends when it does not succeed: git's exit code and what it prints.
pub(super) struct Fail {
    pub code: i32,
    pub msg: String,
}

/// What a command prints, or how it failed.
pub(super) type Run = Result<String, Fail>;

/// `fatal: ...`, exit code 128.
pub(super) fn fatal(msg: &str) -> Fail {
    Fail {
        code: 128,
        msg: format!("fatal: {msg}\n"),
    }
}

/// A failure that prints nothing (`rev-parse --verify -q`, `merge-base --is-ancestor`, `check-ignore -q`).
pub(super) fn quiet(code: i32) -> Fail {
    Fail {
        code,
        msg: String::new(),
    }
}

/// An io error as git reports a folder it could not write.
pub(super) fn io_fatal(err: &std::io::Error) -> Fail {
    fatal(&format!("{err}"))
}

/// A commit: one parent at most, a subject, a time and every tracked file with its text.
pub(super) struct Commit {
    pub parent: Option<String>,
    pub subject: String,
    pub time_s: i64,
    pub files: BTreeMap<String, String>,
}

/// What a worktree has checked out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Head {
    Branch(String),
    Detached(String),
}

/// One checkout of a repository. The first of a repository is its main checkout, with an empty record name.
pub(super) struct Wt {
    pub path: String,
    pub name: String,
    pub head: Head,
    pub locked: Option<String>,
    /// Paths `git add` took for the next `git commit`.
    pub staged: Vec<String>,
}

impl Wt {
    /// The folder with this checkout's HEAD and index, under the repository path.
    pub fn git_dir(&self, repo: &str) -> String {
        if self.name.is_empty() {
            format!("{repo}/.git")
        } else {
            format!("{repo}/.git/worktrees/{}", self.name)
        }
    }
}

/// One repository.
#[derive(Default)]
pub(super) struct Repo {
    pub commits: HashMap<String, Commit>,
    pub branches: BTreeMap<String, String>,
    pub remote_branches: BTreeMap<String, String>,
    pub tags: BTreeMap<String, String>,
    pub remotes: BTreeMap<String, String>,
    pub upstreams: BTreeMap<String, String>,
    pub origin_head: Option<String>,
    pub init_default: Option<String>,
    pub worktrees: Vec<Wt>,
    pub count: u64,
}

const SALT: u64 = 0xfa4e_0000_0000_0000;

/// splitmix64: a bijection on `u64`, so two counters never give the same digits.
fn mix(seed: u64) -> u64 {
    let mut x = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

impl Repo {
    pub fn path(&self) -> &str {
        self.worktrees.first().map_or("", |w| w.path.as_str())
    }

    /// A new commit id: forty hex digits that look like a hash (so the first seven tell commits apart), never the same
    /// twice in a repository: the counter goes through a bijective mix.
    pub fn new_id(&mut self) -> String {
        self.count += 1;
        let (a, b) = (mix(self.count), mix(self.count ^ SALT));
        format!("{a:016x}{b:016x}{:08x}", mix(a ^ b) >> 32)
    }

    /// The commits from `tip` down, newest first.
    pub fn chain(&self, tip: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut next = Some(tip.to_string());
        while let Some(id) = next {
            next = self.commits.get(&id).and_then(|c| c.parent.clone());
            out.push(id);
        }
        out
    }

    /// Is `old` the tip or below it?
    pub fn is_ancestor(&self, old: &str, tip: &str) -> bool {
        self.chain(tip).iter().any(|id| id == old)
    }

    /// The commits of `tip` that `base` does not have, newest first.
    pub fn beyond(&self, base: &str, tip: &str) -> Vec<String> {
        let known: HashSet<String> = self.chain(base).into_iter().collect();
        self.chain(tip)
            .into_iter()
            .filter(|id| !known.contains(id))
            .collect()
    }

    /// The commit a checkout stands on; empty before the first commit.
    pub fn head_id(&self, wt: &Wt) -> String {
        match &wt.head {
            Head::Branch(name) => self.branches.get(name).cloned().unwrap_or_default(),
            Head::Detached(id) => id.clone(),
        }
    }

    /// A revision as git reads it: `HEAD`, a full ref, a branch, a remote branch, a tag, or a commit id.
    pub fn resolve(&self, wt: Option<&Wt>, rev: &str) -> Option<String> {
        if rev == "HEAD" {
            return wt.map(|w| self.head_id(w)).filter(|id| !id.is_empty());
        }
        let tables = [
            ("refs/heads/", &self.branches),
            ("refs/remotes/", &self.remote_branches),
            ("refs/tags/", &self.tags),
        ];
        for (prefix, table) in tables {
            if let Some(id) = rev.strip_prefix(prefix).and_then(|name| table.get(name)) {
                return Some(id.clone());
            }
        }
        for (_, table) in tables {
            if let Some(id) = table.get(rev) {
                return Some(id.clone());
            }
        }
        self.commits
            .keys()
            .find(|id| rev.len() >= 7 && id.starts_with(rev))
            .cloned()
    }

    /// The checkout for a path, the longest one that holds it.
    pub fn locate(&self, cwd: &str) -> Option<usize> {
        let ci = cfg!(windows);
        let key = path_key(cwd, ci);
        self.worktrees
            .iter()
            .enumerate()
            .filter(|(_, w)| {
                let own = path_key(&w.path, ci);
                key == own || key.starts_with(&format!("{own}/"))
            })
            .max_by_key(|(_, w)| w.path.len())
            .map(|(i, _)| i)
    }

    /// The checkout whose folder is exactly this path.
    pub fn exact(&self, path: &str) -> Option<usize> {
        let ci = cfg!(windows);
        let key = path_key(path, ci);
        self.worktrees
            .iter()
            .position(|w| path_key(&w.path, ci) == key)
    }
}

/// Every repository of one fake git.
#[derive(Default)]
pub(super) struct State {
    pub repos: Vec<Repo>,
}

impl State {
    /// The repository and the checkout a command runs in.
    pub fn locate(&self, cwd: &str) -> Result<(usize, usize), Fail> {
        self.repos
            .iter()
            .enumerate()
            .find_map(|(r, repo)| repo.locate(cwd).map(|w| (r, w)))
            .ok_or_else(|| fatal("not a git repository (or any of the parent directories): .git"))
    }
}
