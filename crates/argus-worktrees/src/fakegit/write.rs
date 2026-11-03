//! The commands that change something: the removal's own commands and the few the specs use to build a repository.
//! The folders git keeps on disk (`HEAD`, `index`, the loose branch refs, `worktrees/<name>`) are written too, because the
//! scan reads their times to see that a worktree changed.

use super::model::{Commit, Fail, Head, Repo, Run, Wt, fatal, io_fatal};
use super::read::dirt_of;
use crate::clock::{Clock, SystemClock};
use crate::fs::{Fs, RealFs};
use crate::paths::{base_name, clean_path};
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::Path;

/// Write a file, making its folders.
pub(super) fn write_file(path: &str, text: &str) -> io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)
}

/// The folders a real `git init` leaves in `.git` that a spec or the fake touches: `info` (a spec writes `info/exclude`
/// there), `refs/heads`, `refs/tags`. `worktrees` appears with the first linked worktree, as in git.
pub(super) fn write_layout(repo: &Repo) -> io::Result<()> {
    for folder in ["info", "refs/heads", "refs/tags"] {
        fs::create_dir_all(format!("{}/.git/{folder}", repo.path()))?;
    }
    Ok(())
}

/// The `HEAD` file of a checkout.
pub(super) fn write_head(repo: &Repo, index: usize) -> io::Result<()> {
    let wt = &repo.worktrees[index];
    let text = match &wt.head {
        Head::Branch(name) => format!("ref: refs/heads/{name}\n"),
        Head::Detached(id) => format!("{id}\n"),
    };
    write_file(&format!("{}/HEAD", wt.git_dir(repo.path())), &text)
}

/// The loose ref file of a branch.
pub(super) fn write_ref(repo: &Repo, branch: &str) -> io::Result<()> {
    let id = repo.branches.get(branch).cloned().unwrap_or_default();
    write_file(
        &format!("{}/.git/refs/heads/{branch}", repo.path()),
        &format!("{id}\n"),
    )
}

/// The index file of a checkout, rewritten as git does when a commit is made.
pub(super) fn write_index(repo: &Repo, index: usize) -> io::Result<()> {
    let wt = &repo.worktrees[index];
    let text = format!("index of {}\n", repo.head_id(wt));
    write_file(&format!("{}/index", wt.git_dir(repo.path())), &text)
}

fn io_run(done: io::Result<()>) -> Result<(), Fail> {
    done.map_err(|err| io_fatal(&err))
}

/// A commit on the checkout `index`: the parent's files with `changes` laid over them.
fn make_commit(
    repo: &mut Repo,
    index: usize,
    changes: Vec<(String, String)>,
    subject: String,
    time_s: i64,
) -> Result<(), Fail> {
    let parent = Some(repo.head_id(&repo.worktrees[index])).filter(|id| !id.is_empty());
    let mut files = parent
        .as_ref()
        .and_then(|p| repo.commits.get(p))
        .map(|c| c.files.clone())
        .unwrap_or_default();
    files.extend(changes);
    let id = repo.new_id();
    repo.commits.insert(
        id.clone(),
        Commit {
            parent,
            subject,
            time_s,
            files,
        },
    );
    match repo.worktrees[index].head.clone() {
        Head::Branch(branch) => {
            repo.branches.insert(branch.clone(), id);
            io_run(write_ref(repo, &branch))?;
        }
        Head::Detached(_) => repo.worktrees[index].head = Head::Detached(id),
    }
    io_run(write_head(repo, index))?;
    io_run(write_index(repo, index))
}

/// A commit of one file on the checkout `index`: the file is written and tracked.
pub(super) fn commit_file(
    repo: &mut Repo,
    index: usize,
    name: &str,
    text: &str,
    time_s: i64,
) -> Result<(), Fail> {
    io_run(write_file(
        &format!("{}/{name}", repo.worktrees[index].path),
        text,
    ))?;
    let changes = vec![(name.to_string(), text.to_string())];
    make_commit(repo, index, changes, format!("add {name}"), time_s)
}

