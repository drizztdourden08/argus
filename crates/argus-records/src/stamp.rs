//! A file's time and size, and a cache of what was parsed from a file under that stamp, so a
//! rescan of a big record folder only reads the files that changed.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Modification time and size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stamp {
    mtime: Option<SystemTime>,
    len: u64,
}

/// The stamp of a file, `None` when it cannot be looked at.
pub(crate) fn stamp_of(path: &Path) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some(Stamp {
        mtime: meta.modified().ok(),
        len: meta.len(),
    })
}

/// What was parsed from each file, valid while the file keeps its stamp. A file that did not
/// parse is not kept, so it is read again on the next scan (it may be half written).
pub(crate) struct StampCache<T> {
    entries: HashMap<PathBuf, (Stamp, T)>,
    touched: HashSet<PathBuf>,
}

impl<T> Default for StampCache<T> {
    fn default() -> Self {
        StampCache {
            entries: HashMap::new(),
            touched: HashSet::new(),
        }
    }
}

impl<T: Clone> StampCache<T> {
    /// The parse of a file: the kept one while the stamp is the same, else a new one.
    pub(crate) fn get(&mut self, path: &Path, parse: impl FnOnce(&Path) -> Option<T>) -> Option<T> {
        let stamp = stamp_of(path)?;
        self.touched.insert(path.to_path_buf());
        if let Some((have, value)) = self.entries.get(path) {
            if *have == stamp {
                return Some(value.clone());
            }
        }
        let value = parse(path)?;
        self.entries
            .insert(path.to_path_buf(), (stamp, value.clone()));
        Some(value)
    }

    /// Forgets the files that were not asked for since the last sweep.
    pub(crate) fn sweep(&mut self) {
        let touched = std::mem::take(&mut self.touched);
        self.entries.retain(|path, _| touched.contains(path));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn a_file_is_parsed_again_only_when_it_changed() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("a.json");
        std::fs::write(&file, "1").expect("write");
        let parses = Cell::new(0);
        let read = |p: &Path| {
            parses.set(parses.get() + 1);
            std::fs::read_to_string(p).ok()
        };
        let mut cache = StampCache::default();
        assert_eq!(cache.get(&file, read).as_deref(), Some("1"));
        assert_eq!(cache.get(&file, read).as_deref(), Some("1"));
        assert_eq!(parses.get(), 1);
        std::fs::write(&file, "22").expect("write");
        assert_eq!(cache.get(&file, read).as_deref(), Some("22"));
        assert_eq!(parses.get(), 2);
    }

    #[test]
    fn a_failed_parse_is_not_kept_and_a_swept_file_is_forgotten() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("a.json");
        std::fs::write(&file, "x").expect("write");
        let mut cache: StampCache<String> = StampCache::default();
        assert_eq!(cache.get(&file, |_| None), None);
        assert_eq!(cache.get(&file, |_| Some("v".into())).as_deref(), Some("v"));
        cache.sweep();
        cache.sweep();
        assert!(cache.entries.is_empty());
    }
}
