//! From the facts a scan found to what the page shows and what the thread is told. Pure: the clock, the rule, the
//! keep and the owner's request come in as arguments.
//!
//! - `active`: touched (newest of the last commit and the index file) within `staleDays`
//! - `stale`: not touched for `staleDays` or more, not merged
//! - `merged`: stale age, HEAD merged into a default branch, clean: it can go
//! - `dirty-stale`: stale with uncommitted changes, never "can go", even when merged
//! - `missing`: git lists it but its folder is gone, never a doc
//! - `not-git`: a folder that is not a work tree but that a repository still lists
//! - `leftover`: no `.git`, no repository lists it: only files are left
//! - `error`: git could not be read there
//! - `kept`: one that needs a decision, snoozed by `coord worktree keep`

use crate::board::{Keep, RequestDoc};
use crate::clock::{DAY_MS, iso_from_ms};
use crate::doc::{KeptDoc, LeftoverDoc, Status, WorktreeDoc};
use crate::fact::Fact;
use crate::measure::{Leftover, size_text};
use crate::paths::base_name;
use crate::rule::{DEFAULT_STALE_DAYS, Rule};
use std::cmp::Ordering;

/// The wake engine clips a text to this.
pub const MAX_TEXT: usize = 300;
const MAX_LISTED: usize = 5;
/// What the wake tells a thread to do.
pub const WAKE_TAIL: &str =
    " Keep it (coord worktree keep), remove it (coord worktree remove), or drop its rule.";

/// When a worktree was last touched: the newest of the commit, the index file and the newest file of a leftover.
pub fn touched_of(fact: &Fact) -> Option<i64> {
    [
        fact.commit_ms,
        fact.index_ms,
        fact.leftover.as_ref().and_then(|l| l.newest_ms),
    ]
    .into_iter()
    .flatten()
    .filter(|t| *t > 0)
    .max()
}

/// One fact, judged.
#[derive(Clone, Debug, PartialEq)]
pub struct Judged {
    pub status: Status,
    pub age_days: Option<i64>,
    pub stale_days: u32,
    pub touched_ms: Option<i64>,
    pub kept: Option<Keep>,
}

/// What the judging needs besides the fact.
#[derive(Clone, Copy, Default)]
pub struct Judging<'a> {
    pub rule: Option<&'a Rule>,
    pub keep: Option<&'a Keep>,
    pub now_ms: i64,
    pub default_stale_days: Option<u32>,
}

fn plain_status(fact: &Fact, aged_out: bool) -> Status {
    if !fact.exists {
        Status::Missing
    } else if !fact.is_git {
        if fact.error.is_some() {
            Status::Error
        } else if fact.leftover.is_some() {
            Status::Leftover
        } else {
            Status::NotGit
        }
    } else if !aged_out {
        Status::Active
    } else if fact.main {
        Status::Stale
    } else if fact.dirty > 0 {
        Status::DirtyStale
    } else if fact.merged {
        Status::Merged
    } else {
        Status::Stale
    }
}

/// The status, age, stale days and keep of one fact.
pub fn classify(fact: &Fact, judging: &Judging<'_>) -> Judged {
    let stale_days = judging
        .rule
        .and_then(|r| r.stale_days)
        .or(judging.default_stale_days)
        .unwrap_or(DEFAULT_STALE_DAYS);
    let touched_ms = touched_of(fact);
    let age_ms = touched_ms.map_or(0, |t| (judging.now_ms - t).max(0));
    let age_days = touched_ms.map(|_| age_ms / DAY_MS);
    let status = plain_status(fact, age_ms >= i64::from(stale_days) * DAY_MS);
    let keepable = status.needs_decision() || status == Status::Missing;
    let kept = judging
        .keep
        .filter(|k| keepable && k.until_ms > judging.now_ms)
        .cloned();
    Judged {
        status: if kept.is_some() { Status::Kept } else { status },
        age_days,
        stale_days,
        touched_ms,
        kept,
    }
}

fn leftover_doc(l: &Leftover, touched_ms: Option<i64>, now_ms: i64) -> LeftoverDoc {
    LeftoverDoc {
        size_mb: l.size_mb,
        files: l.files,
        newest_at: touched_ms.map(iso_from_ms),
        age_days: touched_ms.map(|t| (now_ms - t).max(0) / DAY_MS),
        has_nested_git: l.has_nested_git,
        nested_git: l.nested_git.clone(),
        node_modules: l.node_modules,
        is_link: l.is_link,
        partial: l.partial,
        pending: l.pending,
        top_entries: l.top_entries.iter().take(8).cloned().collect(),
        made: false,
    }
}