/// `add [-f] <path>...`: the files the next commit takes.
pub(super) fn stage(repo: &mut Repo, index: usize, args: &[String]) -> String {
    let staged = &mut repo.worktrees[index].staged;
    for path in args.iter().filter(|a| !a.starts_with('-')) {
        if !staged.contains(path) {
            staged.push(path.clone());
        }
    }
    String::new()
}

/// `commit -q -m <subject>`: the staged files as they are on disk now.
pub(super) fn commit_staged(repo: &mut Repo, index: usize, args: &[String]) -> Run {
    let subject = args
        .iter()
        .position(|a| a == "-m")
        .and_then(|at| args.get(at + 1))
        .ok_or_else(|| fatal("fake git: commit takes -m <subject>"))?;
    let wt = &repo.worktrees[index];
    if wt.staged.is_empty() {
        return Err(Fail {
            code: 1,
            msg: "nothing to commit\n".to_string(),
        });
    }
    let mut changes = Vec::new();
    for path in &wt.staged {
        let text =
            fs::read_to_string(format!("{}/{path}", wt.path)).map_err(|err| io_fatal(&err))?;
        changes.push((path.clone(), text));
    }
    repo.worktrees[index].staged.clear();
    let time_s = SystemClock.now_ms() / 1000;
    make_commit(repo, index, changes, subject.clone(), time_s)?;
    Ok(String::new())
}

struct AddArgs {
    branch: Option<String>,
    detach: bool,
    path: String,
    base: Option<String>,
}

fn add_args(args: &[String]) -> Result<AddArgs, Fail> {
    let (mut branch, mut detach, mut positional) = (None, false, Vec::new());
    let mut each = args.iter();
    while let Some(arg) = each.next() {
        match arg.as_str() {
            "-q" => {}
            "--detach" => detach = true,
            "-b" => branch = each.next().cloned(),
            _ => positional.push(arg.clone()),
        }
    }
    let mut positional = positional.into_iter();
    let path = positional
        .next()
        .map(|p| clean_path(&p))
        .ok_or_else(|| fatal("usage: git worktree add <path>"))?;
    Ok(AddArgs {
        branch,
        detach,
        path,
        base: positional.next(),
    })
}

fn add_head(repo: &mut Repo, add: &AddArgs, start: &str) -> Result<Head, Fail> {
    if add.detach {
        return Ok(Head::Detached(start.to_string()));
    }
    let existing = add
        .base
        .as_ref()
        .filter(|b| add.branch.is_none() && repo.branches.contains_key(*b));
    if let Some(name) = existing {
        let elsewhere = repo
            .worktrees
            .iter()
            .find(|w| w.head == Head::Branch(name.clone()));
        if let Some(other) = elsewhere {
            return Err(fatal(&format!(
                "'{name}' is already used by worktree at '{}'",
                other.path
            )));
        }
        return Ok(Head::Branch(name.clone()));
    }
    let name = add.branch.clone().unwrap_or_else(|| base_name(&add.path));
    if repo.branches.contains_key(&name) {
        return Err(fatal(&format!("a branch named '{name}' already exists")));
    }
    repo.branches.insert(name.clone(), start.to_string());
    Ok(Head::Branch(name))
}

