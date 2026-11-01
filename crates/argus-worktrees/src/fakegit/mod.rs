//! A fake git for the specs: an in-memory model of the few repositories a spec needs (commits, branches, HEAD, worktree
//! records, locks, ahead and behind, dirty and untracked files) behind the exec seam ([`GitExec`]).
//!
//! It answers exactly the commands this crate sends and parses only the argument lists the crate uses:
//!
//! - reads: `rev-parse`, `status --porcelain=v1 -b`, `log`, `rev-list --count`, `merge-base --is-ancestor`,
//!   `for-each-ref --contains`, `cherry`, `symbolic-ref`, `config --get`, `remote get-url`, `worktree list --porcelain`;
//! - the removal's own commands: `worktree remove|unlock|prune`, `update-ref -d`, `bundle create|verify`,
//!   `diff HEAD --binary --output=`, `ls-files`, `check-ignore -q`;
//! - what the specs use to build a repository: `worktree add|lock`, `remote add`, `add`, `commit -m`.
//!
//! The working tree is the real temporary folder: dirty and untracked files are read from it against the files the HEAD
//! commit tracks, ignore patterns come from `.git/info/exclude` and the root `.gitignore`, and the folders git keeps
//! (`HEAD`, `index`, the loose branch refs, `worktrees/<name>`) are written so the scan's fingerprints behave. A repository
//! has one parent per commit: no merges, no patch ids, no packed refs. A spec that needs those stays on the real git.

mod disk;
mod model;
mod read;
mod write;

use crate::exec::GitOutput;
use crate::git::GitExec;
use crate::paths::{base_name, clean_path, path_key};
use crate::sync::locked;
use model::{Fail, Head, Repo, Run, State, Wt, fatal};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The fake git of one fixture.
pub struct FakeGit {
    state: Mutex<State>,
}

type Fakes = Vec<(String, Arc<FakeGit>)>;

/// The fakes of the fixtures alive, by the folder the fixture lives in. A path under one of them is a path of that fake.
static FAKES: Mutex<Fakes> = Mutex::new(Vec::new());

/// Make `fake` the git of everything under `root`.
pub fn register(root: &str, fake: Arc<FakeGit>) {
    locked(&FAKES).push((path_key(root, cfg!(windows)), fake));
}

/// Forget the fake of a fixture that is gone.
pub fn unregister(root: &str) {
    let key = path_key(root, cfg!(windows));
    locked(&FAKES).retain(|(k, _)| *k != key);
}

/// The fake that owns this path, if the path lies under a fixture made on a fake.
pub fn fake_for(path: &str) -> Option<Arc<FakeGit>> {
    let key = path_key(path, cfg!(windows));
    locked(&FAKES)
        .iter()
        .find(|(root, _)| key == *root || key.starts_with(&format!("{root}/")))
        .map(|(_, fake)| fake.clone())
}

/// Drop the options the read door puts in front of every read.
fn bare(args: &[String]) -> &[String] {
    let rest = match args {
        [first, rest @ ..] if first == "--no-optional-locks" => rest,
        _ => args,
    };
    match rest {
        [dash, _, rest @ ..] if dash == "-c" => rest,
        _ => rest,
    }
}

fn dispatch(state: &mut State, cwd: &str, args: &[String]) -> Run {
    let (r, w) = state.locate(cwd)?;
    let repo = &mut state.repos[r];
    let Some((command, rest)) = bare(args).split_first() else {
        return Err(fatal("fake git: no command"));
    };
    match command.as_str() {
        "worktree" => worktree(repo, w, rest),
        "rev-parse" => read::rev_parse(repo, &repo.worktrees[w], rest),
        "status" => Ok(read::status(repo, &repo.worktrees[w], rest)),
        "log" => read::log(repo, &repo.worktrees[w], rest),
        "rev-list" => read::rev_list(repo, &repo.worktrees[w], rest),
        "merge-base" => read::merge_base(repo, &repo.worktrees[w], rest),
        "for-each-ref" => read::for_each_ref(repo, &repo.worktrees[w], rest),
        "cherry" => read::cherry(repo, &repo.worktrees[w], rest),
        "symbolic-ref" => read::symbolic_ref(repo),
        "config" => read::config(repo, rest),
        "ls-files" => Ok(read::ls_files(repo, &repo.worktrees[w], rest)),
        "check-ignore" => read::check_ignore(repo, &repo.worktrees[w], rest),
        "remote" if rest.first().map(String::as_str) == Some("get-url") => {
            read::remote_get_url(repo, rest)
        }
        "remote" => write::remote_add(repo, rest),
        "update-ref" => write::update_ref(repo, rest),
        "add" => Ok(write::stage(repo, w, rest)),
        "commit" => write::commit_staged(repo, w, rest),
        "bundle" => write::bundle(repo, &repo.worktrees[w], rest),
        "diff" => write::diff(repo, &repo.worktrees[w], rest),
        other => Err(fatal(&format!(
            "fake git: {other} is not a command the specs use"
        ))),
    }
}

