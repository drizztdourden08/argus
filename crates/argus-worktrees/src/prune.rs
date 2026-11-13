//! `git worktree prune`. Git forgets EVERY prunable record of a repository, also a worktree no rule vetted (a drive that
//! is unplugged right now), and with a record goes its HEAD, so commits on a detached HEAD become unreachable. So
//! `git worktree prune --dry-run --verbose` names what it would forget first, and the prune runs only when every one of
//! those records is a gone worktree of this project that passes its rule and loses no commit (or whose loss the owner
//! approved).

use crate::error::{ErrorCode, Result, WtError};
use crate::exec::GitOutput;
use crate::fact::Fact;
use crate::inspect::rev_kept;
use crate::parse::parse_worktree_list;
use crate::paths::{base_name, clean_path, join_path, same_path};
use crate::remove::{apply_branch, forbidden, plural, short7};
use crate::request::{RemoveInput, RemoveOutcome};
use crate::service::Service;
use crate::{
    board::{BoardFacts, Caller},
    branch::BranchOutcome,
};

/// A record `git worktree prune` would forget: its name under `.git/worktrees` and the folder it stood for, when known.
pub(crate) struct Record {
    pub(crate) name: String,
    pub(crate) path: Option<String>,
}

fn is_removing(line: &str) -> bool {
    line.strip_prefix("Removing")
        .is_some_and(|rest| !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
}

fn forgotten_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("Removing worktrees/")?;
    rest.find(": ").filter(|at| *at > 0).map(|at| &rest[..at])
}

