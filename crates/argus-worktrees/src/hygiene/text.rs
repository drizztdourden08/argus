//! The sentences of the hygiene: the lead's approval ask (an inbox note and a short wake) and the report to the
//! Overlord.

use super::memory::Mark;
use crate::paths::base_name;
use crate::status::MAX_TEXT;

/// The wake reason of the lead's approval ask.
pub const APPROVE_REASON: &str = "worktree-approve";
/// The wake reason of the report to the Overlord.
pub const REPORT_REASON: &str = "worktree-report";
const LISTED: usize = 4;

/// `30 minutes`, `1 hour`, `2 hours`.
pub(crate) fn span_text(ms: i64) -> String {
    let minutes = (ms + 30_000) / 60_000;
    if minutes >= 60 && minutes % 60 == 0 {
        let hours = minutes / 60;
        format!("{hours} hour{}", if hours == 1 { "" } else { "s" })
    } else {
        format!("{minutes} minute{}", if minutes == 1 { "" } else { "s" })
    }
}

fn what_left(mark: &Mark) -> &str {
    mark.why.as_deref().unwrap_or("work would be lost")
}

/// A lock stops even an approved removal: the ask says to lift it first.
fn unlock_hint(mark: &Mark) -> String {
    if what_left(mark).starts_with("it is locked") {
        format!(
            " It is locked: unlock it first (git worktree unlock {}).",
            mark.path
        )
    } else {
        String::new()
    }
}

/// One worktree of the ask: what is left, the approve command and the way to keep it.
fn ask_line(project: &str, id: &str, mark: &Mark) -> String {
    format!(
        "- {id} {} ({}): {}.{} Approve: coord worktree approve {id} --project {project} (full backup, then removed with what is left). Keep: coord lead portfolio \"keep worktree {id} of {project}: <why>\".",
        mark.path,
        mark.branch.as_deref().unwrap_or("detached"),
        what_left(mark),
        unlock_hint(mark)
    )
}

/// The inbox note of the ask: every worktree with its commands.
pub(crate) fn ask_note(project: &str, list: &[(&String, &Mark)], report_after_ms: i64) -> String {
    let (count, s) = (list.len(), if list.len() == 1 { "" } else { "s" });
    let lines: Vec<String> = list
        .iter()
        .map(|(id, mark)| ask_line(project, id, mark))
        .collect();
    format!(
        "Argus: {count} worktree{s} of {project} with nothing attached still hold work. Approve the removal or ask the Overlord for an exception; unanswered in {} and still unattached, it goes to the Overlord for deletion.\n{}",
        span_text(report_after_ms),
        lines.join("\n")
    )
}

fn cell(id: &str, mark: &Mark) -> String {
    format!("{id} .../{}", base_name(&mark.path))
}

/// `2 worktrees with nothing attached hold work: approve or ask the Overlord to keep them (commands in your inbox):
/// <id> .../agent-a, <id> .../agent-b.` Shrinks until it fits the wake engine's 300 characters.
pub(crate) fn ask_wake(list: &[(&String, &Mark)]) -> String {
    let count = list.len();
    let (are, them) = if count == 1 {
        ("worktree with nothing attached holds", "it")
    } else {
        ("worktrees with nothing attached hold", "them")
    };
    let head = format!(
        "{count} {are} work: approve the removal or ask the Overlord to keep {them} (commands in your inbox): "
    );
    for shown in (1..=count.min(LISTED)).rev() {
        let cells: Vec<String> = list[..shown].iter().map(|(id, m)| cell(id, m)).collect();
        let more = if count > shown {
            format!(" (+{} more)", count - shown)
        } else {
            String::new()
        };
        let text = format!("{head}{}{more}.", cells.join(", "));
        if text.chars().count() <= MAX_TEXT || shown == 1 {
            return text.chars().take(MAX_TEXT).collect();
        }
    }
    head.chars().take(MAX_TEXT).collect()
}

