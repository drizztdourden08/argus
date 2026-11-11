//! The rows of the Compare view: one ordered union of the folders and chats of every account.
//! Each folder (in the order the accounts list them) is followed by its chats, a sub-thread
//! right under its parent, then the chats that are in no folder. A cell says what one account
//! holds for the row, and whether it agrees with the others.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;
use serde_json::Value;

use super::drift::drift_of;
use super::job::{Chat, Planned};
use crate::catalog::plan::ChangeKind;
use crate::sidebar::FolderStep;
use crate::types::{Account, Scope};

/// How one account's cell relates to the other accounts'.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Every account agrees.
    Same,
    /// Absent, and a sync will not create it.
    Missing,
    /// Absent, and a sync will create it.
    WillCreate,
    /// Only this account has it.
    OnlyHere,
    /// The folder differs from another account's.
    FolderDiffers,
    /// Archived here, active elsewhere.
    ArchivedHereOnly,
    /// Active here, archived elsewhere.
    ActiveHereOnly,
    /// A top-level chat here, a sub-thread elsewhere.
    NotSubthread,
    /// A sub-thread here, a top-level chat elsewhere.
    Subthread,
}

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
    /// The chat this one was spawned from.
    pub parent_id: Option<String>,
    /// The relation to the other accounts.
    pub status: Status,
}

/// A column of the view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Column {
    /// The account key.
    pub key: String,
    /// The account's name on screen.
    pub label: String,
    /// The account's color.
    pub color: String,
    /// The scope the column reads, `<account>/<org>`.
    pub scope: String,
}

/// One row: a folder or a chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    /// `folder:<name>` (`folder:` for the chats in no folder) or the chat's session id.
    pub id: String,
    /// `folder` or `chat`.
    pub kind: &'static str,
    /// 0 for a folder, 1 for a chat, 2 for a sub-thread.
    pub depth: u8,
    /// The parent chat of a sub-thread.
    pub parent_id: Option<String>,
    /// The folder name or the chat title.
    pub title: String,
    /// One cell per account key.
    pub cells: BTreeMap<String, Cell>,
}

/// The totals above the rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    /// Rows where every cell is the same.
    pub same: usize,
    /// Rows with a difference.
    pub different: usize,
    /// What a sync would change, by kind.
    pub by_kind: super::drift::ByKind,
}

/// The answer of `GET /api/accounts/compare`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Compare {
    /// When the scan ran, epoch milliseconds.
    pub scanned_at: i64,
    /// The accounts, in column order.
    pub accounts: Vec<Column>,
    /// The ordered rows.
    pub rows: Vec<Row>,
    /// The totals.
    pub counts: Counts,
}

/// The sidebar folders of one scope.
pub(super) struct Folders {
    /// Names in the order the app lists them.
    pub(super) names: Vec<String>,
    /// Chat id to its folder name.
    of_chat: HashMap<String, String>,
    /// Folder name to the chat ids in its order list.
    order: HashMap<String, Vec<String>>,
}

pub(super) fn folders_of(planned: &Planned, scope: &str) -> Folders {
    let entry = planned
        .scan
        .keys
        .group_scopes
        .get("value")
        .and_then(|v| v.get(scope));
    let text = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
    let groups: Vec<(String, String)> = entry
        .and_then(|e| e.get("groups"))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|g| Some((text(g, "id")?, text(g, "name")?)))
                .collect()
        })
        .unwrap_or_default();
    let name_of = |id: &str| groups.iter().find(|(i, _)| i == id).map(|(_, n)| n.clone());
    let mut of_chat = HashMap::new();
    if let Some(map) = entry
        .and_then(|e| e.get("assignments"))
        .and_then(Value::as_object)
    {
        for (key, folder) in map {
            let name = folder.as_str().and_then(name_of);
            if let (Some(chat), Some(name)) = (key.strip_prefix("code:"), name) {
                of_chat.insert(chat.to_owned(), name);
            }
        }
    }
    let mut order = HashMap::new();
    for (id, name) in &groups {
        let list = entry
            .and_then(|e| e.get("order"))
            .and_then(|o| o.get(id))
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str()?.strip_prefix("code:").map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        order.insert(name.clone(), list);
    }
    Folders {
        names: groups.into_iter().map(|(_, n)| n).collect(),
        of_chat,
        order,
    }
}

