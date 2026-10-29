//! Brock's release record (`release.json`) read leniently. The folder the record sits in names
//! the project, channel and version; the record's own fields describe the rest.

use std::path::PathBuf;

use serde_json::Value;

use crate::fields::{list, number, opt_text, text};
use crate::{Replay, replay_of};

/// A package tarball of a release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// The package name.
    pub name: String,
    /// Its version.
    pub version: String,
    /// The tarball, a file name beside the record.
    pub tarball: String,
    /// Its size in bytes.
    pub size: u64,
    /// The sha256, when recorded.
    pub sha256: Option<String>,
    /// The sha512, when recorded.
    pub sha512: Option<String>,
}

/// A file an app release ships (an installer, a blockmap).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// The name GitHub would show.
    pub name: String,
    /// The file, beside the record.
    pub path: String,
    /// Its size in bytes.
    pub size: u64,
    /// `installer`, `blockmap` and so on.
    pub kind: String,
}

/// One release in the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// The project key.
    pub project: String,
    /// The channel folder.
    pub channel: String,
    /// The version folder.
    pub version: String,
    /// The folder of the release.
    pub dir: PathBuf,
    /// The git tag.
    pub tag: String,
    /// The commit.
    pub commit: String,
    /// The title.
    pub title: String,
    /// The notes file the record names, beside it.
    pub notes: Option<String>,
    /// When it was written.
    pub published_at: Option<String>,
    /// The package tarballs.
    pub packages: Vec<Package>,
    /// The app files.
    pub assets: Vec<Asset>,
    /// Where it stands on the way to GitHub.
    pub replay: Option<Replay>,
}

/// Where a release sits in the store.
pub struct Place<'a> {
    /// The project key.
    pub project: &'a str,
    /// The channel folder.
    pub channel: &'a str,
    /// The version folder.
    pub version: &'a str,
    /// The folder of the release.
    pub dir: PathBuf,
}

fn package(p: &Value) -> Package {
    Package {
        name: text(p, "name"),
        version: text(p, "version"),
        tarball: text(p, "tarball"),
        size: number(p, "size"),
        sha256: opt_text(p, "sha256"),
        sha512: opt_text(p, "sha512"),
    }
}

fn asset(a: &Value) -> Asset {
    let name = text(a, "name");
    Asset {
        path: opt_text(a, "path").unwrap_or_else(|| name.clone()),
        name,
        size: number(a, "size"),
        kind: text(a, "kind"),
    }
}

/// The release a record describes, `None` when it is not a release record.
#[must_use]
pub fn parse(place: Place<'_>, record: &Value) -> Option<Release> {
    if !record.is_object() || record["kind"].as_str().is_some_and(|k| k != "release") {
        return None;
    }
    Some(Release {
        project: place.project.to_owned(),
        channel: place.channel.to_owned(),
        version: place.version.to_owned(),
        tag: opt_text(record, "tag").unwrap_or_else(|| format!("v{}", place.version)),
        commit: text(record, "commit"),
        title: text(record, "title"),
        notes: opt_text(record, "notes"),
        published_at: opt_text(record, "publishedAt"),
        packages: list(record, "packages").iter().map(package).collect(),
        assets: list(record, "assets").iter().map(asset).collect(),
        replay: replay_of(record),
        dir: place.dir,
    })
}
