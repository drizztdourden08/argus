//! What one run is asked to do.

use std::path::PathBuf;

/// The mode of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Write the plan and report; change nothing else.
    DryRun,
    /// Migrate (or finish an interrupted migration, or verify a finished one).
    Migrate,
    /// Reverse the migration from its journal.
    Rollback,
    /// Put a backup back; the current home is renamed aside.
    Restore(PathBuf),
}

/// A fault a test injects to stop a run in the middle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// Stop after the n-th board (counted from 1) is moved.
    AfterBoard(usize),
    /// Stop after the database step.
    AfterDb,
    /// Damage the backup after it is copied.
    CorruptBackup,
}

/// One run.
#[derive(Clone)]
pub struct Options {
    /// The coord home (default `~/.coord`).
    pub home: PathBuf,
    /// What to do.
    pub mode: Mode,
    /// Whether a process with this id runs.
    pub pid_alive: fn(u32) -> bool,
    /// A fault to inject; `None` in the real binary.
    pub fault: Option<Fault>,
}

impl Options {
    /// The dashboard home, `<home>/dashboard`.
    #[must_use]
    pub fn dashboard(&self) -> PathBuf {
        self.home.join("dashboard")
    }
}
