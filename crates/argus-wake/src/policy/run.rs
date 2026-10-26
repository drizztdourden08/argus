//! A run that failed wakes the lead of its project: an error is never silent.

use crate::text::cut;
use crate::wake::WakeInput;

const LAST_LINE_CLIP: usize = 120;

/// The part of a finished run the wake reads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunView {
    /// `R-3`.
    pub id: String,
    /// Project key.
    pub scope: String,
    /// The launcher's name.
    pub name: String,
    /// `failed`, `exited`, `stopped`...
    pub status: String,
    /// The exit code, when it exited.
    pub exit_code: Option<i64>,
    /// Why it failed, when it never ran or timed out.
    pub error: Option<String>,
    /// The last line of its log.
    pub last_line: Option<String>,
    /// A restart orphaned it.
    pub orphan: bool,
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            skip_sequence(&mut chars);
        } else {
            out.push(c);
        }
    }
    out
}

fn skip_sequence(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while chars.next_if(|c| ('0'..='?').contains(c)).is_some() {}
    while chars.next_if(|c| (' '..='/').contains(c)).is_some() {}
    chars.next_if(|c| ('@'..='~').contains(c));
}

/// `R-3 play failed (exit 1): <its last line>`, or `None` for a run that is not a failure to act on.
#[must_use]
pub fn failed_run_line(run: &RunView) -> Option<String> {
    if run.status != "failed" || run.orphan {
        return None;
    }
    let error = run.error.as_deref().filter(|e| !e.is_empty());
    let how = match (error, run.exit_code) {
        (Some(e), _) if e != "timeout" => e.to_owned(),
        (_, Some(code)) => format!("exit {code}"),
        (e, None) => e.unwrap_or("failed").to_owned(),
    };
    let last = cut(
        &strip_ansi(run.last_line.as_deref().unwrap_or("")),
        LAST_LINE_CLIP,
    );
    let tail = if last.is_empty() {
        String::new()
    } else {
        format!(": {last}")
    };
    Some(format!("{} {} failed ({how}){tail}", run.id, run.name))
}

/// The wake for a failed run, or `None` when it is not a failure or the project has no lead.
#[must_use]
pub fn wake_for_run(run: &RunView, lead: Option<&str>) -> Option<WakeInput> {
    let line = failed_run_line(run)?;
    let lead = lead?;
    let text = format!("{line}. Open its log, fix the cause and tell the owner.");
    Some(WakeInput::about(
        &run.scope,
        lead,
        "run-failed",
        &run.id,
        text,
    ))
}
