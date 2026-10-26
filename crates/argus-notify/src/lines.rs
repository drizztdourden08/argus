//! The activity line each `ping` and `urgent` leaves.

use crate::kinds::{By, ToastState};
use crate::level::Level;
use crate::naming::label_of;
use crate::ports::{ActivityLevel, ActivityLine};
use crate::record::NewNotification;

/// One `note` line for a recorded notification; none for `info`.
#[must_use]
pub fn activity_line(record: &NewNotification) -> Option<ActivityLine> {
    if record.level == Level::Info {
        return None;
    }
    let level = if record.toast == ToastState::Failed {
        ActivityLevel::Warn
    } else {
        ActivityLevel::Info
    };
    Some(ActivityLine {
        level,
        text: format!(
            "{}: {}: {} ({})",
            who(record),
            record.title,
            record.text,
            outcome(record)
        ),
        project: record.project.clone(),
        thread: record.thread.clone(),
    })
}

fn who(record: &NewNotification) -> String {
    match record.by {
        By::Auto => "automatic ping".to_owned(),
        By::Page => "ping from the page".to_owned(),
        By::Thread | By::Overlord => {
            let urgent = if record.level == Level::Urgent {
                "urgent "
            } else {
                ""
            };
            let label = label_of(record.project.as_deref(), record.thread.as_deref());
            format!("{urgent}ping from {label}")
        }
    }
}

fn outcome(record: &NewNotification) -> String {
    let why = record.why.as_deref();
    match (record.toast, why) {
        (ToastState::Shown, None) => "toast shown".to_owned(),
        (ToastState::Shown, Some(why)) => format!("toast {why}"),
        (state, None) => format!("no toast: {}", state.as_str()),
        (state, Some(why)) => format!("no toast: {}, {why}", state.as_str()),
    }
}