/// What the Overlord is told of one worktree.
pub(crate) struct Reported<'a> {
    pub project: &'a str,
    pub id: &'a str,
    pub mark: &'a Mark,
    /// Where its full backup goes, up to the time stamp.
    pub backup: String,
}

/// One worktree of the report: where it is, what is left, the backup and the one command that deletes it.
fn report_line(r: &Reported<'_>) -> String {
    let lead = r.mark.thread.as_deref().unwrap_or("no lead");
    format!(
        "- {} of {} at {} (lead {lead}): {}. Backup first: {}<time>.bundle/.patch. Delete: coord worktree approve {} --project {}.{}",
        r.id,
        r.project,
        r.mark.path,
        what_left(r.mark),
        r.backup,
        r.id,
        r.project,
        unlock_hint(r.mark)
    )
}

/// The Overlord's inbox note: the worktrees whose lead did not answer.
pub(crate) fn report_note(list: &[Reported<'_>], report_after_ms: i64) -> String {
    let (count, s) = (list.len(), if list.len() == 1 { "" } else { "s" });
    let lines: Vec<String> = list.iter().map(report_line).collect();
    format!(
        "Argus: {count} worktree{s} with nothing attached still hold work and the lead did not answer the ask in {}. Delete each now (the command writes a full backup first), or put it on the exception list (coord worktree keep <id> --project <p> --note \"why\").\n{}",
        span_text(report_after_ms),
        lines.join("\n")
    )
}

/// The Overlord's wake.
pub(crate) fn report_wake(list: &[Reported<'_>]) -> String {
    let names: Vec<String> = list
        .iter()
        .take(LISTED)
        .map(|r| format!("{} {}", r.project, cell(r.id, r.mark)))
        .collect();
    let more = list.len().saturating_sub(LISTED);
    let tail = if more > 0 {
        format!(" (+{more} more)")
    } else {
        String::new()
    };
    let text = format!(
        "{} unattached worktree{} with work left, lead did not answer: delete or keep (commands in your inbox): {}{tail}.",
        list.len(),
        if list.len() == 1 { "" } else { "s" },
        names.join(", ")
    );
    text.chars().take(MAX_TEXT).collect()
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hygiene::HygieneState;

    fn mark(path: &str, why: &str) -> Mark {
        Mark {
            state: HygieneState::AwaitingLead,
            path: path.into(),
            branch: Some("feat/x".into()),
            thread: Some("lead".into()),
            agent: Some("A-3".into()),
            why: Some(why.into()),
            since: 0,
            asked: None,
            reported: None,
            request: None,
        }
    }

    #[test]
    fn the_ask_names_the_commands_and_the_wake_fits() {
        let id = "1a2b3c4d5e".to_string();
        let m = mark("X:/p/.claude/worktrees/agent-a", "it has 2 uncommitted changes");
        let note = ask_note("alpha", &[(&id, &m)], 1_800_000);
        assert!(note.contains("unanswered in 30 minutes"), "{note}");
        assert!(note.contains("coord worktree approve 1a2b3c4d5e --project alpha"), "{note}");
        assert!(note.contains("coord lead portfolio \"keep worktree 1a2b3c4d5e of alpha"), "{note}");
        let many: Vec<(String, Mark)> = (0..9)
            .map(|n| {
                (
                    format!("{n:010}"),
                    mark(&format!("X:/a/very/long/path/to/the/worktrees/agent-{n}"), "x"),
                )
            })
            .collect();
        let refs: Vec<(&String, &Mark)> = many.iter().map(|(i, m)| (i, m)).collect();
        let wake = ask_wake(&refs);
        assert!(wake.chars().count() <= MAX_TEXT, "{wake}");
        assert!(wake.contains("more)"), "{wake}");
    }

    #[test]
    fn a_locked_worktree_is_told_to_be_unlocked_first() {
        let m = mark("X:/p/w", "it is locked (moving)");
        assert!(ask_line("alpha", "w1", &m).contains("git worktree unlock X:/p/w"));
    }
}
