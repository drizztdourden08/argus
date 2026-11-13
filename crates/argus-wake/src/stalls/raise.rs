//! What the Overlord hears of the open stalls: a wake the first time, again every `resend_ms` until the stall
//! is acknowledged or clears, and a line in the activity log when one is raised or clears. The wakes table
//! is the memory, so a restart keeps the pace.

use argus_core::clock::age_text;

use super::StallTick;
use super::book::{Open, StallBook};
use super::config::StallConfig;
use super::env::StallEnv;
use crate::activity::{ActivityLine, Level};
use crate::wake::WakeInput;

/// The reason of the wake the Overlord hears.
pub(super) const REASON: &str = "stall";

/// The wakes and activity lines the stalls open at `now` call for.
pub(super) fn due(
    now: i64,
    env: &StallEnv<'_>,
    config: &StallConfig,
    book: &StallBook,
    out: &mut StallTick,
) {
    let overlord = env.boards.overlord();
    for open in book.unacked() {
        let last = env.last_sent(&overlord, env.main, REASON, &open.key);
        if last != 0 && now - last < config.resend_ms {
            continue;
        }
        out.wakes.push(WakeInput::about(
            &overlord,
            env.main,
            REASON,
            &open.key,
            open.found.text.clone(),
        ));
        if last == 0 {
            out.activity
                .push(line("stall", Level::Warn, open.found.text.clone(), open));
        }
    }
}

/// The activity lines of the stalls that cleared.
pub(super) fn cleared(now: i64, gone: &[Open], out: &mut StallTick) {
    for open in gone {
        let words = open
            .found
            .text
            .strip_prefix("stall: ")
            .unwrap_or(&open.found.text);
        let text = format!(
            "stall cleared: {words} (open {})",
            age_text(now - open.found.since)
        );
        out.activity
            .push(line("stall-cleared", Level::Info, text, open));
    }
}

fn line(kind: &str, level: Level, text: String, open: &Open) -> ActivityLine {
    let lead = &open.found.lead;
    ActivityLine {
        kind: kind.to_owned(),
        level,
        text,
        project: Some(open.found.project.clone()),
        thread: (!lead.is_empty()).then(|| lead.clone()),
    }
}
