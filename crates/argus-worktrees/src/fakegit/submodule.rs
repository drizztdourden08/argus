//! Submodules on the fake git. A spec says which folders of a worktree are submodules and in what state; `submodule
//! status`, the `status` run inside one and `worktree remove` answer from that. The superproject's own `status` shows a
//! submodule the way git does: tracked, and modified when it moved or has changes.

use super::FakeGit;
use super::disk::Dirt;
use super::model::{Run, Wt, fatal};
use crate::paths::{join_path, path_key};
use crate::sync::locked;
use std::fmt::Write;
use std::fs;

/// What a submodule folder is like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubState {
    /// Registered, not initialized: an empty folder (`-` in `submodule status`).
    Uninitialized,
    /// Initialized, on the recorded commit, nothing uncommitted.
    Clean,
    /// Initialized, on another commit than the recorded one (`+`).
    Moved,
    /// Initialized, with a merge conflict (`U`).
    Conflict,
    /// Initialized, on the recorded commit, with an uncommitted change the superproject's `status` shows.
    Changed,
    /// Like `Changed`, but the superproject's `status` says nothing (`ignore = dirty`): only a look inside finds it.
    Edited,
}

/// One submodule of a worktree.
pub(super) struct Sub {
    pub path: String,
    pub state: SubState,
}

impl Sub {
    fn initialized(&self) -> bool {
        self.state != SubState::Uninitialized
    }

    /// The mark and the describe `submodule status` prints.
    fn mark(&self) -> (char, &'static str) {
        match self.state {
            SubState::Uninitialized => ('-', ""),
            SubState::Moved => ('+', " (heads/main)"),
            SubState::Conflict => ('U', " (heads/main)"),
            _ => (' ', " (heads/main)"),
        }
    }
}

/// Does the worktree hold an initialized submodule?
pub(super) fn has_initialized(wt: &Wt) -> bool {
    wt.submodules.iter().any(Sub::initialized)
}

/// `submodule status --recursive`: one line per submodule, a made-up commit in each.
pub(super) fn status(wt: &Wt, args: &[String]) -> Run {
    if args != ["status", "--recursive"] {
        return Err(fatal("fake git: submodule takes status --recursive"));
    }
    let mut out = String::new();
    for (n, sub) in wt.submodules.iter().enumerate() {
        let (mark, describe) = sub.mark();
        let sha = if mark == 'U' {
            "0".repeat(40)
        } else {
            format!("{:040x}", n + 1)
        };
        let _ = writeln!(out, "{mark}{sha} {}{describe}", sub.path);
    }
    Ok(out)
}

/// `status --porcelain` run inside a submodule folder: what is not committed there. None when `cwd` is not one.
pub(super) fn inside(wt: &Wt, cwd: &str) -> Option<String> {
    let ci = cfg!(windows);
    let key = path_key(cwd, ci);
    let sub = wt
        .submodules
        .iter()
        .filter(|s| s.initialized())
        .find(|s| path_key(&join_path(&wt.path, &s.path), ci) == key)?;
    Some(match sub.state {
        SubState::Changed | SubState::Edited => " M src/main.c\n".to_string(),
        _ => String::new(),
    })
}

/// The superproject's view of its own `status`: `.gitmodules` and the submodule folders are tracked, a submodule that
/// moved or changed is modified.
pub(super) fn adjust(wt: &Wt, dirt: &mut Dirt) {
    if wt.submodules.is_empty() {
        return;
    }
    let hidden = |name: &str| {
        let name = name.trim_end_matches('/');
        name == ".gitmodules" || wt.submodules.iter().any(|s| s.path == name)
    };
    dirt.lines
        .retain(|line| !hidden(line.get(3..).unwrap_or_default()));
    dirt.others.retain(|name| !hidden(name));
    for sub in &wt.submodules {
        let mark = match sub.state {
            SubState::Moved | SubState::Changed => " M",
            SubState::Conflict => "UU",
            _ => continue,
        };
        dirt.lines.push(format!("{mark} {}", sub.path));
    }
}

impl FakeGit {
    /// A submodule at `path` (below the worktree) in this state: the folder, its `.git` file when initialized, and the
    /// `.gitmodules` entry. Fixtures call it before the scan.
    pub fn add_submodule(&self, worktree: &str, path: &str, state: SubState) {
        let mut guard = locked(&self.state);
        let (r, w) = guard
            .locate(worktree)
            .unwrap_or_else(|fail| panic!("add_submodule in {worktree}: {}", fail.msg));
        let wt = &mut guard.repos[r].worktrees[w];
        let folder = join_path(&wt.path, path);
        fs::create_dir_all(&folder).unwrap();
        if state != SubState::Uninitialized {
            let text = format!("gitdir: {}/.git/modules/{path}\n", wt.path);
            fs::write(join_path(&folder, ".git"), text).unwrap();
        }
        let file = join_path(&wt.path, ".gitmodules");
        let mut text = fs::read_to_string(&file).unwrap_or_default();
        let _ = writeln!(
            text,
            "[submodule \"{path}\"]\n\tpath = {path}\n\turl = https://example.test/{path}.git"
        );
        fs::write(file, text).unwrap();
        wt.submodules.push(Sub {
            path: path.to_string(),
            state,
        });
    }
}
