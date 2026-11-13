//! `board-ids.journal.json`: which boards moved, where the verified backup is, and whether the
//! run finished. Rollback reverses the run from it.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use argus_core::clock::now_iso;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::jsonio;
use crate::plan::Plan;

/// One board of the run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The old key.
    pub key: String,
    /// The new id.
    pub id: String,
    /// `pending`, then `renamed` once the folder is at the id.
    pub phase: String,
}

/// The journal of a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    /// Format version, 1.
    pub version: u32,
    /// When the run began.
    pub started: String,
    /// The verified backup folder.
    pub backup: Option<String>,
    /// The boards.
    pub boards: Vec<Entry>,
    /// Set when the run passed its verification.
    pub complete: bool,
}

/// The journal file of a home.
pub fn path(home: &Path) -> PathBuf {
    home.join("board-ids.journal.json")
}

impl Journal {
    /// A new journal for the boards of `plan`.
    pub fn new(plan: &Plan) -> Self {
        let boards = plan
            .boards
            .iter()
            .map(|(key, p)| Entry {
                key: key.clone(),
                id: p.id.clone(),
                phase: "pending".to_owned(),
            })
            .collect();
        Journal {
            version: 1,
            started: now_iso(),
            backup: None,
            boards,
            complete: false,
        }
    }

    /// Adds an entry for every board of `plan` the journal does not hold.
    pub fn ensure(&mut self, plan: &Plan) {
        for (key, p) in &plan.boards {
            if !self.boards.iter().any(|e| e.key == *key) {
                self.boards.push(Entry {
                    key: key.clone(),
                    id: p.id.clone(),
                    phase: "pending".to_owned(),
                });
            }
        }
    }

    /// The journal of a home, if a run left one.
    pub fn load(home: &Path) -> Result<Option<Journal>> {
        match fs::read_to_string(path(home)) {
            Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Writes the journal.
    pub fn save(&self, home: &Path) -> Result<()> {
        jsonio::write(&path(home), &serde_json::to_value(self)?)
    }

    /// Records the phase of a board.
    pub fn phase(&mut self, home: &Path, key: &str, phase: &str) -> Result<()> {
        if let Some(e) = self.boards.iter_mut().find(|e| e.key == key) {
            phase.clone_into(&mut e.phase);
        }
        self.save(home)
    }
}
