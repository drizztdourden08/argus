//! Where run definitions come from: `board.runDefs` of `<coord home>/<key>/board.json`,
//! written only by `coord run define` and the launcher verbs. The file is read on every
//! start (boards are small and starts are rare), so a definition made a second ago is usable
//! at once.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use serde_json::Value;

use crate::def::{RunDef, defs_from_map};

pub trait DefsSource: Send + Sync {
    /// The definitions of a project, sorted by name; empty when it has none.
    fn defs(&self, key: &str) -> Vec<RunDef>;
}

/// Reads the board file; a missing or half-written file falls back to the last good copy
/// another source (the sync service) keeps.
pub struct BoardFileDefs {
    coord_home: PathBuf,
    fallback: Option<Arc<dyn DefsSource>>,
}

impl BoardFileDefs {
    pub fn new(coord_home: impl Into<PathBuf>, fallback: Option<Arc<dyn DefsSource>>) -> Self {
        Self {
            coord_home: coord_home.into(),
            fallback,
        }
    }

    fn read_file(&self, key: &str) -> Option<Vec<RunDef>> {
        let path = self.coord_home.join(key).join("board.json");
        let board: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
        let defs = board.get("runDefs").and_then(Value::as_object);
        Some(defs.map(defs_from_map).unwrap_or_default())
    }
}

impl DefsSource for BoardFileDefs {
    fn defs(&self, key: &str) -> Vec<RunDef> {
        self.read_file(key)
            .or_else(|| self.fallback.as_ref().map(|f| f.defs(key)))
            .unwrap_or_default()
    }
}

/// Definitions held in memory.
#[derive(Default)]
pub struct MemoryDefs {
    defs: Mutex<BTreeMap<String, Vec<RunDef>>>,
}

impl MemoryDefs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, key: &str, mut defs: Vec<RunDef>) {
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        self.defs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.to_owned(), defs);
    }
}

impl DefsSource for MemoryDefs {
    fn defs(&self, key: &str) -> Vec<RunDef> {
        self.defs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(key)
            .cloned()
            .unwrap_or_default()
    }
}
