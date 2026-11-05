//! A throwaway repository with worktrees in every state the scanner tells apart, built in the OS
//! temp folder with the real `git` (never a repository of the owner).
//!
//! - `fresh`: a commit made now, active
//! - `old-unmerged`: a commit 10 days old, not merged, index 10 days old, stale
//! - `old-merged`: at the tip of `main` (20 days old), index 15 days old, clean, merged
//! - `dirty-old`: like `old-merged` with an untracked file, dirty-stale
//! - `gone`: added, then its folder deleted
//! - `plain`: a folder that is not a git work tree, a leftover

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use argus_core::clock::{iso_from_ms, now_ms};
use argus_core::process::hidden_command;
use filetime::{FileTime, set_file_mtime};
use tempfile::TempDir;

pub const DAY_MS: i64 = 86_400_000;

/// Runs git in `cwd` as a test author; the output text.
pub fn git(cwd: &Path, args: &[&str], dated: Option<i64>) -> String {
    let mut command = hidden_command("git");
    command.args(args).current_dir(cwd);
    for (name, value) in [("NAME", "Test"), ("EMAIL", "test@example.test")] {
        command.env(format!("GIT_AUTHOR_{name}"), value);
        command.env(format!("GIT_COMMITTER_{name}"), value);
    }
    if let Some(days) = dated {
        let when = iso_from_ms(now_ms() - days * DAY_MS);
        command
            .env("GIT_AUTHOR_DATE", &when)
            .env("GIT_COMMITTER_DATE", when);
    }
    let out = command.output().expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn commit(cwd: &Path, name: &str, text: &str, days_ago: i64) {
    std::fs::write(cwd.join(name), text).expect("a file");
    git(cwd, &["add", name], None);
    git(
        cwd,
        &["commit", "-q", "-m", &format!("add {name}")],
        Some(days_ago),
    );
}

/// Sets the modification time of a path to `days` ago.
pub fn age(path: &Path, days: f64) {
    let ago = Duration::from_secs_f64(days * 86_400.0);
    let when = SystemTime::now() - ago;
    set_file_mtime(path, FileTime::from_system_time(when)).expect("a time");
}

fn index_of(dir: &Path) -> PathBuf {
    let dot = dir.join(".git");
    let text = std::fs::read_to_string(&dot).unwrap_or_default();
    match text.strip_prefix("gitdir:") {
        Some(rest) => PathBuf::from(rest.trim()).join("index"),
        None => dot.join("index"),
    }
}

pub struct Fixture {
    _root: TempDir,
    pub repo: PathBuf,
    /// The folder that holds the worktrees.
    pub wts: PathBuf,
    pub path: HashMap<&'static str, PathBuf>,
}

impl Fixture {
    pub fn new() -> Fixture {
        let root = tempfile::tempdir().expect("a temp folder");
        let repo = root.path().join("proj");
        let wts = root.path().join("proj-worktrees");
        std::fs::create_dir_all(&repo).expect("repo folder");
        std::fs::create_dir_all(&wts).expect("worktrees folder");
        git(&repo, &["init", "-q", "-b", "main"], None);
        commit(&repo, "a.txt", "one\n", 20);
        let mut fx = Fixture {
            _root: root,
            repo,
            wts,
            path: HashMap::new(),
        };
        fx.build();
        fx
    }

    /// Adds a worktree on a new branch of its own name.
    pub fn add(&mut self, name: &'static str) -> PathBuf {
        let path = self.wts.join(name);
        git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                name,
                &path.display().to_string(),
            ],
            None,
        );
        self.path.insert(name, path.clone());
        path
    }

    fn build(&mut self) {
        let fresh = self.add("fresh");
        commit(&fresh, "fresh.txt", "new work\n", 0);
        let old = self.add("old-unmerged");
        commit(&old, "old.txt", "older work\n", 10);
        age(&index_of(&old), 10.0);
        let merged = self.add("old-merged");
        age(&index_of(&merged), 15.0);
        let dirty = self.add("dirty-old");
        std::fs::write(dirty.join("scratch.txt"), "not committed\n").expect("a file");
        age(&index_of(&dirty), 15.0);
        let gone = self.add("gone");
        std::fs::remove_dir_all(gone).expect("the folder goes");
        let plain = self.wts.join("plain");
        std::fs::create_dir_all(&plain).expect("plain folder");
        std::fs::write(plain.join("readme.txt"), "not a repository\n").expect("a file");
        self.path.insert("plain", plain);
    }

    /// A folder with no `.git` and one file in it, every time stamp `days` ago.
    pub fn leftover(&self, name: &str, days: f64) -> PathBuf {
        let dir = self.wts.join(name);
        std::fs::create_dir_all(&dir).expect("leftover folder");
        let file = dir.join("a.txt");
        std::fs::write(&file, "left over\n").expect("a file");
        age(&file, days);
        age(&dir, days);
        dir
    }
}
