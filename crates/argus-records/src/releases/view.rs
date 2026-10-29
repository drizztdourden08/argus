//! The releases as the API shows them: per project, per channel, newest version first, with
//! the urls of every file on the feed (`/releases/<project>/<channel>/<version>/<file>`).

use std::fmt::Write as _;

use serde_json::{Value, json};

use super::record::{Asset, Package, Release};
use super::store::ReleaseStore;
use crate::replay_json;
use crate::safe_path::file_under;

fn encode(rel: &str) -> String {
    rel.bytes().fold(String::new(), |mut out, b| {
        if b.is_ascii_alphanumeric() || b"-_.~/".contains(&b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
        out
    })
}

/// The feed path of a file of a release, as a URL path on the server.
fn url(r: &Release, file: &str) -> String {
    let rel = format!("{}/{}/{}/{file}", r.project, r.channel, r.version);
    format!("/releases/{}", encode(&rel))
}

fn asset_json(r: &Release, a: &Asset) -> Value {
    json!({ "name": a.name, "size": a.size, "kind": a.kind, "url": url(r, &a.path) })
}

fn package_json(r: &Release, p: &Package) -> Value {
    json!({
        "name": p.name, "version": p.version, "size": p.size, "url": url(r, &p.tarball),
        "sha256": p.sha256, "sha512": p.sha512,
    })
}

fn notes_url(store: &ReleaseStore, r: &Release) -> Value {
    let name = r.notes.as_deref();
    let there = name.is_some_and(|n| {
        file_under(
            store.root(),
            &format!("{}/{}/{}/{n}", r.project, r.channel, r.version),
        )
        .is_some()
    });
    match name.filter(|_| there) {
        Some(n) => json!(url(r, n)),
        None => Value::Null,
    }
}

fn version_json(store: &ReleaseStore, r: &Release) -> Value {
    json!({
        "version": r.version, "tag": r.tag, "commit": r.commit, "title": r.title,
        "publishedAt": r.published_at, "notesUrl": notes_url(store, r),
        "assets": r.assets.iter().map(|a| asset_json(r, a)).collect::<Vec<_>>(),
        "packages": r.packages.iter().map(|p| package_json(r, p)).collect::<Vec<_>>(),
        "replay": replay_json(r.replay.as_ref()),
    })
}

fn channel_json(store: &ReleaseStore, project: &str, channel: &str) -> Value {
    let versions: Vec<Value> = store
        .channel(project, channel)
        .iter()
        .map(|r| version_json(store, r))
        .collect();
    json!({ "channel": channel, "latest": store.latest(project, channel), "versions": versions })
}

fn names(store: &ReleaseStore, project: Option<&str>) -> Vec<(String, String)> {
    let mut seen: Vec<(String, String)> = Vec::new();
    for r in store
        .releases()
        .iter()
        .filter(|r| project.is_none_or(|p| r.project == p))
    {
        let key = (r.project.clone(), r.channel.clone());
        if !seen.contains(&key) {
            seen.push(key);
        }
    }
    seen
}

impl ReleaseStore {
    /// The channels of one project.
    #[must_use]
    pub fn channels_json(&self, project: &str) -> Value {
        let channels: Vec<Value> = names(self, Some(project))
            .iter()
            .map(|(p, c)| channel_json(self, p, c))
            .collect();
        json!(channels)
    }

    /// Every project with its channels.
    #[must_use]
    pub fn projects_json(&self) -> Value {
        let mut projects: Vec<String> = names(self, None).into_iter().map(|(p, _)| p).collect();
        projects.dedup();
        let rows: Vec<Value> = projects
            .iter()
            .map(|p| json!({ "project": p, "channels": self.channels_json(p) }))
            .collect();
        json!(rows)
    }
}