fn worktree(repo: &mut Repo, from: usize, args: &[String]) -> Run {
    let Some((verb, rest)) = args.split_first() else {
        return Err(fatal("fake git: worktree takes a verb"));
    };
    match verb.as_str() {
        "list" => Ok(read::worktree_list(repo)),
        "add" => write::worktree_add(repo, from, rest),
        "remove" => write::worktree_remove(repo, rest),
        "lock" => write::worktree_lock(repo, rest, true),
        "unlock" => write::worktree_lock(repo, rest, false),
        "prune" => Ok(write::worktree_prune(repo, rest)),
        other => Err(fatal(&format!(
            "fake git: worktree {other} is not a command the specs use"
        ))),
    }
}

fn output(done: Run) -> GitOutput {
    match done {
        Ok(stdout) => GitOutput::success(&stdout),
        Err(Fail { code, msg }) => GitOutput::failure(code, &msg),
    }
}

impl FakeGit {
    /// A repository at `path` on its first branch, with no commit yet.
    pub fn init(path: &str, branch: &str) -> Arc<Self> {
        let path = clean_path(path);
        let repo = Repo {
            worktrees: vec![Wt {
                path: path.clone(),
                name: String::new(),
                head: Head::Branch(branch.to_string()),
                locked: None,
                staged: Vec::new(),
            }],
            ..Repo::default()
        };
        let _ = write::write_layout(&repo);
        let _ = write::write_head(&repo, 0);
        let _ = write::write_index(&repo, 0);
        Arc::new(Self {
            state: Mutex::new(State { repos: vec![repo] }),
        })
    }

    /// Run a command as the door would, without the timeout.
    pub fn run(&self, cwd: &str, args: &[String]) -> GitOutput {
        output(dispatch(&mut locked(&self.state), cwd, args))
    }

    /// Run a command the way a spec does: its output, and a panic with git's message when it fails.
    pub fn cli(&self, cwd: &str, args: &[&str]) -> String {
        let owned: Vec<String> = args.iter().map(ToString::to_string).collect();
        let out = self.run(cwd, &owned);
        assert!(out.ok(), "git {} in {cwd}: {}", args.join(" "), out.stderr);
        out.stdout
    }

    /// Commit one file on the checkout at `cwd`, dated `time_ms`.
    pub fn commit_file(&self, cwd: &str, name: &str, text: &str, time_ms: i64) {
        let mut state = locked(&self.state);
        let done = state.locate(cwd).and_then(|(r, w)| {
            write::commit_file(&mut state.repos[r], w, name, text, time_ms / 1000)
        });
        if let Err(fail) = done {
            panic!("git commit in {cwd}: {}", fail.msg);
        }
    }

    /// The branches of the first repository.
    pub fn branches(&self) -> Vec<String> {
        let state = locked(&self.state);
        state.repos[0].branches.keys().cloned().collect()
    }

    /// The folder names of the checkouts git lists for the first repository.
    pub fn listed(&self) -> Vec<String> {
        let state = locked(&self.state);
        state.repos[0]
            .worktrees
            .iter()
            .map(|w| base_name(&w.path))
            .collect()
    }
}

impl GitExec for FakeGit {
    fn exec(&self, cwd: &str, args: &[String], _timeout: Duration) -> GitOutput {
        if !Path::new(cwd).is_dir() {
            return GitOutput::failure(
                128,
                &format!("fatal: cannot change to '{cwd}': No such file or directory\n"),
            );
        }
        self.run(cwd, args)
    }
}
