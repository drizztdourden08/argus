//! The local package registry of the Overlord's `localpkg`: `<home>/index.json` lists the
//! builds packed from local checkouts (the tarballs sit in `<home>/tgz`), and
//! `localpkg status --json` lists the checkouts that use them. Read only.
// test-tag: critical

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::fields::{number, opt_text, text};
use crate::{replay_json, replay_of};

/// The index of local builds.
pub struct Registry {
    home: PathBuf,
}

/// One entry of the index, named by its key when the entry has no `name`.
fn named<'a>(name: &str, entry: &'a Value) -> (String, &'a Value) {
    (
        opt_text(entry, "name").unwrap_or_else(|| name.to_owned()),
        entry,
    )
}

fn entries(index: &Value) -> Vec<(String, &Value)> {
    match &index["packages"] {
        Value::Object(map) => map.iter().map(|(k, v)| named(k, v)).collect(),
        Value::Array(items) => items.iter().map(|v| named("", v)).collect(),
        _ => Vec::new(),
    }
}

fn first_text(entry: &Value, keys: [&str; 2]) -> Option<String> {
    keys.iter().find_map(|k| opt_text(entry, k))
}

impl Registry {
    /// The registry in a `localpkg` home.
    #[must_use]
    pub fn new(home: &Path) -> Self {
        Registry {
            home: home.to_path_buf(),
        }
    }

    fn size_of(&self, entry: &Value, tarball: &str) -> u64 {
        let recorded = number(entry, "size");
        if recorded > 0 || tarball.is_empty() {
            return recorded;
        }
        let path = self.home.join(tarball);
        std::fs::metadata(path).map_or(0, |m| m.len())
    }

    fn build(&self, name: &str, entry: &Value) -> Value {
        let tarball = text(entry, "tarball");
        json!({
            "name": name, "version": text(entry, "version"),
            "commit": first_text(entry, ["commit", "head"]),
            "checkout": first_text(entry, ["checkout", "project"]),
            "packedAt": opt_text(entry, "packedAt"), "dirty": entry["dirty"] == true,
            "size": self.size_of(entry, &tarball), "tarball": tarball,
            "replay": replay_json(replay_of(entry).as_ref()),
        })
    }

    fn index(&self) -> Value {
        let read = std::fs::read(self.home.join("index.json"));
        read.ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or(Value::Null)
    }

    /// The local builds, by name.
    #[must_use]
    pub fn builds(&self) -> Vec<Value> {
        let index = self.index();
        let mut builds: Vec<(String, Value)> = entries(&index)
            .into_iter()
            .map(|(name, entry)| (name.clone(), self.build(&name, entry)))
            .collect();
        builds.sort_by(|a, b| a.0.cmp(&b.0));
        builds.into_iter().map(|(_, b)| b).collect()
    }

    /// How many builds wait for the go; 0 when the index carries no `replay`.
    #[must_use]
    pub fn pending(&self) -> usize {
        let index = self.index();
        entries(&index)
            .iter()
            .filter(|(_, e)| replay_of(e).is_some_and(|r| r.is_pending()))
            .count()
    }
}

/// The consumers that are on, from the text `localpkg status --json` printed: the checkouts
/// (`root`, or `path`) with the packages they use when the output says so.
#[must_use]
pub fn consumers_from(status: &str) -> Option<Vec<Value>> {
    let parsed: Value = serde_json::from_str(status).ok()?;
    let items = parsed["consumers"].as_array()?;
    Some(
        items
            .iter()
            .map(|c| {
                json!({
                    "path": first_text(c, ["root", "path"]), "since": opt_text(c, "since"),
                    "hook": c["hook"] != false,
                    "packages": c["packages"].as_array().cloned().unwrap_or_default(),
                    "on": true,
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_are_read_from_the_index_with_either_field_names() {
        let home = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(home.path().join("tgz")).expect("folder");
        std::fs::write(home.path().join("tgz/b.tgz"), "12345").expect("tarball");
        let index = json!({ "packages": {
            "@d/b": { "version": "1.0.0", "tarball": "tgz/b.tgz", "project": "X:/brock", "head": "abc", "dirty": true, "packedAt": "t" },
            "@d/a": { "version": "2.0.0", "tarball": "tgz/missing.tgz", "checkout": "X:/a", "commit": "def", "size": 9,
                      "replay": { "state": "pending" } },
        }});
        std::fs::write(home.path().join("index.json"), index.to_string()).expect("index");
        let registry = Registry::new(home.path());
        let builds = registry.builds();
        assert_eq!(builds[0]["name"], "@d/a");
        assert_eq!(
            (builds[0]["commit"].as_str(), builds[0]["size"].as_u64()),
            (Some("def"), Some(9))
        );
        assert_eq!(builds[1]["checkout"], "X:/brock");
        assert_eq!(
            (builds[1]["commit"].as_str(), builds[1]["size"].as_u64()),
            (Some("abc"), Some(5))
        );
        assert_eq!(builds[1]["dirty"], true);
        assert_eq!(registry.pending(), 1);
    }

    #[test]
    fn a_missing_index_is_an_empty_registry() {
        let home = tempfile::tempdir().expect("temp dir");
        let registry = Registry::new(home.path());
        assert_eq!(registry.builds().len(), 0);
        assert_eq!(registry.pending(), 0);
    }

    #[test]
    fn consumers_come_from_the_status_output_and_garbage_is_none() {
        let out = r#"{ "home": "x", "builds": [], "consumers": [ { "root": "X:/rotp", "since": "t", "hook": true } ] }"#;
        let list = consumers_from(out).expect("consumers");
        assert_eq!(list[0]["path"], "X:/rotp");
        assert_eq!(
            (list[0]["on"].clone(), list[0]["packages"].clone()),
            (json!(true), json!([]))
        );
        assert_eq!(consumers_from("not json"), None);
        assert_eq!(consumers_from("{}"), None);
    }
}
