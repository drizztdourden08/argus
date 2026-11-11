//! How many chats and folders each account's home scope holds, for the account cards.

use super::compare::folders_of;
use super::job::Planned;
use crate::types::Account;

/// The chats and manual folders of one account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatCounts {
    /// Syncable chat records.
    pub total: usize,
    /// Of those, the archived ones.
    pub archived: usize,
    /// Manual folders.
    pub folders: usize,
}

/// The counts of each account whose home scope the scan covers, by account key.
#[must_use]
pub fn chat_counts(planned: &Planned, accounts: &[Account]) -> Vec<(String, ChatCounts)> {
    accounts
        .iter()
        .filter_map(|a| {
            let scan = planned.scan.scopes.iter().find(|s| {
                a.desktop_account.as_deref() == Some(s.scope.account.as_str())
                    && a.desktop_org.as_deref() == Some(s.scope.org.as_str())
            })?;
            let counts = ChatCounts {
                total: scan.chats.len(),
                archived: scan.chats.values().filter(|c| c.archived).count(),
                folders: folders_of(planned, &scan.scope.name()).names.len(),
            };
            Some((a.key.clone(), counts))
        })
        .collect()
}
