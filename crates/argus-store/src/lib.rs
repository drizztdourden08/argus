//! The Argus database: one SQLite file opened with the Node pragmas and migrations, the doc
//! store (the page-facing mirror and its change feed) and kv. Writes run in `BEGIN IMMEDIATE`
//! transactions; doc events reach the bus only after the outermost commit.
#![deny(clippy::cognitive_complexity)]

mod activity;
mod activity_entry;
mod activity_notes;
mod db;
mod docs;
mod error;
mod migrations;
mod query;
mod snapshot;
mod tx;

pub use activity::{ActivityLog, KEEP_ROWS, LogNote, MIRROR_ROWS, SCOPE as ACTIVITY_SCOPE};
pub use activity_entry::{
    ActivityEntry, ActivityFilter, ActivityLevel, NewActivity, TEXT_MAX, one_line,
};
pub use db::{Db, DocStore};
pub use docs::{Applied, DocRow, PruneOptions};
pub use error::StoreError;
pub use migrations::MIGRATIONS;
pub use query::{Query, QueryOp};
pub use rusqlite;
pub use snapshot::snapshot;
pub use tx::Tx;
