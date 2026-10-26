//! Doc changes: what an export or a write asks the store to do, and what it did.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Coll;

/// Set or delete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocOp {
    /// Insert or replace the doc.
    Set,
    /// Remove the doc.
    Delete,
}

impl DocOp {
    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            DocOp::Set => "set",
            DocOp::Delete => "delete",
        }
    }
}

/// One doc change. `data` is present on a set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocChange {
    /// The collection.
    pub coll: Coll,
    /// The doc id.
    pub id: String,
    /// Set or delete.
    pub op: DocOp,
    /// The doc on a set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl DocChange {
    /// A set of `data` at `coll/id`.
    #[must_use]
    pub fn set(coll: Coll, id: impl Into<String>, data: Value) -> Self {
        DocChange {
            coll,
            id: id.into(),
            op: DocOp::Set,
            data: Some(data),
        }
    }

    /// A delete of `coll/id`.
    #[must_use]
    pub fn delete(coll: Coll, id: impl Into<String>) -> Self {
        DocChange {
            coll,
            id: id.into(),
            op: DocOp::Delete,
            data: None,
        }
    }

    /// The `coll/id` key of the doc.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}/{}", self.coll, self.id)
    }
}

/// A change the store wrote, with the seq it got.
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedChange {
    /// The change.
    pub change: DocChange,
    /// Its row in the change log.
    pub seq: i64,
}

/// One apply of docs: what the streams fan out.
#[derive(Debug, Clone, PartialEq)]
pub struct DocsEvent {
    /// The scope written.
    pub scope: String,
    /// The seq of the last change.
    pub seq: i64,
    /// What changed, in order.
    pub changes: Vec<AppliedChange>,
}

/// A scope's streams must drop their state and reconnect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetEvent {
    /// The scope.
    pub scope: String,
    /// Why, for example `resync`.
    pub reason: String,
}
