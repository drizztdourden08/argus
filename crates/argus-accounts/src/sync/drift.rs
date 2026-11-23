//! How far the accounts have drifted apart: the counts of a dry-run plan by kind, shown in the
//! `drift` doc of `AccountsState`.

use serde::Serialize;
use serde_json::{Value, json, to_value};

use super::job::Planned;
use super::steps::StepKind;
use crate::sidebar::FolderStep;

/// Changes a sync would make, by kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct ByKind {
    /// Chats to copy or bring up to date.
    pub chat: usize,
    /// Archive indexes to rewrite.
    pub archive: usize,
    /// Folders to create.
    pub folder: usize,
    /// Chats to move into a folder, and folder orders to rewrite.
    pub membership: usize,
    /// Sub-thread links to set.
    pub thread: usize,
}

/// The counts of a dry run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drift {
    /// When the scan ran, epoch milliseconds.
    pub scanned_at: i64,
    /// All changes.
    pub total: usize,
    /// The changes by kind.
    pub by_kind: ByKind,
}

impl Drift {
    /// The `drift` doc.
    #[must_use]
    pub fn to_doc(&self) -> Value {
        to_value(self).unwrap_or_else(|_| json!({}))
    }
}

/// Counts what the plan would change.
#[must_use]
pub fn drift_of(planned: &Planned) -> Drift {
    let mut by = ByKind::default();
    for kind in &planned.kinds {
        match kind {
            StepKind::CopySession | StepKind::UpdateSession => by.chat += 1,
            StepKind::Archive => by.archive += 1,
            StepKind::LinkThread => by.thread += 1,
            _ => {}
        }
    }
    for step in &planned.folders.steps {
        match step {
            FolderStep::CreateFolder { .. }
            | FolderStep::RenameFolder { .. }
            | FolderStep::DeleteFolder { .. }
            | FolderStep::SetView { .. } => by.folder += 1,
            FolderStep::AssignChat { .. }
            | FolderStep::ReorderFolder { .. }
            | FolderStep::MoveChat { .. } => by.membership += 1,
        }
    }
    Drift {
        scanned_at: planned.scan.at,
        total: by.chat + by.archive + by.folder + by.membership + by.thread,
        by_kind: by,
    }
}
