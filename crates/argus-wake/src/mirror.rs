//! The page mirror of the wakes table: the last 50 rows of a project as documents of `wakes`.

use serde::Serialize;

use crate::docs::{Doc, DocSink};
use crate::store::{MIRROR_KEEP, RowsQuery, StoreResult, WakeStore};
use crate::wake::{WakeRow, WakeStatus};

/// The document collection of the wakes.
pub const WAKES_COLLECTION: &str = "wakes";

/// A wake row as the page shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WakeDoc {
    /// `W-n`.
    pub id: String,
    /// Target thread.
    pub thread: String,
    /// Why it was made.
    pub reason: String,
    /// What it is about.
    pub subject: Option<String>,
    /// The line the thread hears.
    pub text: String,
    /// When it was made (ISO).
    pub at: String,
    /// Queued, delivered, coalesced or passive (a notice, never sent to a listener).
    pub status: WakeStatus,
    /// How many listeners took the event (a coalesced row shows its event's).
    pub listeners: u32,
    /// The event id a coalesced row folds into.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub into: Option<String>,
    /// How many wakes an event folds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
}

/// The document of a row; `lead_listeners` is what the event of a coalesced row got.
#[must_use]
pub fn row_doc(row: &WakeRow, lead_listeners: u32) -> WakeDoc {
    let coalesced = row.status == WakeStatus::Coalesced;
    WakeDoc {
        id: row.id.clone(),
        thread: row.thread.clone(),
        reason: row.reason.clone(),
        subject: row.subject.clone(),
        text: row.text.clone(),
        at: row.at.clone(),
        status: row.status,
        listeners: if coalesced {
            lead_listeners
        } else {
            row.listeners
        },
        into: if coalesced {
            row.merged_into.clone()
        } else {
            None
        },
        count: if coalesced { None } else { Some(row.count) },
    }
}

/// The documents of a list of rows, looking up the event of each coalesced one.
///
/// # Errors
/// When an event row cannot be read.
pub fn docs_of(store: &dyn WakeStore, rows: &[WakeRow]) -> StoreResult<Vec<WakeDoc>> {
    rows.iter()
        .map(|row| {
            let lead = lead_listeners(store, rows, row)?;
            Ok(row_doc(row, lead))
        })
        .collect()
}

fn lead_listeners(store: &dyn WakeStore, rows: &[WakeRow], row: &WakeRow) -> StoreResult<u32> {
    let Some(lead_id) = row.merged_into.as_deref() else {
        return Ok(0);
    };
    if let Some(lead) = rows.iter().find(|r| r.id == lead_id) {
        return Ok(lead.listeners);
    }
    Ok(store.get(lead_id)?.map_or(0, |lead| lead.listeners))
}

/// Replace the `wakes` documents of a project with its last [`MIRROR_KEEP`] rows.
///
/// # Errors
/// When rows cannot be read or documents cannot be written.
pub fn mirror_wakes(store: &dyn WakeStore, sink: &dyn DocSink, scope: &str) -> StoreResult<()> {
    let query = RowsQuery {
        limit: MIRROR_KEEP,
        thread: None,
    };
    let rows = store.rows(scope, &query)?;
    let docs = docs_of(store, &rows)?
        .into_iter()
        .map(|doc| Doc {
            id: doc.id.clone(),
            data: serde_json::to_value(&doc).unwrap_or_default(),
        })
        .collect();
    sink.replace(scope, WAKES_COLLECTION, docs)
}
