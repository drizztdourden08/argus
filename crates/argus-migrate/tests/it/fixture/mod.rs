//! A home shaped like the live one: several boards with proofs, the portfolio, a database with
//! every table and kv format, listen stamps, `remote.json` and `config.json`.

mod boards;
mod db;
pub mod snapshot;

use std::fs;
use std::path::PathBuf;

use argus_migrate::{Fault, Mode, Options};
use tempfile::TempDir;

/// The board keys of the fixture.
pub const KEYS: [&str; 5] = ["alpha", "beta-two", "gamma", "portfolio", "argus"];

fn never(_pid: u32) -> bool {
    false
}

/// The fixture home inside a temp folder.
pub struct Fixture {
    /// The temp folder (also holds the backups, as siblings of the home).
    pub dir: TempDir,
    /// The coord home.
    pub home: PathBuf,
}

impl Fixture {
    /// A new fixture home; every call builds the same content.
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("coord");
        fs::create_dir_all(home.join("dashboard")).unwrap();
        boards::write_all(&home);
        db::seed(&home.join("dashboard").join("dashboard.db"));
        Fixture { dir, home }
    }

    /// Options for a run in `mode`, no fault.
    pub fn options(&self, mode: Mode) -> Options {
        self.faulty(mode, None)
    }

    /// Options for a run with an injected fault.
    pub fn faulty(&self, mode: Mode, fault: Option<Fault>) -> Options {
        Options {
            home: self.home.clone(),
            mode,
            pid_alive: never,
            fault,
        }
    }

    /// The backup folder a run made next to the home.
    pub fn backup_dir(&self) -> PathBuf {
        fs::read_dir(self.dir.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("coord-backup-")
            })
            .unwrap()
    }
}
