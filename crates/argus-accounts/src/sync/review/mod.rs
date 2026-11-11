//! The sync review: every folder, chat and sub-thread of the accounts' home scopes side by side
//! ([`rows`]), each difference with the choices the owner has and the one the automatic sync
//! would make ([`diff`]); the plan of the owner's choices and edits ([`decide`], over the wanted
//! [`records`]); the archived chats old enough to clean up ([`cleanup`]); and the run that
//! writes it all ([`run`]).
//!
//! A choice equal to a row's default leaves the row to the automatic plan. Any other choice is
//! applied to every scope that holds the item, the old tool's cross scopes included, so the
//! next automatic plan finds the copies equal and keeps them. "skip" leaves the item as the
//! scan read it on every scope; while its copies still differ, the next automatic plan offers
//! the same change again. Edits are per account: they change the account's home scope and its
//! cross scopes only, so the next automatic plan may carry them to the other accounts (an
//! archive, a move) or bring back what they removed there (a folder deleted or renamed on one
//! account only comes back from the other; a delete is kept, as the tombstone blocks copies).

pub mod cleanup;
pub mod decide;
pub mod diff;
pub mod records;
pub mod rows;
pub mod run;

// test-tag: critical
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use serde::Serialize;

pub use cleanup::{Candidate, Cleanup, cleanup};
pub use decide::{Edit, Request, plan_review};
pub use rows::build_review;
pub use run::run_review;

use super::compare::{Column, Status};
use super::drift::ByKind;
use super::job::Scan;
use crate::types::Account;

/// One account's side of a row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    /// Whether the account has the folder or chat.
    pub present: bool,
    /// The folder the chat is in; for a folder row, its name.
    pub folder: Option<String>,
    /// Whether the chat is archived.
    pub archived: bool,
    /// The chat's `lastActivityAt`, epoch milliseconds.
    pub last_activity_at: Option<i64>,
    /// The chat's `lastFocusedAt`, epoch milliseconds.
    pub last_focused_at: Option<i64>,
    /// The chat this one was spawned from.
    pub parent_id: Option<String>,
    /// The relation to the other accounts.
    pub status: Status,
}

/// What a difference is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    /// An account lacks the chat.
    ChatMissing,
    /// The copies hold different conversations.
    History,
    /// Archived on one account, not on another.
    Archive,
    /// Filed in different folders.
    Membership,
    /// A sub-thread on one account, a top-level chat on another.
    Thread,
    /// An account lacks the folder.
    FolderMissing,
}

/// One answer to a difference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Choice {
    /// `skip`, `copy_to:<key>`, `newest`, `take:<key>`, `archive_all`, `unarchive_all`,
    /// `folder_of:<key>`, `link_all`, `unlink_all`, `create_on:<key>` or `delete_on:<key>`.
    pub id: String,
    /// The words on screen.
    pub label: String,
}

/// A difference of a row and its choices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diff {
    /// What differs. A chat row shows its first difference in the order chat missing, history,
    /// archive, thread, membership; the others follow the row's choice too ("skip" leaves the
    /// whole row alone).
    pub kind: DiffKind,
    /// The answers, the default among them, `skip` last.
    pub choices: Vec<Choice>,
    /// The choice the automatic sync makes.
    pub default: String,
}

/// One row: a folder or a chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    /// `folder:<name>` (`folder:` for the chats in no folder) or the chat's session id.
    pub id: String,
    /// `folder` or `chat`.
    pub kind: &'static str,
    /// 0 for a folder, 1 for a chat, 2 and more for a sub-thread.
    pub depth: u8,
    /// The parent chat of a sub-thread.
    pub parent_id: Option<String>,
    /// The folder row a chat is listed under; none for a folder row.
    pub folder_id: Option<String>,
    /// The folder name or the chat title.
    pub title: String,
    /// One cell per account key.
    pub cells: BTreeMap<String, Cell>,
    /// The difference, none when every account agrees.
    pub diff: Option<Diff>,
}

/// The totals above the rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    /// The chats of each account's home scope, by account key.
    pub chats: BTreeMap<String, usize>,
    /// Rows with a difference.
    pub differences: usize,
    /// What the automatic sync would change, by kind (as the Compare counts).
    pub by_kind: ByKind,
}

/// The answer of `GET /api/accounts/review`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    /// When the scan ran, epoch milliseconds.
    pub scanned_at: i64,
    /// The accounts, in column order.
    pub accounts: Vec<Column>,
    /// Every row, in order.
    pub rows: Vec<Row>,
    /// The totals.
    pub counts: Counts,
}

/// The index in the scan of the account's home scope: its Desktop account and org.
fn home_index(scan: &Scan, account: &Account) -> Option<usize> {
    scan.scopes.iter().position(|s| {
        account.desktop_account.as_deref() == Some(s.scope.account.as_str())
            && account.desktop_org.as_deref() == Some(s.scope.org.as_str())
    })
}

/// The key of the account a scope belongs to (its home or a cross scope), if any.
fn owner_of<'a>(accounts: &'a [Account], scan: &Scan, scope: usize) -> Option<&'a str> {
    let uuid = scan.scopes[scope].scope.account.as_str();
    accounts
        .iter()
        .find(|a| a.desktop_account.as_deref() == Some(uuid))
        .map(|a| a.key.as_str())
}
