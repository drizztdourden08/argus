//! The words of the item wakes and of the owner notices.

use argus_board::item_events::{EventKind, ItemEvent};

use crate::text::clip;

const TITLE_MAX: usize = 60;
const ASK_MAX: usize = 100;

/// Who made the change, as a person reads it.
fn by_name(by: &str) -> &str {
    match by {
        "human" | "owner" => "the owner",
        "" => "someone",
        other => other,
    }
}

/// `I-7 (Login screen)`.
fn subject(event: &ItemEvent) -> String {
    format!("{} ({})", event.id, clip(&event.title, TITLE_MAX))
}

/// The line a thread hears.
pub(super) fn wake_line(event: &ItemEvent) -> String {
    let by = by_name(&event.by);
    let what = subject(event);
    match event.kind {
        EventKind::Assigned => format!("{what} was assigned to you by {by}"),
        EventKind::Unassigned => format!("{what} was taken from you by {by}"),
        EventKind::TodoAssigned => format!("to-do {what} was assigned to you by {by}"),
        EventKind::StageChanged => format!("{what} moved {} (by {by})", event.detail),
        EventKind::Ask => format!(
            "{by} asked on {} ({}): {}",
            event.item,
            event.id,
            clip(&event.detail, ASK_MAX)
        ),
        EventKind::Unblocked => format!("{what} is unblocked: {} settled", event.detail),
        EventKind::ChildrenDone => format!(
            "every child of {what} is done or dropped ({} in all)",
            event.detail
        ),
        EventKind::PlanRunDone => format!("{what}: {}", event.detail),
        EventKind::DepLanded | EventKind::Verify | EventKind::PlanReady => what,
    }
}

/// The title and the text of the toast for an owner notice.
pub(super) fn notice_words(event: &ItemEvent, project: &str) -> (String, String) {
    let what = subject(event);
    let by = by_name(&event.by);
    match event.kind {
        EventKind::Verify => (
            "Ready to verify".to_owned(),
            format!("{project}: {what} is ready to verify"),
        ),
        EventKind::PlanReady => (
            "Ready to run".to_owned(),
            format!(
                "{project}: test plan {what} is ready to run on {}",
                event.detail
            ),
        ),
        EventKind::TodoAssigned => (
            "To-do for you".to_owned(),
            format!("{project}: to-do {what} was given to you by {by}"),
        ),
        _ => (
            "Assigned to you".to_owned(),
            format!("{project}: {what} was assigned to you by {by}"),
        ),
    }
}
