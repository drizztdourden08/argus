//! Whether a sub-agent's stop only waits on its own background command. Such an agent ends its
//! turn to wait, which fires `SubagentStop`, but it is not finished: its real end is the
//! `SubagentHandback` call that delivers the report.

use std::path::Path;

use argus_agents::fs::{FileSystem, StdFs};
use serde_json::Value;

/// How much of the end of the transcript is looked at.
const TAIL_BYTES: u64 = 512 * 1024;
const FLAG: &str = "\"run_in_background\"";
const HANDBACK: &str = "\"name\":\"SubagentHandback\"";

/// Whether `text` holds `"run_in_background"`, a colon and `true`, blanks allowed between.
fn background_started(text: &str) -> bool {
    text.match_indices(FLAG).any(|(at, flag)| {
        let rest = text[at + flag.len()..].trim_start();
        rest.strip_prefix(':')
            .is_some_and(|after| after.trim_start().starts_with("true"))
    })
}

/// The last 512 KB of the transcript, `None` when it cannot be read.
fn tail_of(file: &Path) -> Option<String> {
    let size = StdFs.stat(file)?.size;
    let len = size.min(TAIL_BYTES);
    let bytes = StdFs
        .read_at(file, size - len, usize::try_from(len).ok()?)
        .ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// True when the tail of the transcript at `file` shows a background command started and no
/// handback; a missing or unreadable transcript, or one without a background command, is
/// never waiting.
pub fn on_background(file: &Value) -> bool {
    let Some(file) = file.as_str().filter(|f| !f.is_empty()) else {
        return false;
    };
    tail_of(Path::new(file))
        .is_some_and(|tail| background_started(&tail) && !tail.contains(HANDBACK))
}
