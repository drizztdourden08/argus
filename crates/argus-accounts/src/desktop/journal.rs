//! The switch journal: a small file written before the target's login is put in place and removed
//! once the switch ended or was rolled back. A journal that is still there means a switch was cut
//! short (Argus quit or crashed) while Desktop's login may be half replaced.
//!
//! A leftover journal is repaired by putting the saved login of its `from` account back, only
//! while Claude is closed, and never by saving the login on disk (that could overwrite a good
//! vault entry with a half replaced one).

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::now_iso;
use serde::{Deserialize, Serialize};

use crate::catalog::read::write_atomic;
use crate::ports::{DesktopApp, LoginVault};
use crate::types::Account;

/// What the journal says about the switch that was cut short.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The switch id, `SW-n`.
    pub id: String,
    /// The account key Desktop was on before the switch.
    pub from: String,
    /// The account key the switch was going to.
    pub to: String,
    /// The step the switch had reached.
    pub step: String,
    /// When the entry was written, ISO 8601.
    pub at: String,
}

impl Entry {
    /// An entry for the step `step`, stamped now.
    #[must_use]
    pub fn now(id: &str, from: &str, to: &str, step: &str) -> Self {
        Entry {
            id: id.to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
            step: step.to_owned(),
            at: now_iso(),
        }
    }
}

/// The journal file.
#[derive(Debug, Clone)]
pub struct Journal {
    path: PathBuf,
}

impl Journal {
    /// The journal kept in `path` (`...\accounts\switch-journal.json`).
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Journal {
            path: path.to_path_buf(),
        }
    }

    /// Whether a journal is there.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// The entry, if the file is there. A file that cannot be read still counts as a journal
    /// (see [`Journal::exists`]); it just has no entry.
    #[must_use]
    pub fn read(&self) -> Option<Entry> {
        let text = fs::read_to_string(&self.path).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Writes the entry, replacing the old one.
    ///
    /// # Errors
    /// Why the file could not be written.
    pub fn write(&self, entry: &Entry) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(entry).map_err(|e| e.to_string())?;
        write_atomic(&self.path, &bytes).map_err(|e| format!("the switch journal: {e}"))
    }

    /// Removes the journal; a missing one is fine.
    ///
    /// # Errors
    /// Why the file could not be removed.
    pub fn clear(&self) -> Result<(), String> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("the switch journal could not be removed: {e}")),
        }
    }
}

/// How a look at a leftover journal ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repair {
    /// No journal.
    Clean,
    /// The login of the `from` account was put back and the journal removed.
    Repaired {
        /// The account that was put back.
        from: String,
    },
    /// Claude runs, so nothing can be put back yet.
    Blocked {
        /// The account that will be put back.
        from: String,
    },
}

/// Repairs an interrupted switch: with Claude closed, restores the saved login of the journal's
/// `from` account and clears the journal; with Claude running, changes nothing.
///
/// # Errors
/// The journal names an account that is not known, or the saved login could not be put back (the
/// journal stays then).
pub fn repair(
    journal: &Journal,
    app: &dyn DesktopApp,
    vault: &dyn LoginVault,
    accounts: &[Account],
) -> Result<Repair, String> {
    if !journal.exists() {
        return Ok(Repair::Clean);
    }
    let entry = journal
        .read()
        .ok_or("the switch journal cannot be read; remove it by hand")?;
    if app.is_running() {
        return Ok(Repair::Blocked { from: entry.from });
    }
    let account = accounts
        .iter()
        .find(|a| a.key == entry.from)
        .ok_or_else(|| {
            format!(
                "the interrupted switch came from {}, which is gone",
                entry.from
            )
        })?;
    vault.restore(account)?;
    journal.clear()?;
    Ok(Repair::Repaired { from: entry.from })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use tempfile::TempDir;

    use super::{Entry, Journal, Repair, repair};
    use crate::fixtures::account;
    use crate::ports::{DesktopApp, LoginVault};
    use crate::types::Account;

    struct Fake {
        running: bool,
        restored: Mutex<Vec<String>>,
    }

    impl DesktopApp for Fake {
        fn is_running(&self) -> bool {
            self.running
        }
        fn turn_in_progress(&self) -> bool {
            false
        }
        fn close(&self) -> Result<(), String> {
            Ok(())
        }
        fn start(&self) -> Result<(), String> {
            Ok(())
        }
        fn config_touched_since(&self, _: i64) -> bool {
            true
        }
        fn version(&self) -> Option<String> {
            None
        }
    }

    impl LoginVault for Fake {
        fn save(&self, _: &Account) -> Result<(), String> {
            panic!("a repair must never save the login");
        }
        fn restore(&self, a: &Account) -> Result<(), String> {
            self.restored.lock().unwrap().push(a.key.clone());
            Ok(())
        }
        fn saved_at(&self, _: &Account) -> Option<String> {
            None
        }
    }

    fn journal_with_entry() -> (TempDir, Journal) {
        let dir = TempDir::new().unwrap();
        let journal = Journal::new(&dir.path().join("accounts").join("switch-journal.json"));
        journal
            .write(&Entry::now("SW-1", "a", "b", "restore_login"))
            .unwrap();
        (dir, journal)
    }

    #[test]
    fn write_update_read_and_clear() {
        let (_dir, journal) = journal_with_entry();
        assert!(journal.exists());
        journal
            .write(&Entry::now("SW-1", "a", "b", "start"))
            .unwrap();
        let entry = journal.read().unwrap();
        assert_eq!((entry.from.as_str(), entry.step.as_str()), ("a", "start"));
        journal.clear().unwrap();
        journal.clear().unwrap();
        assert!(!journal.exists());
    }

    #[test]
    fn no_journal_is_clean() {
        let dir = TempDir::new().unwrap();
        let journal = Journal::new(&dir.path().join("none.json"));
        let fake = Fake {
            running: false,
            restored: Mutex::default(),
        };
        let out = repair(&journal, &fake, &fake, &[account("a")]).unwrap();
        assert_eq!(out, Repair::Clean);
    }

    #[test]
    fn a_closed_claude_gets_the_from_login_back_and_the_journal_goes() {
        let (_dir, journal) = journal_with_entry();
        let fake = Fake {
            running: false,
            restored: Mutex::default(),
        };
        let out = repair(&journal, &fake, &fake, &[account("a"), account("b")]).unwrap();
        assert_eq!(
            out,
            Repair::Repaired {
                from: "a".to_owned()
            }
        );
        assert_eq!(*fake.restored.lock().unwrap(), ["a"]);
        assert!(!journal.exists());
    }

    #[test]
    fn a_running_claude_is_left_alone_and_the_journal_stays() {
        let (_dir, journal) = journal_with_entry();
        let fake = Fake {
            running: true,
            restored: Mutex::default(),
        };
        let out = repair(&journal, &fake, &fake, &[account("a")]).unwrap();
        assert_eq!(
            out,
            Repair::Blocked {
                from: "a".to_owned()
            }
        );
        assert!(fake.restored.lock().unwrap().is_empty());
        assert!(journal.exists());
    }

    #[test]
    fn an_unknown_from_account_keeps_the_journal() {
        let (_dir, journal) = journal_with_entry();
        let fake = Fake {
            running: false,
            restored: Mutex::default(),
        };
        assert!(repair(&journal, &fake, &fake, &[account("z")]).is_err());
        assert!(journal.exists());
    }
}
