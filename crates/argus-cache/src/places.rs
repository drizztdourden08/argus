//! Who uses the cache: the known projects and git worktrees, through the lockfiles they hold.

use crate::lockfile::resolved;
use crate::segment::join;
use argus_worktrees::Fs;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;

const LOCKFILES: [&str; 2] = ["pnpm-lock.yaml", "node_modules/.pnpm/lock.yaml"];

/// What a place is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaceKind {
    Project,
    Worktree,
}

/// A project root or a git worktree that Argus knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub label: String,
    pub kind: PlaceKind,
    pub root: String,
}

/// A place that uses an entry, as the page shows it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct User {
    pub label: String,
    pub kind: PlaceKind,
}

/// Which places resolve which cached name at which exact version.
#[derive(Debug, Default)]
pub struct Usage {
    users: BTreeMap<(String, String), Vec<User>>,
    /// Lockfiles that exist and could not be read: no entry may be judged unused while there are any.
    pub problems: Vec<String>,
}

impl Usage {
    /// The places that resolve `name` at exactly `version`.
    pub fn users_of(&self, name: &str, version: &str) -> &[User] {
        self.users
            .get(&(name.to_owned(), version.to_owned()))
            .map_or(&[], Vec::as_slice)
    }
}

/// The pairs of `names` that a place's lockfiles resolve, and the lockfiles that could not be read.
fn read_place(
    fs: &dyn Fs,
    place: &Place,
    names: &BTreeSet<String>,
) -> (BTreeSet<(String, String)>, Vec<String>) {
    let (mut pairs, mut problems) = (BTreeSet::new(), Vec::new());
    for lockfile in LOCKFILES {
        let path = join(&place.root, lockfile);
        match fs.read_text(&path) {
            Ok(text) => pairs.extend(resolved(&text, names)),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => problems.push(format!("{path}: {e}")),
        }
    }
    (pairs, problems)
}

/// Reads the lockfiles of every place for the cached `names`.
pub fn read_usage(fs: &dyn Fs, places: &[Place], names: &BTreeSet<String>) -> Usage {
    let mut usage = Usage::default();
    for place in places {
        let (pairs, problems) = read_place(fs, place, names);
        usage.problems.extend(problems);
        for pair in pairs {
            usage.users.entry(pair).or_default().push(User {
                label: place.label.clone(),
                kind: place.kind,
            });
        }
    }
    for users in usage.users.values_mut() {
        users.sort();
    }
    usage
}
