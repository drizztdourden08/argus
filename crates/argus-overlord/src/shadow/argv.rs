//! The shadow child's command line: the live one, forked from the desktop session, with the
//! write tools refused outright so no allow rule of the project can let them through.

use crate::session::args::{SessionArgs, build_argv};

/// Tools the shadow never has, whatever the settings allow: the file writers, the shell that is
/// not Bash, and the dashboard MCP tools that write.
const DISALLOWED: [&str; 14] = [
    "Edit",
    "Write",
    "NotebookEdit",
    "PowerShell",
    "mcp__coord-dashboard__post_entry",
    "mcp__coord-dashboard__answer",
    "mcp__coord-dashboard__decision",
    "mcp__coord-dashboard__record_test_result",
    "mcp__coord-dashboard__send_to_lead",
    "mcp__coord-dashboard__attach_proof",
    "mcp__coord-dashboard__notify_owner",
    "mcp__coord-dashboard__cancel_dependency",
    "mcp__coord-dashboard__run_start",
    "mcp__coord-dashboard__run_stop",
];

/// The first start forks the desktop session (its context, a new id, the desktop file untouched).
/// A restart resumes the shadow's own session, without forking again.
#[must_use]
pub fn shadow_args(base: &SessionArgs, desktop: &str) -> SessionArgs {
    let first = base.resume.is_none();
    SessionArgs {
        resume: Some(base.resume.clone().unwrap_or_else(|| desktop.to_owned())),
        fork: first,
        ..base.clone()
    }
}

/// The argument line of the shadow child.
#[must_use]
pub fn shadow_argv(base: &SessionArgs, desktop: &str) -> Vec<String> {
    let mut argv = build_argv(&shadow_args(base, desktop));
    argv.extend(["--disallowedTools".to_owned(), DISALLOWED.join(",")]);
    argv
}
