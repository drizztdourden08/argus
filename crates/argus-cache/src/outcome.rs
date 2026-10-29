//! What a prune did.

use crate::entry::{Entry, Kind};
use serde::Serialize;

/// A `<buildKey>` folder a prune deleted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deleted {
    pub kind: Kind,
    pub name: String,
    pub version: String,
    pub build_key: String,
    pub size_bytes: u64,
}

impl Deleted {
    pub fn of(entry: &Entry) -> Self {
        Deleted {
            kind: entry.kind,
            name: entry.name.clone(),
            version: entry.version.clone(),
            build_key: entry.build_key.clone(),
            size_bytes: entry.size_bytes,
        }
    }
}

/// The result of one prune.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    /// When it ran, ISO text.
    pub at: String,
    pub deleted: Vec<Deleted>,
    /// Left-over `.tmp/` folders deleted.
    pub tmp_deleted: usize,
    pub freed_bytes: u64,
    /// Paths outside the cache root that a prune refused to touch.
    pub refused: Vec<String>,
    /// Deletes that failed, with the reason; nothing partial is left in the cache.
    pub failed: Vec<String>,
}

fn megabytes(bytes: u64) -> String {
    let tenths = bytes.saturating_mul(10) / (1024 * 1024);
    format!("{}.{} MB", tenths / 10, tenths % 10)
}

fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

impl Outcome {
    /// Did it delete, refuse and fail nothing?
    pub fn is_quiet(&self) -> bool {
        self.deleted.is_empty()
            && self.tmp_deleted == 0
            && self.refused.is_empty()
            && self.failed.is_empty()
    }

    /// The activity line: counts and bytes freed.
    pub fn summary(&self) -> String {
        let mut parts = vec![
            counted(self.deleted.len(), "entry", "entries"),
            counted(self.tmp_deleted, "temp folder", "temp folders"),
            format!("{} freed", megabytes(self.freed_bytes)),
        ];
        if !self.refused.is_empty() {
            parts.push(format!("{} refused", self.refused.len()));
        }
        if !self.failed.is_empty() {
            parts.push(format!("{} failed", self.failed.len()));
        }
        format!("build cache pruned: {}", parts.join(", "))
    }
}
