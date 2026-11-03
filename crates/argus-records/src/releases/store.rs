//! The release store: `<root>/<project>/<channel>/<version>/release.json` with its files
//! beside it. The index is rebuilt by [`ReleaseStore::rescan`]; an entry without a readable
//! `release.json`, or under a temp name, is not in it (it is half written).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use argus_core::ids::is_key;
use argus_core::remote::ci_root_of;
use serde_json::Value;

use super::record::{Place, Release, parse};
use super::semver::compare;
use super::shipped::{Shipped, of_range};
use crate::Replay;
use crate::safe_path::{children, segment_ok};
use crate::stamp::StampCache;

const RECORD: &str = "release.json";

/// The releases on disk, indexed.
pub struct ReleaseStore {
    root: PathBuf,
    index: RwLock<Arc<Vec<Release>>>,
    cache: Mutex<StampCache<Value>>,
    /// What git said of a (project, previous commit, commit) triple, kept until the process ends.
    shipped: Mutex<HashMap<(String, String, String), Shipped>>,
}

/// The names of the folders in a folder that can be part of the store.
fn folders(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut found = children(dir, std::fs::FileType::is_dir);
    found.retain(|(name, _)| segment_ok(name));
    found
}

fn read_record(path: &Path) -> Option<Value> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

impl ReleaseStore {
    /// A store over a folder; call [`Self::rescan`] to fill the index.
    #[must_use]
    pub fn new(root: &Path) -> Self {
        ReleaseStore {
            root: root.to_path_buf(),
            index: RwLock::default(),
            cache: Mutex::default(),
            shipped: Mutex::default(),
        }
    }

    /// The items and merged pull requests a release ships: the commits since the previous version
    /// of its channel, from the project's `ciRoots` checkout. Empty while git cannot answer (the
    /// next call asks again); a good answer is kept by commit pair.
    #[must_use]
    pub fn shipped(&self, release: &Release, previous: Option<&Release>) -> Shipped {
        let Some(repo) = self
            .root
            .parent()
            .and_then(|home| ci_root_of(home, &release.project))
        else {
            return Shipped::default();
        };
        let before = previous
            .map(|p| p.commit.as_str())
            .filter(|c| !c.is_empty());
        let key = (
            release.project.clone(),
            before.unwrap_or_default().to_owned(),
            release.commit.clone(),
        );
        let kept = self
            .shipped
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key)
            .cloned();
        if let Some(found) = kept {
            return found;
        }
        let asked = of_range(Path::new(&repo), before, &release.commit);
        if let Some(found) = &asked {
            let mut all = self.shipped.lock().unwrap_or_else(PoisonError::into_inner);
            all.insert(key, found.clone());
        }
        asked.unwrap_or_default()
    }

    /// The folder of the store.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The index now: by project, then channel, newest version first.
    #[must_use]
    pub fn releases(&self) -> Arc<Vec<Release>> {
        Arc::clone(&self.index.read().unwrap_or_else(PoisonError::into_inner))
    }

    /// Reads the store again and replaces the index.
    pub fn rescan(&self) {
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let mut found = Vec::new();
        for (project, dir) in folders(&self.root).into_iter().filter(|(p, _)| is_key(p)) {
            for (channel, dir) in folders(&dir) {
                for (version, dir) in folders(&dir) {
                    let place = Place {
                        project: &project,
                        channel: &channel,
                        version: &version,
                        dir: dir.clone(),
                    };
                    let record = cache.get(&dir.join(RECORD), read_record);
                    found.extend(record.and_then(|r| parse(place, &r)));
                }
            }
        }
        cache.sweep();
        found.sort_by(|a, b| {
            (&a.project, &a.channel)
                .cmp(&(&b.project, &b.channel))
                .then_with(|| compare(&b.version, &a.version))
        });
        *self.index.write().unwrap_or_else(PoisonError::into_inner) = Arc::new(found);
    }

    /// The releases of one channel, newest first.
    #[must_use]
    pub fn channel(&self, project: &str, channel: &str) -> Vec<Release> {
        self.releases()
            .iter()
            .filter(|r| r.project == project && r.channel == channel)
            .cloned()
            .collect()
    }

    /// How many releases wait for the go.
    #[must_use]
    pub fn pending(&self) -> usize {
        let pending = |r: &&Release| r.replay.as_ref().is_some_and(Replay::is_pending);
        self.releases().iter().filter(pending).count()
    }
}
