//! The page mirrors of the wake engine and its registry: one collection of one scope replaced
//! at a time, in the doc store.

use std::sync::Arc;

use argus_core::{Coll, DocChange, StorageError};
use argus_store::Db;
use argus_wake::docs::{Doc, DocSink};
use argus_wake::store::StoreResult;

/// Writes the `wakes` and `registry` collections.
pub struct WakeDocs {
    db: Arc<Db>,
}

impl WakeDocs {
    /// The docs of a database.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        WakeDocs { db }
    }
}

impl DocSink for WakeDocs {
    fn replace(&self, scope: &str, collection: &str, docs: Vec<Doc>) -> StoreResult<()> {
        let coll: Coll = collection
            .parse()
            .map_err(|_| StorageError(format!("unknown collection \"{collection}\"")))?;
        let all = docs
            .into_iter()
            .map(|d| DocChange::set(coll, d.id, d.data))
            .collect();
        self.db
            .docs()
            .replace(scope, &[coll], all)
            .map(|_| ())
            .map_err(|e| StorageError(format!("{collection} docs of {scope}: {e}")))
    }
}
