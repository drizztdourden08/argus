//! The review rows: the Compare rows (the same order: each folder, its chats, sub-threads under
//! their parent, then the chats in no folder), each with the times of every cell, the folder
//! row it is listed under and its difference.

use std::collections::BTreeMap;

use serde_json::Value;

use super::diff::{Home, Look, chat_diff, folder_diff};
use super::records::Records;
use super::{Cell, Counts, Review, Row, home_index};
use crate::sync::compare::{self, build_compare};
use crate::sync::drift::drift_of;
use crate::sync::job::{Planned, millis};
use crate::types::Account;

/// The accounts with a home scope in the scan, in column order.
pub(super) fn homes(planned: &Planned, accounts: &[Account]) -> Vec<Home> {
    accounts
        .iter()
        .filter_map(|a| {
            let index = home_index(&planned.scan, a)?;
            Some(Home {
                key: a.key.clone(),
                label: a.label.clone(),
                index,
                scope: planned.scan.scopes[index].scope.name(),
            })
        })
        .collect()
}

/// Builds the review from a scan and its automatic plan.
#[must_use]
pub fn build_review(planned: &Planned, accounts: &[Account]) -> Review {
    let compare = build_compare(planned, accounts);
    let look = Look {
        planned,
        records: Records::from_auto(&planned.scan, &planned.catalog),
        homes: homes(planned, accounts),
    };
    let mut folder: Option<String> = None;
    let rows: Vec<Row> = compare
        .rows
        .iter()
        .map(|row| {
            let is_folder = row.kind == "folder";
            if is_folder {
                folder = Some(row.id.clone());
            }
            Row {
                id: row.id.clone(),
                kind: row.kind,
                depth: row.depth,
                parent_id: row.parent_id.clone(),
                folder_id: if is_folder { None } else { folder.clone() },
                title: row.title.clone(),
                cells: cells(&look, row),
                diff: if is_folder {
                    folder_diff(&look, row)
                } else {
                    chat_diff(&look, row)
                },
            }
        })
        .collect();
    let chats = look
        .homes
        .iter()
        .map(|h| (h.key.clone(), planned.scan.scopes[h.index].chats.len()))
        .collect();
    Review {
        scanned_at: compare.scanned_at,
        accounts: compare.accounts,
        counts: Counts {
            chats,
            differences: rows.iter().filter(|r| r.diff.is_some()).count(),
            by_kind: drift_of(planned).by_kind,
        },
        rows,
    }
}

fn cells(look: &Look, row: &compare::Row) -> BTreeMap<String, Cell> {
    row.cells
        .iter()
        .map(|(key, c)| {
            let record = look
                .homes
                .iter()
                .find(|h| &h.key == key)
                .and_then(|h| look.planned.scan.scopes[h.index].chats.get(&row.id))
                .map(|chat| &chat.record);
            let time = |field: &str| record.and_then(|r: &Value| millis(r, field));
            let cell = Cell {
                present: c.present,
                folder: c.folder.clone(),
                archived: c.archived,
                last_activity_at: time("lastActivityAt"),
                last_focused_at: time("lastFocusedAt"),
                parent_id: c.parent_id.clone(),
                status: c.status,
            };
            (key.clone(), cell)
        })
        .collect()
}