/// `worktree add -q [-b <branch>] [--detach] <path> [<base>]`: the folder, its tracked files and git's records of it.
pub(super) fn worktree_add(repo: &mut Repo, from: usize, args: &[String]) -> Run {
    let add = add_args(args)?;
    let start = match &add.base {
        Some(rev) => repo
            .resolve(repo.worktrees.get(from), rev)
            .ok_or_else(|| fatal(&format!("invalid reference: {rev}")))?,
        None => repo.head_id(&repo.worktrees[from]),
    };
    let head = add_head(repo, &add, &start)?;
    let stem = base_name(&add.path);
    let mut name = stem.clone();
    let mut n = 1;
    while repo.worktrees.iter().any(|w| w.name == name) {
        name = format!("{stem}{n}");
        n += 1;
    }
    let files = repo
        .commits
        .get(&start)
        .map(|c| c.files.clone())
        .unwrap_or_default();
    let index = repo.worktrees.len();
    repo.worktrees.push(Wt {
        path: add.path.clone(),
        name,
        head: head.clone(),
        locked: None,
        staged: Vec::new(),
    });
    let git_dir = repo.worktrees[index].git_dir(repo.path());
    io_run(fs::create_dir_all(&add.path))?;
    for (rel, text) in &files {
        io_run(write_file(&format!("{}/{rel}", add.path), text))?;
    }
    io_run(write_file(
        &format!("{}/.git", add.path),
        &format!("gitdir: {git_dir}\n"),
    ))?;
    io_run(write_file(
        &format!("{git_dir}/gitdir"),
        &format!("{}/.git\n", add.path),
    ))?;
    io_run(write_file(&format!("{git_dir}/commondir"), "../..\n"))?;
    io_run(write_head(repo, index))?;
    io_run(write_index(repo, index))?;
    if let Head::Branch(branch) = &head {
        io_run(write_ref(repo, branch))?;
    }
    Ok(String::new())
}

/// `worktree remove [--force] <path>`: git's refusals, then the folder and the record.
pub(super) fn worktree_remove(repo: &mut Repo, args: &[String]) -> Run {
    let force = args.iter().any(|a| a == "--force" || a == "-f");
    let path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .ok_or_else(|| fatal("usage: git worktree remove <worktree>"))?;
    let index = repo
        .exact(path)
        .ok_or_else(|| fatal(&format!("'{path}' is not a working tree")))?;
    let wt = &repo.worktrees[index];
    if index == 0 {
        return Err(fatal(&format!("'{path}' is a main working tree")));
    }
    if let Some(reason) = &wt.locked {
        return Err(fatal(&format!(
            "cannot remove a locked working tree, lock reason: {reason}\nuse 'remove -f -f' to override or unlock first"
        )));
    }
    let there = Path::new(&wt.path).exists();
    if !force && there && !dirt_of(repo, wt).lines.is_empty() {
        return Err(fatal(&format!(
            "'{path}' contains modified or untracked files, use --force to delete it"
        )));
    }
    if there {
        RealFs.remove_tree(&wt.path).map_err(|err| Fail {
            code: 128,
            msg: format!("error: failed to delete '{path}': {err}\n"),
        })?;
    }
    let record = wt.git_dir(repo.path());
    let _ = fs::remove_dir_all(record);
    repo.worktrees.remove(index);
    Ok(String::new())
}

/// `worktree lock [--reason <r>] <path>` and `worktree unlock <path>`.
pub(super) fn worktree_lock(repo: &mut Repo, args: &[String], lock: bool) -> Run {
    let mut reason = String::new();
    let mut path = None;
    let mut each = args.iter();
    while let Some(arg) = each.next() {
        if arg == "--reason" {
            reason = each.next().cloned().unwrap_or_default();
        } else {
            path = Some(arg);
        }
    }
    let path = path.ok_or_else(|| fatal("usage: git worktree lock <path>"))?;
    let index = repo
        .exact(path)
        .ok_or_else(|| fatal(&format!("'{path}' is not a working tree")))?;
    repo.worktrees[index].locked = lock.then_some(reason);
    Ok(String::new())
}

