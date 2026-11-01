//! What `coord ping` says to the thread that pinged, in one sentence.
// test-tag: critical

use crate::kinds::ToastState;
use crate::level::Level;
use crate::request::Outcome;

const URGENT_WARNING: &str =
    " Urgent stays on his screen until he dismisses it: use it only when he must act now.";

/// The sentence for the pinging thread, from the outcome of its ping.
#[must_use]
pub fn sentence(outcome: &Outcome, title: &str, level: Level) -> String {
    let urgent = if level == Level::Urgent {
        URGENT_WARNING
    } else {
        ""
    };
    let why = outcome.why.as_deref();
    let body = match (outcome.toast, why) {
        (ToastState::Shown, None) => format!("pinged the owner: {title} (toast shown)."),
        (ToastState::Shown, Some(why)) => {
            format!("pinged the owner: {title} (toast shown: {why}).")
        }
        (ToastState::Failed, why) => format!(
            "recorded, but the toast could not be shown: {}. The owner sees it in Argus > Notifications.",
            why.unwrap_or("unknown reason")
        ),
        (ToastState::Limited | ToastState::Skipped, why) => {
            format!("recorded, no toast: {}.", why.unwrap_or("skipped"))
        }
    };
    format!("{body}{urgent}")
}

#[cfg(test)]
mod tests {
    use super::sentence;
    use crate::kinds::ToastState;
    use crate::level::Level;
    use crate::request::Outcome;

    fn outcome(toast: ToastState, why: Option<&str>) -> Outcome {
        Outcome {
            notification: None,
            toast,
            why: why.map(str::to_owned),
        }
    }

    #[test]
    fn each_toast_state_has_its_sentence() {
        let say = |toast, why| sentence(&outcome(toast, why), "T", Level::Ping);
        assert_eq!(
            say(ToastState::Shown, None),
            "pinged the owner: T (toast shown)."
        );
        assert_eq!(
            say(ToastState::Limited, Some("you pinged 4 times in a minute")),
            "recorded, no toast: you pinged 4 times in a minute."
        );
        assert_eq!(
            say(ToastState::Skipped, Some("info is recorded only")),
            "recorded, no toast: info is recorded only."
        );
        assert_eq!(
            say(ToastState::Failed, Some("PowerShell exit 2")),
            "recorded, but the toast could not be shown: PowerShell exit 2. The owner sees it in Argus > Notifications."
        );
    }

    #[test]
    fn an_urgent_ping_carries_its_warning() {
        let said = sentence(&outcome(ToastState::Shown, None), "T", Level::Urgent);
        assert!(said.ends_with("use it only when he must act now."));
    }
}
