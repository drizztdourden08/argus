//! K3, E2: a worktree of a project keeps the build cache entries its lockfile resolves, on a real
//! throwaway repository with the real `git`.
// test-tag: e2e

use std::fs;
use std::path::Path;

use crate::worktrees_rig::{Rig, Setup};
use argus_core::clock::{iso_from_ms, now_ms};
use serde_json::{Value, json};

const DAY: i64 = 86_400_000;
const LOCK: &str = "lockfileVersion: '9.0'\n\npackages:\n\n  '@drizztdourden08/brock-core@0.115.3':\n    resolution: {integrity: sha512-x}\n";

fn entry(root: &Path, version: &str, key: &str, size: u64, used_days_ago: i64) {
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

fn row<'a>(body: &'a Value, version: &str) -> &'a Value {
    body["rows"]
        .as_array()
        .and_then(|rows| rows.iter().find(|r| r["version"] == version))
        .expect("the row")
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn a_worktree_of_a_project_keeps_what_its_lockfile_resolves() {
    let cache_dir = tempfile::tempdir().expect("temp dir");
    let r = Rig::with(Setup {
        cache_root: Some(cache_dir.path().to_path_buf()),
        ..Setup::default()
    })
    .await;
    entry(cache_dir.path(), "0.115.3", "node-aa", 500, 100);
    r.scanned().await;
    fs::write(r.path("fresh").join("pnpm-lock.yaml"), LOCK).expect("worktree lockfile");
    fs::write(r.path("plain").join("pnpm-lock.yaml"), LOCK).expect("leftover lockfile");
    let body = r.t.get("/api/cache").await.json();
    let users = &row(&body, "0.115.3")["users"];
    assert_eq!(
        users,
        &json!([{ "label": "alpha / fresh", "kind": "worktree" }])
    );
    let out = r.cli("/api/cache/prune", &json!({})).await;
    assert_eq!(
        out.json()["deleted"].as_array().map(Vec::len),
        Some(0),
        "{}",
        out.text()
    );
    assert!(
        cache_dir
            .path()
            .join("package/@drizztdourden08+brock-core/0.115.3/node-aa")
            .exists()
    );
}
