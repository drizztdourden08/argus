//! The difference of a row and its default: the choice whose outcome is what the automatic
//! plan writes. A chat the automatic plan leaves alone (its copies disagree, no transcript, a
//! tombstone) defaults to `skip`.

use serde_json::Value;

use super::records::Records;
use super::{Choice, Diff, DiffKind};
use crate::catalog::merge::fingerprint;
use crate::sidebar::{FolderStep, chat_folder};
use crate::sync::compare;
use crate::sync::job::Planned;

/// An account with a home scope in the scan.
pub(crate) struct Home {
    /// The account key.
    pub(crate) key: String,
    /// The account's name on screen.
    pub(crate) label: String,
    /// The index of its home scope in the scan.
    pub(crate) index: usize,
    /// The home scope's name.
    pub(crate) scope: String,
}

/// What the differences are worked out from.
pub(crate) struct Look<'a> {
    /// The scan and its automatic plan.
    pub(crate) planned: &'a Planned,
    /// The records once the automatic plan is written.
    pub(crate) records: Records<'a>,
    /// The accounts' home scopes.
    pub(crate) homes: Vec<Home>,
}

fn choice(id: String, label: String) -> Choice {
    Choice { id, label }
}

fn skip() -> Choice {
    choice("skip".to_owned(), "Skip".to_owned())
}

fn diff(kind: DiffKind, mut choices: Vec<Choice>, default: Option<String>) -> Diff {
    choices.push(skip());
    Diff {
        kind,
        choices,
        default: default.unwrap_or_else(|| "skip".to_owned()),
    }
}

/// The one value every item has, if they agree.
fn agreed<T: PartialEq + Clone>(items: &[T]) -> Option<T> {
    let first = items.first()?;
    items.iter().all(|i| i == first).then(|| first.clone())
}

fn differ<T: PartialEq + Clone>(items: &[T]) -> bool {
    agreed(items).is_none()
}

/// The difference of a chat row.
pub(crate) fn chat_diff(look: &Look, row: &compare::Row) -> Option<Diff> {
    let id = row.id.as_str();
    let scan = &look.planned.scan;
    let chats: Vec<_> = look
        .homes
        .iter()
        .map(|h| scan.scopes[h.index].chats.get(id))
        .collect();
    if chats.iter().all(Option::is_none) {
        return None;
    }
    if chats.iter().any(Option::is_none) {
        return Some(missing(look, id));
    }
    let chats: Vec<_> = chats.into_iter().flatten().collect();
    let prints: Vec<String> = chats.iter().map(|c| fingerprint(&c.record)).collect();
    if differ(&prints) {
        return Some(history(look, id));
    }
    let archived: Vec<bool> = chats.iter().map(|c| c.archived).collect();
    if differ(&archived) {
        return Some(archive(look, id));
    }
    let parents: Vec<Option<String>> = chats.iter().map(|c| c.parent.clone()).collect();
    if differ(&parents) {
        return Some(thread(look, id));
    }
    let folders: Vec<Option<String>> = look
        .homes
        .iter()
        .map(|h| row.cells.get(&h.key).and_then(|c| c.folder.clone()))
        .collect();
    if differ(&folders) {
        return Some(membership(look, id, &folders));
    }
    None
}

fn missing(look: &Look, id: &str) -> Diff {
    let scan = &look.planned.scan;
    let copied: Vec<&Home> = look
        .homes
        .iter()
        .filter(|h| !scan.scopes[h.index].chats.contains_key(id))
        .filter(|h| look.records.wanted(h.index, id).is_some())
        .collect();
    let choices = copied
        .iter()
        .map(|h| choice(format!("copy_to:{}", h.key), format!("Copy to {}", h.label)))
        .collect();
    let default = copied.first().map(|h| format!("copy_to:{}", h.key));
    diff(DiffKind::ChatMissing, choices, default)
}

