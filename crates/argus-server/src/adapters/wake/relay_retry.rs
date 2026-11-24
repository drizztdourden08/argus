//! What the relay undoes when a headless resume did not end well: the resumed CLI already set the thread
//! working and marked its inbox delivered, and it died without a Stop hook. The thread goes back to idle, the
//! messages the run took are undelivered, and a fresh wake makes the lead due again.

use argus_board::Board;
use argus_board::store::{touch, with_board};
use argus_wake::relay::Ward;
use serde_json::{Value, json};

use super::backoff::clock_text;
use super::relay::RelayHost;
use crate::WakeRequest;

/// The reason of the wake that makes a lead due again.
const RETRY_REASON: &str = "wake-relay";

/// Sets the thread idle and takes the delivered mark off the messages delivered at or after `since`.
/// Returns whether anything changed.
fn put_back(board: &mut Board, thread: &str, since: &str) -> bool {
    let mut changed = false;
    if let Some(entry) = board["threads"].get_mut(thread) {
        if entry["activity"].as_str() != Some("idle") {
            entry["activity"] = json!("idle");
            changed = true;
        }
    }
    if changed {
        touch(board, "threads", thread);
    }
    let Some(list) = board["inbox"].get_mut(thread).and_then(Value::as_array_mut) else {
        return changed;
    };
    for message in list.iter_mut() {
        let at = message["deliveredAt"].as_str().unwrap_or("");
        if !at.is_empty() && at >= since {
            message["deliveredAt"] = Value::Null;
            changed = true;
        }
    }
    changed
}

impl RelayHost {
    /// Puts the lead back as it was before the failed resume that started at `since` (an ISO time).
    pub(super) fn put_back(&self, target: &Ward, since: &str) {
        let key = &target.key;
        let done = with_board(&self.coord_home, &key.project, false, |board| {
            Ok(put_back(board, &key.thread, since))
        });
        if matches!(done, Ok(true)) {
            self.requeue(target);
        }
    }

    fn requeue(&self, target: &Ward) {
        let wake = WakeRequest::about(
            &target.key.project,
            &target.key.thread,
            RETRY_REASON,
            "retry",
            format!(
                "Argus: a wake of {} could not be delivered, run coord inbox",
                target.key.thread
            ),
        );
        drop(self.hook.submit(&[wake]));
    }

    /// Tells the owner that the account of `target` is at its limit and when the relay tries again.
    pub(super) fn tell_limited(&self, target: &Ward, retry_at: i64) {
        let text = format!(
            "{} lead could not be woken: account limit reached, retrying at {}",
            target.key.project,
            clock_text(retry_at)
        );
        self.ping(target, format!("{} is asleep", target.key.thread), text);
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Board {
        json!({
            "threads": {"lead": {"activity": "working"}},
            "inbox": {"lead": [
                {"id": "m1", "deliveredAt": "2026-11-23T00:00:00.000Z"},
                {"id": "m2", "deliveredAt": "2026-11-23T00:05:10.000Z"},
                {"id": "m3", "deliveredAt": null},
            ]},
        })
    }

    #[test]
    fn a_failed_run_leaves_the_lead_idle_and_its_new_messages_undelivered() {
        let mut board = board();
        assert!(put_back(&mut board, "lead", "2026-11-23T00:05:00.000Z"));
        assert_eq!(board["threads"]["lead"]["activity"], "idle");
        let inbox = &board["inbox"]["lead"];
        assert_eq!(inbox[0]["deliveredAt"], "2026-11-23T00:00:00.000Z");
        assert_eq!(inbox[1]["deliveredAt"], Value::Null);
        assert_eq!(inbox[2]["deliveredAt"], Value::Null);
    }

    #[test]
    fn nothing_to_undo_changes_nothing() {
        let mut board = json!({"threads": {"lead": {"activity": "idle"}}, "inbox": {}});
        assert!(!put_back(&mut board, "lead", "2026-11-23T00:05:00.000Z"));
        assert_eq!(board["dirty"], Value::Null);
    }
}