/// `worktree prune [--dry-run --verbose]`: forgets the records whose `.git` file is gone and that are not locked.
pub(super) fn worktree_prune(repo: &mut Repo, args: &[String]) -> String {
    let dry = args.iter().any(|a| a == "--dry-run");
    let gone: Vec<usize> = repo
        .worktrees
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, w)| w.locked.is_none() && !Path::new(&format!("{}/.git", w.path)).exists())
        .map(|(i, _)| i)
        .collect();
    if dry {
        return gone.iter().fold(String::new(), |mut out, &i| {
            let _ = writeln!(
                out,
                "Removing worktrees/{}: gitdir file points to non-existent location",
                repo.worktrees[i].name
            );
            out
        });
    }
    for &i in gone.iter().rev() {
        let record = repo.worktrees[i].git_dir(repo.path());
        let _ = fs::remove_dir_all(record);
        repo.worktrees.remove(i);
    }
    String::new()
}

/// `update-ref -d refs/heads/<branch> <old>`: only while the branch still is `old`.
pub(super) fn update_ref(repo: &mut Repo, args: &[String]) -> Run {
    let [_, name, old] = args else {
        return Err(fatal("fake git: update-ref takes -d <ref> <old>"));
    };
    let branch = name.strip_prefix("refs/heads/").unwrap_or(name);
    match repo.branches.get(branch) {
        Some(now) if now == old => {}
        Some(now) => {
            return Err(fatal(&format!(
                "update_ref failed for ref '{name}': cannot lock ref '{name}': is at {now} but expected {old}"
            )));
        }
        None => {
            return Err(fatal(&format!(
                "update_ref failed for ref '{name}': unable to resolve reference '{name}'"
            )));
        }
    }
    repo.branches.remove(branch);
    repo.upstreams.remove(branch);
    let _ = fs::remove_file(format!("{}/.git/refs/heads/{branch}", repo.path()));
    Ok(String::new())
}

const BUNDLE_HEADER: &str = "# v2 git bundle";

/// `bundle create <file> <ref> [--not --remotes]` and `bundle verify <file>`. A bundle here is a header and the tip.
pub(super) fn bundle(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    if let [verb, file] = args {
        if verb != "verify" {
            return Err(fatal("fake git: bundle takes create or verify"));
        }
        let text = fs::read_to_string(file).unwrap_or_default();
        if text.starts_with(BUNDLE_HEADER) {
            return Ok("The bundle is okay\n".to_string());
        }
        return Err(fatal(&format!(
            "'{file}' does not look like a v2 or v3 bundle file"
        )));
    }
    let [_, file, rev, rest @ ..] = args else {
        return Err(fatal("Need a repository to create a bundle."));
    };
    let empty = || fatal("Refusing to create empty bundle.");
    let tip = repo.resolve(Some(wt), rev).ok_or_else(empty)?;
    let beyond_remotes = rest.iter().any(|a| a == "--remotes");
    if beyond_remotes
        && repo
            .remote_branches
            .values()
            .any(|r| repo.is_ancestor(&tip, r))
    {
        return Err(empty());
    }
    io_run(write_file(file, &format!("{BUNDLE_HEADER}\n{tip} {rev}\n")))?;
    Ok(String::new())
}

/// `diff HEAD --binary --output=<file>`: the new text of every tracked file that changed.
pub(super) fn diff(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    let file = args
        .iter()
        .find_map(|a| a.strip_prefix("--output="))
        .ok_or_else(|| fatal("fake git: diff takes --output=<file>"))?;
    let mut text = String::new();
    for path in dirt_of(repo, wt).changed {
        let _ = writeln!(text, "diff --git a/{path} b/{path}");
        match fs::read_to_string(format!("{}/{path}", wt.path)) {
            Ok(now) => now.lines().for_each(|line| {
                let _ = writeln!(text, "+{line}");
            }),
            Err(_) => text.push_str("deleted file mode 100644\n"),
        }
    }
    io_run(write_file(file, &text))?;
    Ok(String::new())
}

/// `remote add <name> <url>`.
pub(super) fn remote_add(repo: &mut Repo, args: &[String]) -> Run {
    let [_, name, url] = args else {
        return Err(fatal(
            "fake git: remote takes add <name> <url> or get-url <name>",
        ));
    };
    repo.remotes.insert(name.clone(), url.clone());
    Ok(String::new())
}