/// One account's data for the view.
struct Col<'a> {
    column: Column,
    scope: Scope,
    chats: &'a BTreeMap<String, Chat>,
    folders: Folders,
}

fn columns<'a>(planned: &'a Planned, accounts: &[Account]) -> Vec<Col<'a>> {
    accounts
        .iter()
        .filter_map(|a| {
            let scan = planned.scan.scopes.iter().find(|s| {
                a.desktop_account.as_deref() == Some(s.scope.account.as_str())
                    && a.desktop_org.as_deref() == Some(s.scope.org.as_str())
            })?;
            let scope = scan.scope.clone();
            Some(Col {
                column: Column {
                    key: a.key.clone(),
                    label: a.label.clone(),
                    color: a.color.clone(),
                    scope: scope.name(),
                },
                folders: folders_of(planned, &scope.name()),
                chats: &scan.chats,
                scope,
            })
        })
        .collect()
}

/// A chat's row facts: its leading column's view of it.
struct Place {
    title: String,
    folder: Option<String>,
    parent: Option<String>,
    focused: i64,
    rank: usize,
}

/// The column that focused the chat last (one where it has a folder wins a tie) leads.
fn place_of(cols: &[Col], id: &str) -> Option<Place> {
    let (col, chat) = cols
        .iter()
        .filter_map(|c| Some((c, c.chats.get(id)?)))
        .max_by_key(|(c, chat)| (chat.focused, c.folders.of_chat.contains_key(id)))?;
    let folder = col.folders.of_chat.get(id).cloned();
    let rank = folder
        .as_ref()
        .and_then(|f| col.folders.order.get(f))
        .and_then(|list| list.iter().position(|c| c == id))
        .unwrap_or(usize::MAX);
    Some(Place {
        title: chat.title.clone(),
        folder,
        parent: chat.parent.clone(),
        focused: chat.focused,
        rank,
    })
}

/// A parent that exists in the union and does not lead back to the chat itself.
fn usable_parent(places: &BTreeMap<String, Place>, id: &str) -> Option<String> {
    let parent = places.get(id)?.parent.clone()?;
    let mut at = parent.clone();
    for _ in 0..16 {
        let up = places.get(&at)?;
        if at == id {
            return None;
        }
        match &up.parent {
            Some(next) => at = next.clone(),
            None => return Some(parent),
        }
    }
    None
}

fn chat_cell(cols: &[Col], col: &Col, planned: &Planned, id: &str) -> Cell {
    let mine = col.chats.get(id);
    let folder = col.folders.of_chat.get(id).cloned();
    let Some(chat) = mine else {
        let copied = planned.catalog.changes.iter().any(|c| {
            c.kind == ChangeKind::CopySession && c.scope == col.scope && c.session_id == id
        });
        return Cell {
            present: false,
            folder: None,
            archived: false,
            parent_id: None,
            status: if copied {
                Status::WillCreate
            } else {
                Status::Missing
            },
        };
    };
    let others: Vec<(&Chat, Option<&String>)> = cols
        .iter()
        .filter(|c| c.column.key != col.column.key)
        .filter_map(|c| Some((c.chats.get(id)?, c.folders.of_chat.get(id))))
        .collect();
    let status = chat_status(chat, folder.as_ref(), &others, cols.len());
    Cell {
        present: true,
        folder,
        archived: chat.archived,
        parent_id: chat.parent.clone(),
        status,
    }
}

/// A chat of another account that holds the same conversation, with the folder it sits in there.
type Peer<'a> = (&'a Chat, Option<&'a String>);

