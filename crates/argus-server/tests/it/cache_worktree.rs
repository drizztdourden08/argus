//! K3, E2: a worktree of a project keeps the build cache entries its lockfile resolves, on a real
//! throwaway repository with the real `git`.
// test-tag: e2e

use std::fs;

use crate::cache_rows::{LOCK, entry, row};
use crate::worktrees_rig::{Rig, Setup};
use serde_json::json;

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