/// The doc of a worktree.
pub fn doc_of(fact: &Fact, judging: &Judging<'_>, request: Option<RequestDoc>) -> WorktreeDoc {
    let judged = classify(fact, judging);
    WorktreeDoc {
        id: fact.id.clone(),
        path: fact.path.clone(),
        name: base_name(&fact.path),
        rule: fact.rule.clone(),
        thread: judging
            .rule
            .and_then(|r| r.thread.clone())
            .or_else(|| fact.thread.clone()),
        branch: fact.branch.clone(),
        detached: fact.detached,
        head: fact.head.as_ref().map(|h| h.chars().take(7).collect()),
        subject: fact.subject.clone(),
        last_commit_at: fact.commit_ms.map(iso_from_ms),
        last_touched_at: judged.touched_ms.map(iso_from_ms),
        age_days: judged.age_days,
        stale_days: judged.stale_days,
        dirty: fact.dirty,
        ahead: fact.ahead,
        behind: fact.behind,
        upstream: fact.upstream.clone(),
        merged: fact.merged,
        merged_into: fact.merged_into.clone(),
        referenced: fact.referenced,
        unmerged: fact.unmerged,
        unmerged_subjects: fact.unmerged_subjects.iter().take(3).cloned().collect(),
        prunable: fact.prunable.clone(),
        locked: fact.locked.clone(),
        main: fact.main,
        repo: fact.repo.clone(),
        status: judged.status,
        kept: judged.kept.as_ref().map(|k| KeptDoc {
            until: iso_from_ms(k.until_ms),
            note: k.note.clone(),
            by: k.by.clone(),
        }),
        request,
        leftover: fact
            .leftover
            .as_ref()
            .map(|l| leftover_doc(l, judged.touched_ms, judging.now_ms)),
        job: None,
        error: fact.error.clone(),
    }
}

/// Worst first: a missing one, then the oldest, then by path.
pub fn by_worst(a: &WorktreeDoc, b: &WorktreeDoc) -> Ordering {
    (b.status == Status::Missing)
        .cmp(&(a.status == Status::Missing))
        .then_with(|| b.age_days.unwrap_or(0).cmp(&a.age_days.unwrap_or(0)))
        .then_with(|| a.path.cmp(&b.path))
}

fn short_path(path: &str) -> String {
    format!(".../{}", base_name(path))
}

fn cell_of(d: &WorktreeDoc, path: &str) -> String {
    let where_ = d
        .branch
        .as_ref()
        .map(|b| format!("branch {b}"))
        .or_else(|| d.detached.then(|| "detached".to_string()))
        .unwrap_or_default();
    let age = d
        .age_days
        .map_or_else(|| "?".to_string(), |a| a.to_string());
    match d.status {
        Status::Missing => format!("{path} missing (git still lists it)"),
        Status::Merged => format!("{path} merged and clean (can go)"),
        Status::DirtyStale => {
            let place = if where_.is_empty() {
                String::new()
            } else {
                format!(" ({where_})")
            };
            format!("{path} stale {age}d with {} uncommitted{place}", d.dirty)
        }
        _ => {
            let ahead = (d.ahead > 0).then(|| format!("{} ahead", d.ahead));
            let extra = [Some(where_).filter(|w| !w.is_empty()), ahead]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
            let extra = if extra.is_empty() {
                String::new()
            } else {
                format!(" ({extra})")
            };
            format!("{path} stale {age}d{extra}")
        }
    }
}

fn clip(text: &str) -> String {
    text.chars().take(MAX_TEXT).collect()
}

/// `3 worktrees need a decision: X:/t/.claude/worktrees/foo stale 6d (branch foo, 2 ahead), .../bar merged and clean
/// (can go) (+1 more). Keep it ..., remove it ..., or drop its rule.` The text shrinks until it fits 300 characters; the
/// first worktree keeps its full path, the others a short one. `prefix` (the "too many" sentence) goes first.
pub fn wake_text(list: &[&WorktreeDoc], prefix: &str) -> String {
    let lead = if prefix.trim().is_empty() {
        String::new()
    } else {
        format!("{} ", prefix.trim())
    };
    if list.is_empty() {
        return clip(lead.trim());
    }
    let count = list.len();
    let head = format!(
        "{lead}{count} worktree{} a decision: ",
        if count == 1 { " needs" } else { "s need" }
    );
    let cells: Vec<String> = list
        .iter()
        .enumerate()
        .map(|(i, d)| {
            cell_of(
                d,
                &if i == 0 {
                    d.path.clone()
                } else {
                    short_path(&d.path)
                },
            )
        })
        .collect();
    for n in (1..=count.min(MAX_LISTED)).rev() {
        let more = if count > n {
            format!(" (+{} more)", count - n)
        } else {
            String::new()
        };
        let text = format!("{head}{}{more}.{WAKE_TAIL}", cells[..n].join(", "));
        if text.chars().count() <= MAX_TEXT {
            return text;
        }
    }
    let first = cell_of(list[0], &short_path(&list[0].path));
    let more = if count > 1 {
        format!(" (+{} more)", count - 1)
    } else {
        String::new()
    };
    clip(&format!("{head}{first}{more}.{WAKE_TAIL}"))
}

/// The sentence put first in the wake when a project holds more than `manyAt` live folders.
pub fn many_text(
    name: &str,
    count: usize,
    git: usize,
    leftovers: usize,
    leftover_mb: f64,
) -> String {
    if leftovers == 0 {
        return format!(
            "{name} has {count} worktrees: that is a lot, remove the ones you no longer need (coord worktree list, coord worktree remove)."
        );
    }
    let size = if leftover_mb >= 1.0 {
        format!(" (about {})", size_text(Some(leftover_mb)))
    } else {
        String::new()
    };
    let plural = |n: usize, one: &str| {
        if n == 1 {
            one.to_string()
        } else {
            format!("{one}s")
        }
    };
    format!(
        "{name} has {count} folders: {git} git {} and {leftovers} {}{size}: `coord worktree remove --all-leftovers` clears the old ones.",
        plural(git, "worktree"),
        plural(leftovers, "leftover")
    )
}