fn history(look: &Look, id: &str) -> Diff {
    let plan = &look.planned.catalog;
    let blocked = plan.conflicts.iter().any(|c| c.session_id == id)
        || plan.missing_transcripts.iter().any(|m| m == id);
    let mut choices = Vec::new();
    if !blocked {
        choices.push(choice("newest".to_owned(), "Newest".to_owned()));
    }
    for h in &look.homes {
        choices.push(choice(
            format!("take:{}", h.key),
            format!("Take {}'s", h.label),
        ));
    }
    let default = (!blocked).then(|| "newest".to_owned());
    diff(DiffKind::History, choices, default)
}

fn archive(look: &Look, id: &str) -> Diff {
    let wanted: Vec<Option<bool>> = look
        .homes
        .iter()
        .map(|h| look.records.archived(h.index, id))
        .collect();
    let default = match agreed(&wanted).flatten() {
        Some(true) => Some("archive_all".to_owned()),
        Some(false) => Some("unarchive_all".to_owned()),
        None => None,
    };
    let choices = vec![
        choice("archive_all".to_owned(), "Archive on all".to_owned()),
        choice("unarchive_all".to_owned(), "Unarchive on all".to_owned()),
    ];
    diff(DiffKind::Archive, choices, default)
}

fn thread(look: &Look, id: &str) -> Diff {
    let wanted: Vec<bool> = look
        .homes
        .iter()
        .map(|h| look.records.parent(h.index, id).is_some())
        .collect();
    let default =
        agreed(&wanted).map(|linked| if linked { "link_all" } else { "unlink_all" }.to_owned());
    let choices = vec![
        choice("link_all".to_owned(), "Link on all".to_owned()),
        choice("unlink_all".to_owned(), "Unlink on all".to_owned()),
    ];
    diff(DiffKind::Thread, choices, default)
}

fn membership(look: &Look, id: &str, folders: &[Option<String>]) -> Diff {
    let after = &look.planned.folders.keys;
    let finals: Vec<Option<String>> = look
        .homes
        .iter()
        .map(|h| chat_folder(after, &h.scope, id))
        .collect();
    let default = agreed(&finals).and_then(|f| {
        let at = folders.iter().position(|now| *now == f)?;
        Some(format!("folder_of:{}", look.homes[at].key))
    });
    let choices = look
        .homes
        .iter()
        .zip(folders)
        .map(|(h, f)| {
            let shown = f.as_deref().unwrap_or("no folder");
            choice(
                format!("folder_of:{}", h.key),
                format!("{}'s folder ({shown})", h.label),
            )
        })
        .collect();
    diff(DiffKind::Membership, choices, default)
}

/// The difference of a folder row; the "No folder" row has none.
pub(crate) fn folder_diff(look: &Look, row: &compare::Row) -> Option<Diff> {
    let name = row.id.strip_prefix("folder:").filter(|n| !n.is_empty())?;
    let present = |h: &&Home| row.cells.get(&h.key).is_some_and(|c| c.present);
    let (have, lack): (Vec<&Home>, Vec<&Home>) = look.homes.iter().partition(present);
    if lack.is_empty() || have.is_empty() {
        return None;
    }
    let created: Vec<&&Home> = lack
        .iter()
        .filter(|h| {
            look.planned.folders.steps.iter().any(|s| {
                matches!(s, FolderStep::CreateFolder { scope, name: n, .. }
                    if *scope == h.scope && n == name)
            })
        })
        .collect();
    let mut choices: Vec<Choice> = created
        .iter()
        .map(|h| {
            choice(
                format!("create_on:{}", h.key),
                format!("Create on {}", h.label),
            )
        })
        .collect();
    choices.extend(have.iter().map(|h| {
        choice(
            format!("delete_on:{}", h.key),
            format!("Delete on {}", h.label),
        )
    }));
    let default = created.first().map(|h| format!("create_on:{}", h.key));
    Some(diff(DiffKind::FolderMissing, choices, default))
}

/// The record of the chat in an account's home scope, as the scan read it.
pub(crate) fn home_record<'a>(look: &'a Look, key: &str, id: &str) -> Option<&'a Value> {
    let home = look.homes.iter().find(|h| h.key == key)?;
    look.records.current(home.index, id)
}
