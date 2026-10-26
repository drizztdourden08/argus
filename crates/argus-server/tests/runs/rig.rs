//! A test server whose run service starts pipes the test drives by hand, and the calls the run
//! tests make over and over.

#![allow(dead_code)]

use std::sync::Arc;

use argus_launchers::FakeHost;
use argus_testkit::{Reply, Request, SseReader, TestOptions, TestServer, write_board};
use serde_json::{Value, json};

#[path = "../support/wait.rs"]
mod wait;

pub use wait::until;

/// A definition the way `coord run define` writes it; `cwd` is a folder that exists.
pub fn def(name: &str, cwd: &str, extra: &Value) -> Value {
    let mut d = json!({
        "name": name, "cmd": format!("fake {name}"), "cwd": cwd, "kind": "task", "env": {},
        "hand": false, "visible": false, "timeoutSec": 0, "parallel": 1,
        "definedBy": "test", "definedAt": "2026-01-01T00:00:00.000Z", "rev": 1,
    });
    if let (Some(base), Some(more)) = (d.as_object_mut(), extra.as_object()) {
        base.extend(more.clone());
    }
    d
}

pub struct Rig {
    pub t: TestServer,
    pub host: FakeHost,
}

impl Rig {
    pub async fn with(options: TestOptions) -> Rig {
        let host = FakeHost::new();
        let wired = host.clone();
        let tweak = options.tweak.clone();
        let t = TestServer::start(TestOptions {
            sync: false,
            tweak: Some(Arc::new(move |opts| {
                opts.run_service.host = Some(wired.clone());
                if let Some(more) = &tweak {
                    more(opts);
                }
            })),
            ..options
        })
        .await;
        Rig { t, host }
    }

    pub async fn start() -> Rig {
        Rig::with(TestOptions::default()).await
    }

    /// Defines launches of project `alpha` on its board file.
    pub fn define(&self, defs: &[(&str, Value)]) {
        let cwd = self.t.coord_home.display().to_string();
        let run_defs: serde_json::Map<String, Value> = defs
            .iter()
            .map(|(name, extra)| ((*name).to_owned(), def(name, &cwd, extra)))
            .collect();
        write_board(
            &self.t.coord_home,
            "alpha",
            &json!({ "project": "alpha", "schema": 2, "runDefs": run_defs }),
        );
    }

    /// A call as the CLI makes it (a script: no fetch metadata).
    pub async fn cli(&self, method: &str, path: &str, body: &Value) -> Reply {
        self.t.write(method, path, body).await
    }

    /// A call as the owner's page makes it.
    pub async fn page(&self, method: &str, path: &str, body: &Value) -> Reply {
        let request = Request::new(method, path).json(body);
        self.t.send(self.t.page(request)).await
    }

    /// Starts a launch as the CLI; the run it answers.
    pub async fn begin(&self, name: &str) -> Value {
        let out = self
            .cli("POST", "/api/p/alpha/runs", &json!({ "name": name }))
            .await;
        assert_eq!(out.status, 200, "{}", out.text());
        out.json()["run"].clone()
    }

    pub async fn run(&self, rid: &str) -> Value {
        self.t.get(&format!("/api/p/alpha/runs/{rid}")).await.json()["run"].clone()
    }

    /// Waits until the run has this status.
    pub async fn until_status(&self, rid: &str, status: &str) -> Value {
        until(&format!("{rid} to be {status}"), || async {
            let run = self.run(rid).await;
            (run["status"] == status).then_some(run)
        })
        .await
    }

    /// The text of the lines of a run, in order.
    pub async fn texts(&self, rid: &str) -> Vec<String> {
        let page = self
            .t
            .get(&format!("/api/p/alpha/runs/{rid}/logs?limit=5000"))
            .await
            .json();
        let lines = page["lines"].as_array().cloned().unwrap_or_default();
        lines
            .iter()
            .filter_map(|l| l["text"].as_str().map(str::to_owned))
            .collect()
    }

    pub async fn until_line(&self, rid: &str, text: &str) {
        until(&format!("the line {text}"), || async {
            self.texts(rid)
                .await
                .iter()
                .any(|t| t == text)
                .then_some(())
        })
        .await;
    }

    /// Opens the log stream of a run.
    pub async fn tail(&self, rid: &str, query: &str) -> SseReader {
        self.t
            .sse(&format!("/api/p/alpha/runs/{rid}/logs/stream{query}"))
            .await
    }
}
