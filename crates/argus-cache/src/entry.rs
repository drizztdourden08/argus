//! One cache entry: a `<buildKey>` folder of a package version or an Electron version.

use serde::Serialize;

/// What an entry holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Package,
    Electron,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::Package, Kind::Electron];

    /// The top folder of the kind in the cache.
    pub fn dir(self) -> &'static str {
        match self {
            Kind::Package => "package",
            Kind::Electron => "electron",
        }
    }
}

/// A `<buildKey>` folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub kind: Kind,
    /// The package name, scope slash restored (`@drizztdourden08/brock-core`); `electron` for Electron.
    pub name: String,
    pub version: String,
    pub build_key: String,
    /// The folder, forward slashes.
    pub path: String,
    pub size_bytes: u64,
    /// `lastUsedAt` of the manifest, else `builtAt`, else the folder's own time.
    pub last_used_ms: i64,
}
