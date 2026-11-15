//! The commands that only read: what the scan and the guards ask git. Each parses only the argument lists the crate sends.

use super::disk::{Dirt, Patterns, dirt};
use super::model::{Fail, Head, Repo, Run, Wt, fatal, quiet};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::Path;

/// The files a checkout's HEAD tracks, with their text.
pub(super) fn tracked_of(repo: &Repo, wt: &Wt) -> BTreeMap<String, String> {
    repo.commits
        .get(&repo.head_id(wt))
        .map(|c| c.files.clone())
        .unwrap_or_default()
}

/// What the disk of a checkout holds against its HEAD.
pub(super) fn dirt_of(repo: &Repo, wt: &Wt) -> Dirt {
    let patterns = Patterns::read(repo.path(), &wt.path);
    let mut found = dirt(&wt.path, &patterns, &tracked_of(repo, wt));
    super::submodule::adjust(wt, &mut found);
    found
}

fn unknown(rev: &str) -> Fail {
    fatal(&format!(
        "ambiguous argument '{rev}': unknown revision or path not in the working tree."
    ))
}

fn has(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

/// `rev-parse --verify -q <ref>`, `rev-parse --show-toplevel [--abbrev-ref HEAD] --git-dir`, `rev-parse <rev>...`.
pub(super) fn rev_parse(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    if has(args, "--verify") {
        let rev = args.last().map_or("", String::as_str);
        return repo
            .resolve(Some(wt), rev)
            .map(|id| format!("{id}\n"))
            .ok_or_else(|| quiet(1));
    }
    if !has(args, "--show-toplevel") {
        let mut out = String::new();
        for rev in args {
            let id = repo.resolve(Some(wt), rev).ok_or_else(|| unknown(rev))?;
            let _ = writeln!(out, "{id}");
        }
        return Ok(out);
    }
    let mut out = format!("{}\n", wt.path);
    if has(args, "--abbrev-ref") {
        if repo.head_id(wt).is_empty() {
            return Err(unknown("HEAD"));
        }
        let name = match &wt.head {
            Head::Branch(name) => name.as_str(),
            Head::Detached(_) => "HEAD",
        };
        let _ = writeln!(out, "{name}");
    }
    let git_dir = if wt.name.is_empty() {
        ".git".to_string()
    } else {
        wt.git_dir(repo.path())
    };
    let _ = writeln!(out, "{git_dir}");
    Ok(out)
}

fn upstream_line(repo: &Repo, name: &str, up: &str) -> String {
    let mine = repo.branches.get(name).cloned().unwrap_or_default();
    let Some(theirs) = repo.remote_branches.get(up) else {
        return format!("{name}...{up} [gone]");
    };
    let ahead = repo.beyond(theirs, &mine).len();
    let behind = repo.beyond(&mine, theirs).len();
    let counts = match (ahead, behind) {
        (0, 0) => String::new(),
        (a, 0) => format!(" [ahead {a}]"),
        (0, b) => format!(" [behind {b}]"),
        (a, b) => format!(" [ahead {a}, behind {b}]"),
    };
    format!("{name}...{up}{counts}")
}

/// `status --porcelain=v1 [-b]`: the branch line only with `-b`.
pub(super) fn status(repo: &Repo, wt: &Wt, args: &[String]) -> String {
    let head = match &wt.head {
        Head::Detached(_) => "HEAD (no branch)".to_string(),
        Head::Branch(name) => repo
            .upstreams
            .get(name)
            .map_or_else(|| name.clone(), |up| upstream_line(repo, name, up)),
    };
    let mut out = if has(args, "-b") {
        format!("## {head}\n")
    } else {
        String::new()
    };
    for line in dirt_of(repo, wt).lines {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// The commits a revision or a `from..to` range names, newest first.
fn range(repo: &Repo, wt: &Wt, spec: &str) -> Result<Vec<String>, Fail> {
    let one = |rev: &str| repo.resolve(Some(wt), rev).ok_or_else(|| unknown(rev));
    match spec.split_once("..") {
        Some((from, to)) => {
            let to = if to.is_empty() { "HEAD" } else { to };
            Ok(repo.beyond(&one(from)?, &one(to)?))
        }
        None => Ok(repo.chain(&one(spec)?)),
    }
}

fn render(repo: &Repo, id: &str, format: &str) -> String {
    let (time, subject) = repo
        .commits
        .get(id)
        .map_or((0, ""), |c| (c.time_s, c.subject.as_str()));
    format
        .replace("%x1f", "\x1f")
        .replace("%H", id)
        .replace("%ct", &time.to_string())
        .replace("%s", subject)
}

/// `log -N --format=<f> [<rev>|<from>..<to>]`.
pub(super) fn log(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    let (mut limit, mut format, mut spec) = (usize::MAX, "%H".to_string(), "HEAD".to_string());
    for arg in args {
        if let Some(f) = arg.strip_prefix("--format=") {
            format = f.to_string();
        } else if let Some(n) = arg.strip_prefix('-').and_then(|n| n.parse().ok()) {
            limit = n;
        } else {
            spec.clone_from(arg);
        }
    }
    let mut out = String::new();
    for id in range(repo, wt, &spec)?.iter().take(limit) {
        out.push_str(&render(repo, id, &format));
        out.push('\n');
    }
    Ok(out)
}

/// `rev-list --count <range>`.
pub(super) fn rev_list(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    let spec = args.last().map_or("HEAD", String::as_str);
    Ok(format!("{}\n", range(repo, wt, spec)?.len()))
}

/// `merge-base --is-ancestor <old> <tip>`.
pub(super) fn merge_base(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    let [_, old, tip] = args else {
        return Err(fatal(
            "fake git: merge-base takes --is-ancestor <old> <tip>",
        ));
    };
    let old = repo.resolve(Some(wt), old).ok_or_else(|| unknown(old))?;
    let tip = repo.resolve(Some(wt), tip).ok_or_else(|| unknown(tip))?;
    if repo.is_ancestor(&old, &tip) {
        Ok(String::new())
    } else {
        Err(quiet(1))
    }
}

/// `for-each-ref --contains <rev> --count=1 --format=%(refname) <prefix>...`.
pub(super) fn for_each_ref(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    let rev = args
        .iter()
        .position(|a| a == "--contains")
        .and_then(|at| args.get(at + 1))
        .ok_or_else(|| fatal("fake git: for-each-ref takes --contains <rev>"))?;
    let rev = repo.resolve(Some(wt), rev).ok_or_else(|| unknown(rev))?;
    let wanted: Vec<&String> = args.iter().filter(|a| a.starts_with("refs/")).collect();
    let tables = [
        ("refs/heads/", &repo.branches),
        ("refs/remotes/", &repo.remote_branches),
        ("refs/tags/", &repo.tags),
    ];
    let mut hits: Vec<String> = Vec::new();
    for (prefix, table) in tables {
        if !wanted.iter().any(|w| prefix.starts_with(w.as_str())) {
            continue;
        }
        for (name, tip) in table {
            if repo.is_ancestor(&rev, tip) {
                hits.push(format!("{prefix}{name}"));
            }
        }
    }
    hits.sort();
    Ok(hits.first().map(|h| format!("{h}\n")).unwrap_or_default())
}

/// `cherry <base> <ref>`: no patch ids here, so every commit beyond the base is one the base lacks.
pub(super) fn cherry(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    let [base, name, ..] = args else {
        return Err(fatal("fake git: cherry takes <base> <ref>"));
    };
    let base = repo.resolve(Some(wt), base).ok_or_else(|| unknown(base))?;
    let tip = repo.resolve(Some(wt), name).ok_or_else(|| unknown(name))?;
    let mut ids = repo.beyond(&base, &tip);
    ids.reverse();
    Ok(ids.iter().fold(String::new(), |mut out, id| {
        let _ = writeln!(out, "+ {id}");
        out
    }))
}

/// `symbolic-ref -q --short refs/remotes/origin/HEAD`.
pub(super) fn symbolic_ref(repo: &Repo) -> Run {
    repo.origin_head
        .as_ref()
        .map(|head| format!("{head}\n"))
        .ok_or_else(|| quiet(1))
}

/// `config --get init.defaultBranch`.
pub(super) fn config(repo: &Repo, args: &[String]) -> Run {
    match (args.get(1).map(String::as_str), &repo.init_default) {
        (Some("init.defaultBranch"), Some(name)) => Ok(format!("{name}\n")),
        _ => Err(quiet(1)),
    }
}

/// `remote get-url <name>`.
pub(super) fn remote_get_url(repo: &Repo, args: &[String]) -> Run {
    let name = args.get(1).map_or("origin", String::as_str);
    repo.remotes
        .get(name)
        .map(|url| format!("{url}\n"))
        .ok_or_else(|| Fail {
            code: 2,
            msg: format!("error: No such remote '{name}'\n"),
        })
}

/// `worktree list --porcelain`.
pub(super) fn worktree_list(repo: &Repo) -> String {
    let mut out = String::new();
    for (index, wt) in repo.worktrees.iter().enumerate() {
        let _ = write!(out, "worktree {}\nHEAD {}\n", wt.path, repo.head_id(wt));
        match &wt.head {
            Head::Branch(name) => {
                let _ = writeln!(out, "branch refs/heads/{name}");
            }
            Head::Detached(_) => out.push_str("detached\n"),
        }
        if let Some(reason) = &wt.locked {
            out.push_str(format!("locked {reason}").trim_end());
            out.push('\n');
        } else if index > 0 && !Path::new(&format!("{}/.git", wt.path)).exists() {
            out.push_str("prunable gitdir file points to non-existent location\n");
        }
        out.push('\n');
    }
    out
}

/// `ls-files --others --exclude-standard -z` and `ls-files --cached -z -- <path>...` (a path may carry `:(literal)`):
/// the tracked files that are a path or lie under one.
pub(super) fn ls_files(repo: &Repo, wt: &Wt, args: &[String]) -> String {
    let names: Vec<String> = if has(args, "--others") {
        dirt_of(repo, wt).others
    } else {
        let wanted: Vec<&str> = args
            .iter()
            .skip_while(|a| *a != "--")
            .skip(1)
            .map(|a| a.strip_prefix(":(literal)").unwrap_or(a.as_str()))
            .collect();
        tracked_of(repo, wt)
            .into_keys()
            .filter(|k| {
                wanted
                    .iter()
                    .any(|w| k.as_str() == *w || k.starts_with(&format!("{w}/")))
            })
            .collect()
    };
    names.iter().fold(String::new(), |mut out, name| {
        let _ = write!(out, "{name}\0");
        out
    })
}

/// `check-ignore -q <folder>`: exit 0 when the folder is ignored, 1 when it is not.
pub(super) fn check_ignore(repo: &Repo, wt: &Wt, args: &[String]) -> Run {
    let path = args.last().map_or("", String::as_str);
    if Patterns::read(repo.path(), &wt.path).ignored(path, true) {
        Ok(String::new())
    } else {
        Err(quiet(1))
    }
}
