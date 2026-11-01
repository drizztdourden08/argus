//! The build cache of Brock and Tessera over HTTP, on throwaway cache folders, projects and worktrees: the listing with
//! its users, the totals, the prune on demand and the one activity line it leaves, the grace period of the config,
//! shadow mode, and the wait for the first worktree scan. The prune rule itself is tested in `argus-cache`. K2, K3: the listing shape and the prune; the worktree and lockfile case is in `cache_worktree.rs`.
// test-tag: standard

use std::fs;
use std::path::Path;
use std::sync::Arc;

use argus_core::clock::{iso_from_ms, now_ms};
use argus_server::CacheOptions;
use argus_testkit::{TestOptions, TestServer, make_home};
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

fn project(dir: &Path, name: &str) -> String {
    let root = dir.join(name);
    fs::create_dir_all(&root).expect("project folder");
    fs::write(root.join("pnpm-lock.yaml"), LOCK).expect("lockfile");
    root.display().to_string()
}

struct Cache {
    t: TestServer,
    root: std::path::PathBuf,
    _dirs: tempfile::TempDir,
}

async fn cache_with(shadow: bool, config: Value) -> Cache {
    let dirs = tempfile::tempdir().expect("temp dir");
    let root = dirs.path().join("cache");
    fs::create_dir_all(&root).expect("cache root");
    entry(&root, "0.115.3", "node-aa", 500, 100);
    entry(&root, "0.100.0", "node-aa", 300, 40);
    entry(&root, "0.110.0", "node-aa", 200, 2);
    let mut config = config;
    config["projectRoots"] = json!({ "brock": project(dirs.path(), "brock") });
    let served = root.clone();
    let t = TestServer::start(TestOptions {
        shadow,
        config,
        tweak: Some(Arc::new(move |opts| {
            opts.worktrees.enabled = false;
            opts.cache = CacheOptions {
                enabled: true,
                timers: false,
                root: Some(served.clone()),
            };
        })),
        ..TestOptions::default()
    })
    .await;
    Cache {
        t,
        root,
        _dirs: dirs,
    }
}

async fn cache() -> Cache {
    cache_with(false, json!({})).await
}

fn row<'a>(body: &'a Value, version: &str) -> &'a Value {
    body["rows"]
        .as_array()
        .and_then(|rows| rows.iter().find(|r| r["version"] == version))
        .expect("the row")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_listing_has_a_row_per_version_with_its_users_and_the_totals() {
    let c = cache().await;
    let body = c.t.get("/api/cache").await.json();
    assert_eq!(body["exists"], true);
    assert_eq!(body["graceDays"], 14);
    assert_eq!(body["lastPrune"], Value::Null);
    assert_eq!(body["shadow"], false);
    let used = row(&body, "0.115.3");
    assert_eq!(used["inUse"], true);
    assert_eq!(used["userCount"], 1);
    assert_eq!(
        used["users"][0],
        json!({ "label": "brock", "kind": "project" })
    );
    assert_eq!(used["kind"], "package");
    assert_eq!(used["name"], "@drizztdourden08/brock-core");
    assert_eq!(used["buildKeys"], 1);
    assert_eq!(used["sizeBytes"], 500);
    assert_eq!(row(&body, "0.100.0")["prunable"], 1);
    assert_eq!(row(&body, "0.110.0")["prunable"], 0);
    assert!(row(&body, "0.110.0")["deleteAfter"].is_string());
    assert_eq!(body["totals"]["versions"], 3);
    assert_eq!(body["totals"]["prunableBytes"], 300);
    assert_eq!(body["totals"]["sizeBytes"], 1000);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_prune_deletes_what_is_unused_past_the_grace_and_leaves_one_activity_line() {
    let c = cache().await;
    let done = c.t.write("POST", "/api/cache/prune", &json!({})).await;
    assert_eq!(done.status, 200, "{}", done.text());
    let out = done.json();
    assert_eq!(out["deleted"].as_array().map(Vec::len), Some(1));
    assert_eq!(out["deleted"][0]["version"], "0.100.0");
    assert_eq!(out["freedBytes"], 300);
    let name = c.root.join("package/@drizztdourden08+brock-core");
    assert!(!name.join("0.100.0").exists());
    assert!(name.join("0.115.3/node-aa").exists());
    assert!(name.join("0.110.0/node-aa").exists());
    let lines = c.t.get("/api/activity?kind=build-cache").await.json();
    let texts: Vec<&str> = lines["activity"]
        .as_array()
        .expect("lines")
        .iter()
        .filter_map(|l| l["text"].as_str())
        .collect();
    assert_eq!(
        texts,
        ["build cache pruned: 1 entry, 0 temp folders, 300 B freed"]
    );
    let body = c.t.get("/api/cache").await.json();
    assert_eq!(body["lastPrune"]["freedBytes"], 300);
    assert_eq!(body["totals"]["versions"], 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_grace_period_comes_from_the_config() {
    let c = cache_with(false, json!({ "buildCache": { "graceDays": 60 } })).await;
    let body = c.t.get("/api/cache").await.json();
    assert_eq!(body["graceDays"], 60);
    let out =
        c.t.write("POST", "/api/cache/prune", &json!({}))
            .await
            .json();
    assert_eq!(out["deleted"].as_array().map(Vec::len), Some(0));
    assert!(
        c.root
            .join("package/@drizztdourden08+brock-core/0.100.0")
            .exists()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn shadow_mode_lists_and_deletes_nothing() {
    let c = cache_with(true, json!({})).await;
    assert_eq!(c.t.get("/api/cache").await.json()["shadow"], true);
    let refused = c.t.write("POST", "/api/cache/prune", &json!({})).await;
    assert_eq!(refused.status, 403);
    assert_eq!(refused.json()["error"]["code"], "read_only");
    assert!(
        c.root
            .join("package/@drizztdourden08+brock-core/0.100.0")
            .exists()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_prune_waits_for_the_first_worktree_scan() {
    let dirs = tempfile::tempdir().expect("temp dir");
    let root = dirs.path().join("cache");
    fs::create_dir_all(&root).expect("cache root");
    entry(&root, "0.100.0", "node-aa", 300, 40);
    let home = tempfile::tempdir().expect("coord home");
    make_home(home.path());
    let served = root.clone();
    let t = TestServer::start(TestOptions {
        seed: true,
        coord_home: Some(home.path().to_path_buf()),
        tweak: Some(Arc::new(move |opts| {
            opts.cache = CacheOptions {
                enabled: true,
                timers: false,
                root: Some(served.clone()),
            };
        })),
        ..TestOptions::default()
    })
    .await;
    let early = t.write("POST", "/api/cache/prune", &json!({})).await;
    assert_eq!(early.status, 409, "{}", early.text());
    assert!(
        root.join("package/@drizztdourden08+brock-core/0.100.0")
            .exists()
    );
    let service = t.svc.worktrees.clone().expect("the worktree service");
    tokio::task::spawn_blocking(move || service.scan_all())
        .await
        .expect("scan");
    let late = t.write("POST", "/api/cache/prune", &json!({})).await;
    assert_eq!(late.status, 200, "{}", late.text());
    assert!(
        !root
            .join("package/@drizztdourden08+brock-core/0.100.0")
            .exists()
    );
}
