//! The read-only allow list of the shadow. Every ask the project's own rules do not settle comes to
//! the permission tool; the shadow answers it here, with no owner toast.

use argus_core::lead::{LEAD_ROLE, LEGACY};
use serde_json::Value;

use super::THREAD;

/// The message of a denial.
pub const DENIED: &str = "shadow mode: only reading is allowed; say what you would do instead";

const READ_TOOLS: [&str; 3] = ["Read", "Grep", "Glob"];
const DASHBOARD: &str = "mcp__coord-dashboard__";
const DASHBOARD_READS: [&str; 15] = [
    "board_summary",
    "get_item",
    "get_test",
    "health",
    "list_agents",
    "list_dependencies",
    "list_entries",
    "list_items",
    "list_launchers",
    "list_tests",
    "list_threads",
    "portfolio_overview",
    "run_logs",
    "run_status",
    "search",
];

/// The `coord` groups and the sub-verbs of each that only read.
const READ_VERBS: [(&str, &[&str]); 16] = [
    ("item", &["list", "show"]),
    ("thread", &["list", "show"]),
    ("entry", &["list", "show"]),
    ("test", &["list", "show"]),
    ("agent", &["list", "show"]),
    ("board", &["list", "show"]),
    ("lock", &["list", "show"]),
    ("log", &["show"]),
    ("types", &["list", "show"]),
    ("launcher", &["list", "show"]),
    ("worktree", &["list", "show"]),
    ("proc", &["list", "show"]),
    ("wake", &["list", "show", "status"]),
    ("pr", &["list", "show", "status"]),
    ("check", &["status"]),
    ("activity", &["list", "show"]),
];

/// Characters that chain, redirect or substitute: a command holding one is never read-only.
pub(crate) const SHELL: [char; 11] = [';', '&', '|', '<', '>', '`', '$', '(', ')', '\n', '\r'];

/// What the shadow does with one tool ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Deny,
}

/// Allow for the read tools, the read tools of the dashboard MCP, a `coord` read verb through
/// Bash and the shadow's own registration; deny everything else.
#[must_use]
pub fn verdict(tool: &str, input: &Value) -> Verdict {
    let reads = READ_TOOLS.contains(&tool)
        || tool
            .strip_prefix(DASHBOARD)
            .is_some_and(|name| DASHBOARD_READS.contains(&name))
        || (tool == "Bash" && input["command"].as_str().is_some_and(read_command));
    if reads { Verdict::Allow } else { Verdict::Deny }
}

fn read_command(command: &str) -> bool {
    if command.contains(SHELL) {
        return false;
    }
    let words: Vec<&str> = command.split_whitespace().collect();
    coord_args(&words).is_some_and(coord_words_allowed)
}

/// Whether the words of a `coord` call (the verb and its positional words, no options) are
/// something the shadow may run: a read verb, its own registration or a thread note.
#[must_use]
pub fn coord_words_allowed(rest: &[&str]) -> bool {
    is_read(rest) || is_registration(rest) || matches!(rest, ["thread", "note", ..])
}

/// The words after `coord`, or after `node <path>/coord.mjs`.
fn coord_args<'a>(words: &'a [&'a str]) -> Option<&'a [&'a str]> {
    match words {
        ["coord", rest @ ..] => Some(rest),
        ["node", script, rest @ ..] if script.ends_with("coord.mjs") => Some(rest),
        _ => None,
    }
}

fn is_read(rest: &[&str]) -> bool {
    match rest {
        ["inbox"] => true,
        [group, sub, ..] => READ_VERBS
            .iter()
            .any(|(name, subs)| name == group && subs.contains(sub)),
        _ => false,
    }
}

/// `coord thread register lead-hosted ...`, never another thread or the lead role.
fn is_registration(rest: &[&str]) -> bool {
    match rest {
        ["thread", "register", name, options @ ..] => {
            *name == THREAD
                && !options
                    .iter()
                    .any(|w| [LEAD_ROLE, LEGACY].iter().any(|word| w.contains(word)))
        }
        _ => false,
    }
}
