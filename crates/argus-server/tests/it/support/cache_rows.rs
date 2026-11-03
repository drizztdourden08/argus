//! Shared by the cache specs: a build cache entry on disk, the lockfile that resolves it, and a row of the listing.

use std::fs;
use std::path::Path;

use argus_core::clock::{iso_from_ms, now_ms};
use serde_json::{Value, json};

pub const LOCK: &str = "lockfileVersion: '9.0'\n\npackages:\n\n  '@drizztdourden08/brock-core@0.115.3':\n    resolution: {integrity: sha512-x}\n";

const DAY: i64 = 86_400_000;

pub fn entry(root: &Path, version: &str, key: &str, size: u64, used_days_ago: i64) {
    let dir = root
        .join("package/@drizztdourden08+brock-core")
        .join(version)
        .join(key);
    fs::create_dir_all(dir.join("files")).expect("entry folders");
    let used = iso_from_ms(now_ms() - used_days_ago * DAY);
    let manifest = json!({ "schema": 1, "sizeBytes": size, "builtAt": used, "lastUsedAt": used });
    fs::write(dir.join("manifest.json"), manifest.to_string()).expect("manifest");
    fs::write(dir.join("files/index.js"), "x").expect("file");
}

pub fn row<'a>(body: &'a Value, version: &str) -> &'a Value {
    body["rows"]
        .as_array()
        .and_then(|rows| rows.iter().find(|r| r["version"] == version))
        .expect("the row")
}
