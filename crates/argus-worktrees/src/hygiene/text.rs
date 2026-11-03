//! The sentences of the hygiene: the `worktree-review` wake and the inbox note of what went.

use super::memory::Mark;
use crate::paths::base_name;
use crate::status::MAX_TEXT;

/// The wake reason for released worktrees.
pub const REASON: &str = "worktree-review";
const LISTED: usize = 4;

/// `1 hour`, `90 minutes`, `2 hours`.
pub(crate) fn grace_text(ms: i64) -> String {
    let minutes = (ms + 30_000) / 60_000;
    if minutes >= 60 && minutes % 60 == 0 {
        let hours = minutes / 60;
        format!("{hours} hour{}", if hours == 1 { "" } else { "s" })
    } else {
        format!("{minutes} minute{}", if minutes == 1 { "" } else { "s" })
    }
}

fn cell(id: &str, mark: &Mark, full: bool) -> String {
    let path = if full {
        mark.path.clone()
    } else {
        format!(".../{}", base_name(&mark.path))
    };
    let agent = mark
        .agent
        .as_deref()
        .map(|a| format!(" ({a})"))
        .unwrap_or_default();
    format!("{id} {path}{agent}")
}

/// `2 worktrees are released: Argus removes them in 1 hour unless you keep one (...) or release
/// it now (...): <id> <path> (A-3), <id> .../agent-b (A-4).` Shrinks until it fits the wake engine's 300 characters.
pub(crate) fn review_text(list: &[(&String, &Mark)], grace_ms: i64) -> String {
    let count = list.len();
    let (are, them) = if count == 1 {
        ("worktree is", "it")
    } else {
        ("worktrees are", "them")
    };
    let head = format!(
        "{count} {are} released: Argus removes {them} in {} unless you keep one (coord worktree keep <id> --note \"why\") or release it now (coord worktree release <id>): ",
        grace_text(grace_ms)
    );
    for shown in (1..=count.min(LISTED)).rev() {
        let cells: Vec<String> = list[..shown]
            .iter()
            .enumerate()
            .map(|(i, (id, mark))| cell(id, mark, i == 0))
            .collect();
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

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hygiene::HygieneState;

    fn mark(path: &str) -> Mark {
        Mark {
            state: HygieneState::Released,
            path: path.into(),
            branch: None,
            thread: Some("lead".into()),
            agent: Some("A-3".into()),
            why: None,
            item: None,
            since: 0,
            told: None,
            request: None,
        }
    }

    #[test]
    fn the_wake_names_the_worktrees_and_the_way_out_and_fits() {
        let (a, b) = ("1a2b3c4d5e".to_string(), "2b3c4d5e6f".to_string());
        let (ma, mb) = (
            mark("X:/p/.claude/worktrees/agent-a"),
            mark("X:/p/.claude/worktrees/agent-b"),
        );
        let one = review_text(&[(&a, &ma)], 3_600_000);
        assert!(
            one.starts_with("1 worktree is released: Argus removes it in 1 hour"),
            "{one}"
        );
        assert!(
            one.ends_with("1a2b3c4d5e X:/p/.claude/worktrees/agent-a (A-3)."),
            "{one}"
        );
        let two = review_text(&[(&a, &ma), (&b, &mb)], 5_400_000);
        assert!(two.contains("removes them in 90 minutes"), "{two}");
        let many: Vec<(String, Mark)> = (0..9)
            .map(|n| {
                (
                    format!("{n:010}"),
                    mark(&format!("X:/a/very/long/path/to/the/worktrees/agent-{n}")),
                )
            })
            .collect();
        let refs: Vec<(&String, &Mark)> = many.iter().map(|(i, m)| (i, m)).collect();
        let text = review_text(&refs, 3_600_000);
        assert!(text.chars().count() <= MAX_TEXT, "{text}");
        assert!(text.contains("more)"), "{text}");
    }
}