fn chat_status(chat: &Chat, folder: Option<&String>, others: &[Peer], columns: usize) -> Status {
    if others.is_empty() {
        return if columns > 1 {
            Status::OnlyHere
        } else {
            Status::Same
        };
    }
    let any = |f: &dyn Fn(&Peer) -> bool| others.iter().any(f);
    if chat.archived && any(&|(o, _)| !o.archived) {
        Status::ArchivedHereOnly
    } else if !chat.archived && any(&|(o, _)| o.archived) {
        Status::ActiveHereOnly
    } else if chat.parent.is_some() && any(&|(o, _)| o.parent.is_none()) {
        Status::Subthread
    } else if chat.parent.is_none() && any(&|(o, _)| o.parent.is_some()) {
        Status::NotSubthread
    } else if any(&|(_, f)| *f != folder) {
        Status::FolderDiffers
    } else {
        Status::Same
    }
}

fn folder_cell(cols: &[Col], col: &Col, planned: &Planned, name: &str) -> Cell {
    let present = col.folders.names.iter().any(|n| n == name);
    let status = if present {
        let everywhere = cols
            .iter()
            .all(|c| c.folders.names.iter().any(|n| n == name));
        if everywhere {
            Status::Same
        } else {
            Status::OnlyHere
        }
    } else {
        let planned_here = planned.folders.steps.iter().any(|s| {
            matches!(s, FolderStep::CreateFolder { scope, name: n, .. }
                if *scope == col.scope.name() && n == name)
        });
        if planned_here {
            Status::WillCreate
        } else {
            Status::Missing
        }
    };
    Cell {
        present,
        folder: present.then(|| name.to_owned()),
        archived: false,
        parent_id: None,
        status,
    }
}

fn row_of(cols: &[Col], planned: &Planned, id: &str, depth: u8) -> Row {
    let name = id.strip_prefix("folder:");
    let cells = cols
        .iter()
        .map(|col| {
            let cell = match name {
                Some("") => Cell {
                    present: true,
                    folder: None,
                    archived: false,
                    parent_id: None,
                    status: Status::Same,
                },
                Some(n) => folder_cell(cols, col, planned, n),
                None => chat_cell(cols, col, planned, id),
            };
            (col.column.key.clone(), cell)
        })
        .collect();
    let title = match name {
        Some("") => "No folder".to_owned(),
        Some(n) => n.to_owned(),
        None => place_of(cols, id).map(|p| p.title).unwrap_or_default(),
    };
    let parent_id = (depth == 2)
        .then(|| place_of(cols, id).and_then(|p| p.parent))
        .flatten();
    Row {
        id: id.to_owned(),
        kind: if name.is_some() { "folder" } else { "chat" },
        depth,
        parent_id,
        title,
        cells,
    }
}

/// The folder names of every account, in the order the accounts list them.
fn folder_union(cols: &[Col]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut names = Vec::new();
    for name in cols.iter().flat_map(|c| &c.folders.names) {
        if seen.insert(name.clone()) {
            names.push(name.clone());
        }
    }
    names
}

/// Builds the rows from a scan and its plan.
#[must_use]
pub fn build_compare(planned: &Planned, accounts: &[Account]) -> Compare {
    let cols = columns(planned, accounts);
    let ids: BTreeSet<&String> = cols.iter().flat_map(|c| c.chats.keys()).collect();
    let places: BTreeMap<String, Place> = ids
        .into_iter()
        .filter_map(|id| Some((id.clone(), place_of(&cols, id)?)))
        .collect();
    let parents: HashMap<&String, String> = places
        .keys()
        .filter_map(|id| Some((id, usable_parent(&places, id)?)))
        .collect();
    let mut rows = Vec::new();
    let mut groups = folder_union(&cols);
    groups.push(String::new());
    for name in groups {
        let mut top: Vec<&String> = places
            .iter()
            .filter(|(id, p)| {
                p.folder.as_deref().unwrap_or("") == name && !parents.contains_key(id)
            })
            .map(|(id, _)| id)
            .collect();
        if top.is_empty() && name.is_empty() {
            continue;
        }
        top.sort_by_key(|id| {
            (
                places[*id].rank,
                Reverse(places[*id].focused),
                (*id).clone(),
            )
        });
        rows.push(row_of(&cols, planned, &format!("folder:{name}"), 0));
        for id in top {
            rows.push(row_of(&cols, planned, id, 1));
            push_children(&mut rows, &cols, planned, (&places, &parents), id);
        }
    }
    let (same, different) = tally(&rows);
    Compare {
        scanned_at: planned.scan.at,
        accounts: cols.iter().map(|c| c.column.clone()).collect(),
        rows,
        counts: Counts {
            same,
            different,
            by_kind: drift_of(planned).by_kind,
        },
    }
}

