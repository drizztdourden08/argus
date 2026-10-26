//! The audit line every launcher and bundle write leaves on the board.

use super::record::{AuditLine, LauncherBoard, Who};

const AUDIT_MAX: usize = 120;

/// `text` cut to 120 characters with an ellipsis, like the CLI does.
pub fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let head: String = text.chars().take(max - 1).collect();
    format!("{head}\u{2026}")
}

pub fn audit(board: &mut LauncherBoard, who: &Who, text: &str, at: &str) {
    board.audit.push(AuditLine {
        at: at.to_owned(),
        thread: who.actor.clone(),
        text: clip(text, AUDIT_MAX),
    });
}
