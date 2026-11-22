//! The three rules a first start makes, all the owner's. The prompts follow `portfolio.md`.

use super::{By, Rule, RuleInput, create};

const BOARD_SWEEP: &str = "Sweep every board with portfolio_overview and list_dependencies \
--state open. For each lead that is idle with open items and no blocker, each stalled dependency \
or unread message older than 10 min, message that lead directly with the item and what to do. Ack \
stalls with coord stall ack. One line per finding.";

const DEFENDER_WATCH: &str = "Run the read-only Defender check from portfolio.md. Say nothing \
when it is fine. Ping the owner when real-time scanning is on and the engine is busy.";

const DAILY_SYNC: &str = "Run the daily remote sync window as portfolio.md describes, lead by \
lead, then close it.";

fn rule(n: u32, title: &str, prompt: &str, every_min: i64, now_ms: i64) -> Option<Rule> {
    let input = RuleInput {
        title: title.to_owned(),
        prompt: prompt.to_owned(),
        every_min,
        enabled: true,
    };
    create(&format!("OR-{n}"), &input, By::Owner, now_ms).ok()
}

/// The seed rules `OR-1` to `OR-3`, each due one period after `now_ms`.
#[must_use]
pub fn seed(now_ms: i64) -> Vec<Rule> {
    [
        rule(1, "Board sweep", BOARD_SWEEP, 15, now_ms),
        rule(2, "Defender watch", DEFENDER_WATCH, 60, now_ms),
        rule(3, "Daily sync window", DAILY_SYNC, 1440, now_ms),
    ]
    .into_iter()
    .flatten()
    .collect()
}