/// The record names a dry run lists; a line Argus cannot read stops the prune.
fn dry_run_names(repo: &str, out: &GitOutput) -> Result<Vec<String>> {
    let text = format!("{}\n{}", out.stdout, out.stderr);
    let mut names: Vec<String> = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| is_removing(l)) {
        let Some(name) = forgotten_name(line) else {
            return Err(WtError::new(
                ErrorCode::GitFailed,
                format!(
                    "could not read what git worktree prune would forget in {repo} (\"{}\"); nothing was pruned",
                    line.chars().take(100).collect::<String>()
                ),
            ));
        };
        if !names.iter().any(|n| n == name) {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

/// The folder a worktree record stood for, from its `gitdir` file (it names `<folder>/.git`).
fn folder_of_gitdir(text: &str) -> String {
    let path = clean_path(text.trim());
    match path.len().checked_sub(5) {
        Some(at) if path.is_char_boundary(at) && path[at..].eq_ignore_ascii_case("/.git") => {
            path[..at].to_string()
        }
        _ => path,
    }
}

impl Service {
    /// The records `git worktree prune` would forget in `repo`.
    pub(crate) fn prune_records(&self, repo: &str) -> Result<Vec<Record>> {
        let dry = self
            .deps
            .git
            .write(repo, &["worktree", "prune", "--dry-run", "--verbose"]);
        if !dry.ok() {
            return Err(WtError::new(
                ErrorCode::GitFailed,
                format!(
                    "git worktree prune --dry-run failed in {repo}: {}; nothing was pruned",
                    dry.failure_text()
                ),
            ));
        }
        let names = dry_run_names(repo, &dry)?;
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let listing = self
            .deps
            .git
            .read(repo, &["worktree", "list", "--porcelain"]);
        if !listing.ok() {
            return Err(WtError::new(
                ErrorCode::GitFailed,
                format!(
                    "git worktree list failed in {repo}: {}; nothing was pruned",
                    listing.failure_text()
                ),
            ));
        }
        let items = parse_worktree_list(&listing.stdout);
        Ok(names
            .into_iter()
            .map(|name| {
                let file = join_path(repo, &format!(".git/worktrees/{name}/gitdir"));
                let path = self
                    .deps
                    .fs
                    .read_text(&file)
                    .ok()
                    .map(|text| folder_of_gitdir(&text))
                    .or_else(|| {
                        items
                            .iter()
                            .skip(1)
                            .find(|i| i.prunable.is_some() && base_name(&i.path) == name)
                            .map(|i| i.path.clone())
                    });
                Record { name, path }
            })
            .collect())
    }

    /// Refuses (forbidden) unless pruning `repo` forgets only records of `allowed` folders.
    pub(crate) fn vet_prune(&self, repo: &str, allowed: &[&str]) -> Result<()> {
        let ci = self.ci();
        let vetted = |r: &Record| {
            r.path
                .as_deref()
                .is_some_and(|p| allowed.iter().any(|a| same_path(a, p, ci)))
        };
        let extra: Vec<Record> = self
            .prune_records(repo)?
            .into_iter()
            .filter(|r| !vetted(r))
            .collect();
        if extra.is_empty() {
            return Ok(());
        }
        let names: String = extra
            .iter()
            .map(|r| {
                r.path
                    .clone()
                    .unwrap_or_else(|| format!("worktrees/{}", r.name))
            })
            .collect::<Vec<_>>()
            .join(", ")
            .chars()
            .take(300)
            .collect();
        Err(forbidden(format!(
            "git worktree prune would also forget {} of {repo} that no rule of this project vetted ({names}): nothing was pruned. Restore the folder or drive, or run git worktree prune there yourself",
            plural(
                u32::try_from(extra.len()).unwrap_or(u32::MAX),
                "other worktree record"
            )
        )))
    }

    pub(crate) fn run_prune(&self, repo: &str) -> Result<()> {
        let run = self.deps.git.write(repo, &["worktree", "prune"]);
        if run.ok() {
            return Ok(());
        }
        Err(WtError::new(
            ErrorCode::GitFailed,
            format!(
                "git worktree prune failed in {repo}: {}",
                run.failure_text()
            ),
        ))
    }

    /// `git worktree prune` in a repository, but only when every record it would forget is one of `allowed`.
    pub(crate) fn prune_repo(&self, repo: &str, allowed: &[&str]) -> Result<()> {
        self.vet_prune(repo, allowed)?;
        self.run_prune(repo)
    }

    /// The gone worktrees of the project that a prune may forget: under a valid rule, not locked, folder still gone, and
    /// no commit lost.
    pub(crate) fn vetted_gone(&self, project: &str, board: &BoardFacts) -> Vec<Fact> {
        let vetted = |f: &Fact| {
            f.locked.is_none()
                && !self.deps.fs.exists(&f.path)
                && self.rule_of(board, f).is_ok()
                && !self.loses_commits(f)
        };
        self.gone_of(project).into_iter().filter(vetted).collect()
    }

    /// A gone worktree on a detached HEAD that no branch, tag or remote keeps: pruning forgets the only record of it.
    fn loses_commits(&self, fact: &Fact) -> bool {
        let (Some(repo), Some(head)) = (fact.repo.as_deref(), fact.head.as_deref()) else {
            return fact.detached;
        };
        fact.detached && rev_kept(&self.deps.git, repo, head) != Some(true)
    }

    /// A folder that is gone: only git's record is left.
    pub(crate) fn prune_gone(&self, input: &RemoveInput, fact: &Fact) -> Result<RemoveOutcome> {
        if fact.exists {
            return Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} is not gone: remove it with coord worktree remove (prune is for folders that no longer exist)",
                    fact.path
                ),
            ));
        }
        if self.deps.fs.exists(&fact.path) {
            self.request_rescan(&input.project);
            return Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} exists again since the listing: nothing was pruned, the listing is being refreshed",
                    fact.path
                ),
            ));
        }
        self.authorize_prune(input, fact)?;
        let repo = Self::git_repo(fact)?;
        let board = self.board(&input.project)?;
        let mut vetted: Vec<String> = self
            .vetted_gone(&input.project, &board)
            .into_iter()
            .map(|f| f.path)
            .collect();
        vetted.push(fact.path.clone());
        self.prune_repo(repo, &vetted.iter().map(String::as_str).collect::<Vec<_>>())?;
        let plan = self.plan_branch(input.branch, input.discard, fact);
        let branch = if input.branch {
            self.delete_branch(fact, &plan)
        } else {
            BranchOutcome::default()
        };
        self.forget(&input.project, &fact.id);
        let mut out = RemoveOutcome {
            removed: true,
            pruned_only: true,
            path: Some(fact.path.clone()),
            branch: fact.branch.clone(),
            pruned: vec![fact.path.clone()],
            ..RemoveOutcome::default()
        };
        apply_branch(&mut out, &branch);
        out.text = out.sentence();
        Ok(out)
    }

    /// Pruning needs no say, except for a detached HEAD no ref keeps: that needs `discard` and the owner's say. `discard`
    /// means nothing else here: a branch of a gone worktree is only deleted when it is merged.
    fn authorize_prune(&self, input: &RemoveInput, fact: &Fact) -> Result<()> {
        if !self.loses_commits(fact) {
            if input.discard {
                return Err(forbidden(format!(
                    "{} is gone; its branch is only deleted when it is merged, never discarded from here",
                    fact.path
                )));
            }
            return Ok(());
        }
        let what = format!(
            "the commit {} of {} is on a detached HEAD that no branch keeps",
            short7_or_unknown(fact.head.as_deref()),
            fact.path
        );
        if !input.discard {
            return Err(forbidden(format!(
                "{} is gone, and {what}: pruning forgets it and the commit becomes unreachable. Nothing was pruned. The owner can approve it (--discard --owner-entry <id> with his entry that asks for it)",
                fact.path
            )));
        }
        if input.caller != Caller::Page {
            let board = self.board(&input.project)?;
            let entry = input.owner_entry.as_deref().unwrap_or_default();
            if let Some(why) = crate::approval::approval_of(&board, entry, fact, self.now()) {
                return Err(forbidden(format!(
                    "discarding {what} needs the owner: {why}"
                )));
            }
        }
        Ok(())
    }

    /// Every gone worktree of the project: one `git worktree prune` per repository, every repository vetted first.
    pub(crate) fn prune_all(&self, input: &RemoveInput) -> Result<RemoveOutcome> {
        let board = self.board(&input.project)?;
        let rules = board.valid_rules();
        crate::guards::check_caller(&board, &input.caller, &rules)?;
        let gone = self.vetted_gone(&input.project, &board);
        let mut repos: Vec<&str> = gone.iter().filter_map(|f| f.repo.as_deref()).collect();
        repos.sort_unstable();
        repos.dedup();
        let vetted: Vec<&str> = gone.iter().map(|f| f.path.as_str()).collect();
        for repo in &repos {
            self.vet_prune(repo, &vetted)?;
        }
        for repo in &repos {
            self.run_prune(repo)?;
        }
        for fact in &gone {
            self.forget(&input.project, &fact.id);
        }
        Ok(pruned_outcome(&gone))
    }
}

