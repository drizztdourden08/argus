//! A server whose worktree scanner looks at a throwaway repository, with timers on and no real
//! process list, and the calls the worktree tests make over and over.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use argus_core::clock::iso_from_ms;
use argus_core::clock::now_ms;
use argus_server::adapters::link::WakeLink;
use argus_testkit::{Reply, Request, TestOptions, TestServer, alpha_board, make_home, write_board};
use argus_worktrees::{GitExec, ProcInfo, ProcessLister, WakeEvent, clean_path, worktree_id};
use serde_json::{Value, json};
use tempfile::TempDir;

use super::fixture::Fixture;

#[path = "../support/wait.rs"]
mod wait;

pub use wait::until;

/// A machine with no process on it: a removal never finds its folder in use.
pub struct NoProcs;

impl ProcessLister for NoProcs {
    fn list(&self) -> Result<Vec<ProcInfo>, String> {
        Ok(Vec::new())
    }
}

/// The wakes the scanner asked for.
#[derive(Default)]
pub struct Wakes {
    pub events: Mutex<Vec<WakeEvent>>,
}

impl WakeLink for Wakes {
    fn worktree_wakes(&self, events: &[WakeEvent]) -> Result<(), String> {
        self.events.lock().unwrap().extend_from_slice(events);
        Ok(())
    }
}

#[derive(Default)]
pub struct Setup {
    pub shadow: bool,
    pub enabled_off: bool,
    pub git: Option<Arc<dyn GitExec>>,
}

pub struct Rig {
    pub t: TestServer,
    pub fx: Fixture,
    pub wakes: Arc<Wakes>,
    _home: TempDir,
}

/// The id of a worktree folder.
pub fn id_of(path: &std::path::Path) -> String {
    worktree_id(&path.display().to_string(), cfg!(windows))
}

pub fn rule(fx: &Fixture) -> Value {
    json!({ "id": "W-1", "path": clean_path(&fx.wts.display().to_string()), "kind": "folder", "include": [], "exclude": [], "thread": "ui", "note": "the ui thread builds here", "staleDays": null, "addedBy": "ui", "addedAt": iso_from_ms(now_ms()) })
}

impl Rig {
    pub async fn start() -> Rig {
        Rig::with(Setup::default()).await
    }

    pub async fn with(setup: Setup) -> Rig {
        let fx = Fixture::new();
        let home = tempfile::tempdir().expect("a coord home");
        make_home(home.path());
        let mut board = alpha_board(home.path());
        board["worktreeRules"] = json!([rule(&fx)]);
        board["counters"]["worktreeRule"] = json!(1);
        write_board(home.path(), "alpha", &board);
        let Setup {
            shadow,
            enabled_off,
            git,
        } = setup;
        let t = TestServer::start(TestOptions {
            shadow,
            coord_home: Some(home.path().to_path_buf()),
            config: json!({ "worktrees": { "startDelayMs": 300, "liveMs": 1000 } }),
            tweak: Some(Arc::new(move |opts| {
                opts.worktrees.timers = true;
                opts.worktrees.enabled = !enabled_off;
                opts.worktrees.procs = Some(Arc::new(NoProcs));
                opts.worktrees.git.clone_from(&git);
            })),
            ..TestOptions::default()
        })
        .await;
        let wakes = Arc::new(Wakes::default());
        t.svc.wake_link.set(wakes.clone());
        Rig {
            t,
            fx,
            wakes,
            _home: home,
        }
    }

    /// The docs of the Worktrees tab by folder name.
    pub async fn docs(&self) -> serde_json::Map<String, Value> {
        let list = self.t.get("/api/p/alpha/db/worktrees").await.json();
        let rows = list["docs"].as_array().cloned().unwrap_or_default();
        rows.iter()
            .filter_map(|d| Some((d["data"]["name"].as_str()?.to_owned(), d["data"].clone())))
            .collect()
    }

    pub async fn meta(&self) -> Value {
        self.t.get("/api/p/alpha/db/meta/worktrees").await.json()["data"].clone()
    }

    /// Waits for the first scan.
    pub async fn scanned(&self) -> Value {
        until("the first scan", || async {
            let meta = self.meta().await;
            meta["scannedAt"].is_string().then_some(meta)
        })
        .await
    }

    /// A call as the owner's page makes it.
    pub async fn page(&self, path: &str, body: &Value) -> Reply {
        self.t
            .send(self.t.page(Request::new("POST", path).json(body)))
            .await
    }

    /// A call as a thread of the CLI makes it.
    pub async fn cli(&self, path: &str, body: &Value) -> Reply {
        self.t.write("POST", path, body).await
    }

    pub async fn activity(&self, kind: &str) -> Vec<String> {
        let rows = self
            .t
            .get(&format!("/api/activity?kind={kind}"))
            .await
            .json();
        let list = rows["activity"].as_array().cloned().unwrap_or_default();
        list.iter()
            .filter_map(|a| a["text"].as_str().map(str::to_owned))
            .collect()
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.fx.path[name].clone()
    }
}
