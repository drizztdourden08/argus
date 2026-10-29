//! `manifest.json` of an entry: only the three fields Argus reads.

use crate::segment::join;
use argus_core::clock::parse_ms;
use argus_worktrees::Fs;
use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    size_bytes: Option<u64>,
    built_at: Option<String>,
    last_used_at: Option<String>,
}

/// What the manifest says; a missing or broken manifest says nothing.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Facts {
    pub size_bytes: Option<u64>,
    pub used_ms: Option<i64>,
}

pub fn read(fs: &dyn Fs, dir: &str) -> Facts {
    let Ok(text) = fs.read_text(&join(dir, "manifest.json")) else {
        return Facts::default();
    };
    let manifest: Manifest = serde_json::from_str(&text).unwrap_or_default();
    let used = manifest.last_used_at.or(manifest.built_at);
    Facts {
        size_bytes: manifest.size_bytes,
        used_ms: used.as_deref().and_then(parse_ms),
    }
}
