//! What the update feed serves: the files of the store, and the channel's `latest.yml`.
//!
//! electron-builder writes `latest.yml` last inside each version folder. The channel level
//! `<channel>/latest.yml` (and `latest-mac.yml`, `latest-linux.yml`) is therefore made here:
//! the newest version that has the file, a release before a prerelease, with every `url:` and
//! `path:` pointing into its version folder. A channel level file on disk wins over that.

use std::path::PathBuf;

use super::semver::is_prerelease;
use super::store::ReleaseStore;
use super::yml::{rewrite_urls, version_of};
use crate::safe_path::{file_under, segments};

/// The manifests an updater asks for at the channel level.
pub const MANIFESTS: [&str; 3] = ["latest.yml", "latest-mac.yml", "latest-linux.yml"];

/// What a feed path is answered with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Served {
    /// A file of the store.
    File(PathBuf),
    /// A manifest made from a version's.
    Text(String),
}

impl ReleaseStore {
    /// The newest version of a channel that has the manifest, and its path: a release before a
    /// prerelease (a channel of prereleases only falls back to the newest of them).
    fn newest_with(&self, project: &str, channel: &str, name: &str) -> Option<(String, PathBuf)> {
        let with_file: Vec<(String, PathBuf)> = self
            .channel(project, channel)
            .into_iter()
            .filter_map(|r| {
                let rel = format!("{project}/{channel}/{}/{name}", r.version);
                Some((r.version, file_under(self.root(), &rel)?))
            })
            .collect();
        let release = with_file.iter().find(|(v, _)| !is_prerelease(v));
        release.or(with_file.first()).cloned()
    }

    fn channel_manifest(&self, project: &str, channel: &str, name: &str) -> Option<PathBuf> {
        file_under(self.root(), &format!("{project}/{channel}/{name}"))
    }

    /// The version `latest.yml` of a channel points at: the channel level file's, else the
    /// newest version's.
    #[must_use]
    pub fn latest(&self, project: &str, channel: &str) -> Option<String> {
        let name = MANIFESTS[0];
        if let Some(file) = self.channel_manifest(project, channel, name) {
            return version_of(&std::fs::read_to_string(file).ok()?);
        }
        self.newest_with(project, channel, name).map(|(v, _)| v)
    }

    fn made_manifest(&self, project: &str, channel: &str, name: &str) -> Option<Served> {
        let (version, file) = self.newest_with(project, channel, name)?;
        let text = std::fs::read_to_string(file).ok()?;
        Some(Served::Text(rewrite_urls(&text, &version)))
    }

    /// The answer to `<project>/<channel>/<path>`; `None` is a 404 (nothing there, a folder, or
    /// a path that is not plain names inside the store).
    #[must_use]
    pub fn serve(&self, rel: &str) -> Option<Served> {
        let parts = segments(rel)?;
        if let [project, channel, name] = parts.as_slice() {
            if MANIFESTS.contains(name) {
                let on_disk = self.channel_manifest(project, channel, name);
                return on_disk
                    .map(Served::File)
                    .or_else(|| self.made_manifest(project, channel, name));
            }
        }
        file_under(self.root(), rel).map(Served::File)
    }
}
