//! The page-facing documents: collections the pages read, replaced one project at a time.

use serde_json::Value;

use crate::store::StoreResult;

/// One document of a collection.
#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    /// The document id.
    pub id: String,
    /// The document body.
    pub data: Value,
}

/// Where the mirrors are written.
pub trait DocSink: Send + Sync {
    /// Replace every document of `collection` in `scope` with `docs`.
    ///
    /// # Errors
    /// When the documents cannot be written.
    fn replace(&self, scope: &str, collection: &str, docs: Vec<Doc>) -> StoreResult<()>;
}