fn short7_or_unknown(head: Option<&str>) -> String {
    let sha = short7(head);
    if sha.is_empty() {
        "(unknown)".to_string()
    } else {
        sha
    }
}

fn pruned_outcome(gone: &[Fact]) -> RemoveOutcome {
    let names: String = gone
        .iter()
        .map(|f| base_name(&f.path))
        .collect::<Vec<_>>()
        .join(", ")
        .chars()
        .take(200)
        .collect();
    let text = if gone.is_empty() {
        "nothing to prune: no gone worktree".to_string()
    } else {
        format!(
            "pruned {} ({names})",
            plural(
                u32::try_from(gone.len()).unwrap_or(u32::MAX),
                "gone worktree"
            )
        )
    };
    RemoveOutcome {
        removed: !gone.is_empty(),
        pruned_only: true,
        pruned: gone.iter().map(|f| f.path.clone()).collect(),
        text,
        ..RemoveOutcome::default()
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn dry(stderr: &str) -> GitOutput {
        GitOutput {
            code: Some(0),
            stderr: stderr.to_string(),
            ..GitOutput::default()
        }
    }

    #[test]
    fn the_names_of_a_dry_run_are_read_once_each() {
        let text = "Removing worktrees/a: gitdir file points to non-existent location\nRemoving worktrees/b: invalid gitdir file\nRemoving worktrees/a: again\nnoise\n";
        assert_eq!(dry_run_names("/r", &dry(text)).unwrap(), ["a", "b"]);
        let none = Vec::<String>::new();
        assert_eq!(dry_run_names("/r", &dry("")).unwrap(), none);
        assert_eq!(
            dry_run_names(
                "/r",
                &dry("Removed nothing
")
            )
            .unwrap(),
            none
        );
    }

    #[test]
    fn a_removing_line_that_cannot_be_read_stops_the_prune() {
        let err = dry_run_names("/r", &dry("Removing something else\n")).unwrap_err();
        assert_eq!(err.code, ErrorCode::GitFailed);
        assert!(
            err.message
                .contains("could not read what git worktree prune would forget in /r")
                && err.message.contains("nothing was pruned"),
            "{}",
            err.message
        );
    }

    #[test]
    fn the_folder_of_a_record_is_its_gitdir_without_the_dot_git() {
        let windows = "X:/w/a/.git\n".replace('/', "\u{5c}");
        assert_eq!(folder_of_gitdir(&windows), "X:/w/a");
        assert_eq!(folder_of_gitdir("/w/a/.GIT"), "/w/a");
        assert_eq!(folder_of_gitdir("/w/a"), "/w/a");
    }
}