/// The sub-threads of a chat, newest first, each followed by its own.
fn push_children(
    rows: &mut Vec<Row>,
    cols: &[Col],
    planned: &Planned,
    (places, parents): (&BTreeMap<String, Place>, &HashMap<&String, String>),
    parent: &str,
) {
    let mut kids: Vec<&String> = parents
        .iter()
        .filter(|(_, p)| p.as_str() == parent)
        .map(|(id, _)| *id)
        .collect();
    kids.sort_by_key(|id| (Reverse(places[*id].focused), (*id).clone()));
    for id in kids {
        rows.push(row_of(cols, planned, id, 2));
        push_children(rows, cols, planned, (places, parents), id);
    }
}

fn tally(rows: &[Row]) -> (usize, usize) {
    let same = rows
        .iter()
        .filter(|r| r.cells.values().all(|c| c.status == Status::Same))
        .count();
    (same, rows.len() - same)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::catalog::plan::Plan;
    use crate::fixtures::account_in;
    use crate::sidebar::{FolderPlan, SidebarKeys};
    use crate::sync::job::{Scan, ScopeScan};

    const A: &str = "acct-a/org-a";
    const B: &str = "acct-b/org-b";

    fn account(key: &str, scope: &str) -> Account {
        let (acct, org) = scope.split_once('/').unwrap();
        account_in(
            key,
            &key.to_uppercase(),
            Some(acct.to_owned()),
            Some(org.to_owned()),
        )
    }

    fn chat(title: &str, focused: i64, parent: Option<&str>, archived: bool) -> Chat {
        Chat {
            title: title.to_owned(),
            archived,
            focused,
            parent: parent.map(str::to_owned),
            record: json!({}),
        }
    }

    fn scope_scan(scope: &str, chats: Vec<(&str, Chat)>) -> ScopeScan {
        let (account, org) = scope.split_once('/').unwrap();
        ScopeScan {
            scope: Scope {
                account: account.into(),
                org: org.into(),
            },
            chats: chats
                .into_iter()
                .map(|(id, c)| (id.to_owned(), c))
                .collect(),
        }
    }

    /// A folder: its name and the chat ids in order.
    type FolderSpec<'a> = (&'a str, Vec<&'a str>);

    /// Folders per scope.
    fn keys(per_scope: &[(&str, Vec<FolderSpec>)]) -> SidebarKeys {
        let mut groups = serde_json::Map::new();
        for (scope, folders) in per_scope {
            let mut list = Vec::new();
            let mut assignments = serde_json::Map::new();
            let mut order = serde_json::Map::new();
            for (name, ids) in folders {
                let fid = format!("cg-{name}");
                list.push(json!({"id": fid, "name": name}));
                let members: Vec<String> = ids.iter().map(|i| format!("code:{i}")).collect();
                for m in &members {
                    assignments.insert(m.clone(), json!(fid));
                }
                order.insert(fid, json!(members));
            }
            groups.insert(
                (*scope).into(),
                json!({"groups": list, "assignments": assignments, "order": order}),
            );
        }
        SidebarKeys {
            code_sections: json!({"value": {}}),
            group_scopes: json!({"value": groups}),
            dframe_store: json!({"state": {}}),
            encodings: [1, 1, 1],
        }
    }

    fn planned(scopes: Vec<ScopeScan>, keys: SidebarKeys) -> Planned {
        Planned {
            scan: Scan {
                at: 5,
                scopes,
                keys,
            },
            catalog: Plan::default(),
            kinds: Vec::new(),
            folders: FolderPlan {
                steps: Vec::new(),
                keys: SidebarKeys {
                    code_sections: json!({}),
                    group_scopes: json!({}),
                    dframe_store: json!({}),
                    encodings: [1, 1, 1],
                },
            },
        }
    }

    #[test]
    fn rows_run_folder_by_folder_with_sub_threads_under_their_parent() {
        let a = scope_scan(
            A,
            vec![
                ("local_1", chat("one", 10, None, false)),
                ("local_2", chat("two", 30, Some("local_1"), false)),
                ("local_3", chat("loose", 20, None, false)),
                ("local_4", chat("beta chat", 5, None, false)),
                ("local_5", chat("first in alpha", 50, None, true)),
            ],
        );
        let b = scope_scan(B, vec![("local_1", chat("one", 10, None, false))]);
        let k = keys(&[
            (
                A,
                vec![
                    ("Alpha", vec!["local_5", "local_1"]),
                    ("Beta", vec!["local_4"]),
                ],
            ),
            (B, vec![("Beta", vec![]), ("Alpha", vec!["local_1"])]),
        ]);
        let compare = build_compare(&planned(vec![a, b], k), &[account("a", A), account("b", B)]);
        let shown: Vec<(&str, u8)> = compare
            .rows
            .iter()
            .map(|r| (r.id.as_str(), r.depth))
            .collect();
        assert_eq!(
            shown,
            [
                ("folder:Alpha", 0),
                ("local_5", 1),
                ("local_1", 1),
                ("local_2", 2),
                ("folder:Beta", 0),
                ("local_4", 1),
                ("folder:", 0),
                ("local_3", 1),
            ]
        );
        assert_eq!(compare.rows[3].parent_id.as_deref(), Some("local_1"));
        assert_eq!(compare.accounts.len(), 2);
        assert_eq!(compare.scanned_at, 5);
    }

    #[test]
    fn cells_say_how_each_account_differs() {
        let a = scope_scan(
            A,
            vec![
                ("local_1", chat("one", 10, None, true)),
                ("local_2", chat("two", 10, None, false)),
            ],
        );
        let b = scope_scan(B, vec![("local_1", chat("one", 10, None, false))]);
        let k = keys(&[(A, vec![("Alpha", vec![])]), (B, vec![])]);
        let compare = build_compare(&planned(vec![a, b], k), &[account("a", A), account("b", B)]);
        let row = |id: &str| compare.rows.iter().find(|r| r.id == id).unwrap();
        assert_eq!(row("local_1").cells["a"].status, Status::ArchivedHereOnly);
        assert_eq!(row("local_1").cells["b"].status, Status::ActiveHereOnly);
        assert_eq!(row("local_2").cells["a"].status, Status::OnlyHere);
        assert_eq!(row("local_2").cells["b"].status, Status::Missing);
        assert!(!row("local_2").cells["b"].present);
        assert_eq!(row("folder:Alpha").cells["a"].status, Status::OnlyHere);
        assert_eq!(row("folder:Alpha").cells["b"].status, Status::Missing);
        assert_eq!(compare.counts.same, 1, "only the no-folder header agrees");
    }

    #[test]
    fn a_parent_cycle_does_not_hide_chats() {
        let a = scope_scan(
            A,
            vec![
                ("local_1", chat("one", 10, Some("local_2"), false)),
                ("local_2", chat("two", 20, Some("local_1"), false)),
            ],
        );
        let compare = build_compare(&planned(vec![a], keys(&[(A, vec![])])), &[account("a", A)]);
        let ids: Vec<&str> = compare.rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["folder:", "local_2", "local_1"]);
        assert!(compare.rows.iter().skip(1).all(|r| r.depth == 1));
    }
}
