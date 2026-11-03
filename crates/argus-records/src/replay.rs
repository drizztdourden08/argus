//! The `replay { state, at, url }` block every record carries: what is left to do on GitHub
//! when the owner gives the go. Older writers called it `remote`; both are read.
// test-tag: critical

use serde_json::{Value, json};

use crate::fields::opt_text;

/// Where a record stands on the way to GitHub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replay {
    /// `pending`, `done` or `failed`.
    pub state: String,
    /// When it was replayed.
    pub at: Option<String>,
    /// Where it is on GitHub.
    pub url: Option<String>,
}

impl Replay {
    /// Whether the record still waits for the go.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.state == "pending"
    }
}

/// The replay block of a record, from `replay` or the older `remote`.
#[must_use]
pub fn replay_of(record: &Value) -> Option<Replay> {
    let block = [&record["replay"], &record["remote"]]
        .into_iter()
        .find(|b| b.is_object())?;
    Some(Replay {
        state: opt_text(block, "state").unwrap_or_default(),
        at: opt_text(block, "at"),
        url: opt_text(block, "url"),
    })
}

/// The block as the API shows it: an object, or `null` for a record that has none.
#[must_use]
pub fn replay_json(replay: Option<&Replay>) -> Value {
    replay.map_or(
        Value::Null,
        |r| json!({ "state": r.state, "at": r.at, "url": r.url }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_is_read_and_the_older_remote_name_is_accepted() {
        let new = json!({ "replay": { "state": "pending", "at": null, "url": null } });
        assert!(replay_of(&new).expect("block").is_pending());
        let old = json!({ "remote": { "state": "done", "at": "t", "url": "u" } });
        let r = replay_of(&old).expect("block");
        assert_eq!((r.state.as_str(), r.url.as_deref()), ("done", Some("u")));
        assert!(!r.is_pending());
        assert_eq!(replay_of(&json!({})), None);
        assert_eq!(replay_json(None), Value::Null);
    }
}
