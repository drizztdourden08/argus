//! The texts of the switch-over and the rollback (I-173): the wake that asks the desktop Overlord
//! for its handover, the first message of the live child, and the handover file the hosted side
//! leaves for the desktop Overlord. All pure.

use std::fmt::Write;

/// The reason of the wake that asks the desktop Overlord for its handover.
pub const SWITCH_REASON: &str = "overlord-switch";

/// The subject of that wake.
pub const SWITCH_KIND: &str = "switch";

/// The wake text for the desktop Overlord at the switch-over. `handover` is the file to write.
#[must_use]
pub fn switch_text(handover: &str) -> String {
    format!(
        "Argus is switching to the hosted Overlord. Write your handover to {handover} now: what is \
         open, the agents running and their stop points, and the next steps. Then end your turn. \
         Your coordinator thread is ended; the hosted Overlord takes it."
    )
}

const REGISTER: &str = "LIVE MODE (Argus). You are the Overlord now, hosted by Argus. First run \
     `coord thread register coordinator --role coordinator`.";

/// The first message of the live child. At a switch-over `handover` is the file the desktop
/// Overlord writes; any other start (a boot, the owner's start) has none.
#[must_use]
pub fn live_intro(handover: Option<&str>) -> String {
    let Some(file) = handover else {
        return REGISTER.to_owned();
    };
    format!(
        "{REGISTER} Then read {file} if it is there: the desktop Overlord writes it as you start, \
         so if it is missing, look again at your next wake. A wake with the reason \
         {SWITCH_REASON} was meant for the desktop Overlord: ignore it."
    )
}

/// What the hosted side hands back at a rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handback {
    /// When it was written.
    pub at: String,
    /// The hosted child's session id, if it had one.
    pub session_id: Option<String>,
    /// The note of the hosted coordinator thread.
    pub note: Option<String>,
    /// The open items of the Overlord's board, as id and title.
    pub items: Vec<(String, String)>,
}

fn or_none(text: Option<&str>) -> &str {
    text.filter(|t| !t.trim().is_empty()).unwrap_or("none")
}

fn items_text(items: &[(String, String)]) -> String {
    if items.is_empty() {
        return "- none\n".to_owned();
    }
    let mut text = String::new();
    for (id, title) in items {
        let _ = writeln!(text, "- {id}: {title}");
    }
    text
}

/// The handover file the hosted side writes at a rollback, for the desktop Overlord.
#[must_use]
pub fn handback_text(back: &Handback) -> String {
    format!(
        "# Handover from the hosted Overlord\n\n\
         Written by Argus at {at}, when the owner rolled back to the desktop Overlord.\n\n\
         - Hosted session: {session}\n\
         - Thread note: {note}\n\n\
         ## To take the thread back\n\n\
         Run `coord thread register coordinator --role coordinator`, then `coord inbox`.\n\n\
         ## Open items on the board\n\n\
         {items}",
        at = back.at,
        session = or_none(back.session_id.as_deref()),
        note = or_none(back.note.as_deref()),
        items = items_text(&back.items),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "C:/Users/o/.coord/portfolio/handover.md";

    fn back() -> Handback {
        Handback {
            at: "2026-11-14T10:00:00Z".to_owned(),
            session_id: Some("s-9".to_owned()),
            note: Some("watching Brock".to_owned()),
            items: vec![("I-3".to_owned(), "Ship the tray".to_owned())],
        }
    }

    // test-tag: critical
    #[test]
    fn the_handback_names_the_session_the_note_and_each_open_item() {
        let text = handback_text(&back());
        assert!(text.contains("Hosted session: s-9"), "{text}");
        assert!(text.contains("Thread note: watching Brock"), "{text}");
        assert!(text.contains("- I-3: Ship the tray"), "{text}");
        assert!(text.contains("thread register coordinator"), "{text}");
    }

    #[test]
    fn an_empty_handback_says_none() {
        let empty = Handback {
            at: "t".to_owned(),
            session_id: None,
            note: Some("  ".to_owned()),
            items: Vec::new(),
        };
        let text = handback_text(&empty);
        assert!(text.contains("Hosted session: none"), "{text}");
        assert!(text.contains("Thread note: none"), "{text}");
        assert!(text.ends_with("- none\n"), "{text}");
    }

    #[test]
    fn the_wake_and_the_intro_name_the_handover_file() {
        assert!(switch_text(FILE).contains(FILE));
        let intro = live_intro(Some(FILE));
        assert!(intro.contains(FILE), "{intro}");
        assert!(intro.contains(SWITCH_REASON), "{intro}");
    }

    #[test]
    fn a_start_without_a_switch_never_registers() {
        let intro = live_intro(None);
        assert!(intro.contains("do not run"), "{intro}");
        assert!(!intro.contains("handover"), "{intro}");
    }
}
