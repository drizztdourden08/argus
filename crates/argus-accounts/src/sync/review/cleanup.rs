//! The cleanup candidates: archived chats of each account's home scope that saw no activity for
//! more than a number of days. The owner picks among them; they are deleted as review edits.

use serde::Serialize;

use super::home_index;
use crate::sidebar::chat_folder;
use crate::sync::job::{Scan, millis};
use crate::types::Account;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// One chat that may go.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    /// The account key.
    pub account: String,
    /// The chat.
    pub session_id: String,
    /// Its title.
    pub title: String,
    /// The folder it is in on that account.
    pub folder: Option<String>,
    /// Its `lastActivityAt`, epoch milliseconds.
    pub last_activity_at: i64,
}

/// The answer of `GET /api/accounts/cleanup`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cleanup {
    /// The age limit asked for.
    pub days: u32,
    /// The chats older than that, per account, oldest first.
    pub candidates: Vec<Candidate>,
}

/// The archived chats whose last activity is more than `days` before `now_ms`. A chat with no
/// `lastActivityAt` is never a candidate.
#[must_use]
pub fn cleanup(scan: &Scan, accounts: &[Account], now_ms: i64, days: u32) -> Cleanup {
    let before = now_ms - i64::from(days) * DAY_MS;
    let mut candidates = Vec::new();
    for account in accounts {
        let Some(index) = home_index(scan, account) else {
            continue;
        };
        let home = &scan.scopes[index];
        let mut found: Vec<Candidate> = home
            .chats
            .iter()
            .filter(|(_, c)| c.archived)
            .filter_map(|(id, c)| {
                let at = millis(&c.record, "lastActivityAt").filter(|&t| t < before)?;
                Some(Candidate {
                    account: account.key.clone(),
                    session_id: id.clone(),
                    title: c.title.clone(),
                    folder: chat_folder(&scan.keys, &home.scope.name(), id),
                    last_activity_at: at,
                })
            })
            .collect();
        found.sort_by_key(|c| (c.last_activity_at, c.session_id.clone()));
        candidates.extend(found);
    }
    Cleanup { days, candidates }
}
