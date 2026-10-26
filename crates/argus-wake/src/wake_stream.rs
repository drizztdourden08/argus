//! The pure parts of `dashboard wake-stream --project <key> [--thread <id|*>] [--id <label>]`: the
//! arguments, the ONE line printed per wake (hooks and Claude sessions parse it), and the reconnect pauses.
//!
//! The line is `WAKE <thread>: <line1>; <line2> ... Run: coord inbox`. Nothing else goes to stdout.

use argus_core::ids::is_key;
use serde_json::Value;

use crate::external::{clean_label, valid_thread_key};
use crate::text::one_line;

/// The first pause before a reconnect.
pub const BACKOFF_START_MS: u64 = 1000;
/// The longest pause before a reconnect.
pub const BACKOFF_MAX_MS: u64 = 15_000;
/// A connection silent for this long is dead (the server pings every 15 s).
pub const SILENT_MS: u64 = 60_000;

/// The arguments of the command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeArgs {
    /// The project key.
    pub project: String,
    /// A thread id, or `*` for every thread of the project.
    pub thread: String,
    /// A label the server lists.
    pub id: Option<String>,
}

#[derive(Default)]
struct Parsed {
    project: Option<String>,
    thread: Option<String>,
    id: Option<String>,
}

fn flag(arg: &str) -> Option<(&str, Option<&str>)> {
    let body = arg.strip_prefix("--")?;
    Some(match body.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (body, None),
    })
}

fn store(parsed: &mut Parsed, name: &str, value: String) {
    match name {
        "project" => parsed.project = Some(value),
        "thread" => parsed.thread = Some(value),
        _ => parsed.id = Some(value),
    }
}

/// Parse the command line after `dashboard` (it may start with `wake-stream`).
///
/// # Errors
/// The message to print on stderr when an argument is missing or wrong.
pub fn parse_wake_args(argv: &[String]) -> Result<WakeArgs, String> {
    let mut parsed = Parsed::default();
    let mut rest = argv.iter();
    while let Some(arg) = rest.next() {
        match flag(arg) {
            Some((name @ ("project" | "thread" | "id"), inline)) => {
                let value = inline.map(str::to_owned).or_else(|| rest.next().cloned());
                match value {
                    Some(v) if !v.starts_with("--") => store(&mut parsed, name, v),
                    _ => return Err(format!("--{name} needs a value")),
                }
            }
            Some(_) => {}
            None if arg == "wake-stream" => {}
            None => return Err(format!("unexpected argument \"{arg}\"")),
        }
    }
    finish(parsed)
}

fn finish(parsed: Parsed) -> Result<WakeArgs, String> {
    let project = parsed.project.filter(|p| is_key(p));
    let Some(project) = project else {
        return Err("--project <key> is required (the project key, e.g. rotp)".to_owned());
    };
    let thread = parsed.thread.unwrap_or_else(|| "*".to_owned());
    if !valid_thread_key(&thread) {
        return Err("--thread must be a thread id or *".to_owned());
    }
    Ok(WakeArgs {
        project,
        thread,
        id: parsed.id.as_deref().and_then(clean_label),
    })
}

/// One wake event as the line a Monitor shows.
#[must_use]
pub fn line_of(thread: &str, lines: &[String]) -> String {
    let joined: Vec<String> = lines
        .iter()
        .map(|l| one_line(l))
        .filter(|l| !l.is_empty())
        .collect();
    format!("WAKE {thread}: {} Run: coord inbox", joined.join("; "))
}

/// What a frame of the stream asks the command to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamAction {
    /// Print this line on stdout.
    Print(String),
    /// A newer wake-stream for the same thread took over: exit 0 and never reconnect.
    Replaced,
}

/// What to do with one SSE frame (`event` name and its JSON data).
#[must_use]
pub fn action_of(name: &str, data: &Value) -> Option<StreamAction> {
    match name {
        "wake" => {
            let thread = data.get("thread").and_then(Value::as_str).unwrap_or("");
            let lines: Vec<String> = data
                .get("lines")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|l| l.as_str().map_or_else(|| l.to_string(), str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            Some(StreamAction::Print(line_of(thread, &lines)))
        }
        "replaced" => Some(StreamAction::Replaced),
        _ => None,
    }
}

/// The pause after a failed connection: doubles up to the maximum.
#[must_use]
pub fn next_backoff(current_ms: u64) -> u64 {
    current_ms.saturating_mul(2).min(BACKOFF_MAX_MS)
}
