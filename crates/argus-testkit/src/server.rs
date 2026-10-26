//! A real server on an ephemeral loopback port with throwaway homes. Requests go through the
//! small client, so a test can set any header; the bearer is sent only when the test asks.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use argus_core::Paths;
use argus_server::{ServeOptions, ServerHandle, Services, serve};
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::{Reply, Request, SseReader, make_home, request};

/// Changes the serve options of a test server after the testkit set them.
pub type Tweak = Arc<dyn Fn(&mut ServeOptions) + Send + Sync>;

/// How to start a test server. Services that sample or act on the machine are off unless a
/// tweak turns them on.
pub struct TestOptions {
    /// Write the fixture boards first.
    pub seed: bool,
    /// Keys merged into `config.json` before the start (one level deep).
    pub config: Value,
    /// Run the sync service.
    pub sync: bool,
    /// Shadow mode.
    pub shadow: bool,
    /// The SSE heartbeat.
    pub heartbeat: Duration,
    /// Reuse a coord home (a restart).
    pub coord_home: Option<PathBuf>,
    /// Reuse a dashboard home (a restart).
    pub dashboard_home: Option<PathBuf>,
    /// Last say on the serve options (a service's seams).
    pub tweak: Option<Tweak>,
}

impl Default for TestOptions {
    fn default() -> Self {
        TestOptions {
            seed: false,
            config: json!({}),
            sync: true,
            shadow: false,
            heartbeat: Duration::from_secs(15),
            coord_home: None,
            dashboard_home: None,
            tweak: None,
        }
    }
}

/// A running test server.
pub struct TestServer {
    /// `http://127.0.0.1:<port>`.
    pub url: String,
    /// The port.
    pub port: u16,
    /// The token.
    pub token: String,
    /// The coord home.
    pub coord_home: PathBuf,
    /// The dashboard home.
    pub dashboard_home: PathBuf,
    /// The services.
    pub svc: Arc<Services>,
    handle: Option<ServerHandle>,
    dirs: Vec<TempDir>,
}

fn patch_config(paths: &Paths, patch: &Value) {
    let Some(patch) = patch.as_object().filter(|p| !p.is_empty()) else {
        return;
    };
    let mut c = argus_core::config::load_config(paths, None).expect("config");
    let mut v = serde_json::to_value(&c).expect("config json");
    for (k, val) in patch {
        match (v.get_mut(k), val) {
            (Some(Value::Object(have)), Value::Object(add)) => have.extend(add.clone()),
            _ => v[k] = val.clone(),
        }
    }
    c = serde_json::from_value(v).expect("patched config");
    argus_core::config::save_config(paths, &c, None).expect("save config");
}

fn dir_or_temp(given: Option<PathBuf>, dirs: &mut Vec<TempDir>) -> PathBuf {
    given.unwrap_or_else(|| {
        let d = tempfile::tempdir().expect("temp dir");
        let p = d.path().to_path_buf();
        dirs.push(d);
        p
    })
}

impl TestServer {
    /// Starts a server.
    ///
    /// # Panics
    /// When it does not start (a test failure).
    pub async fn start(o: TestOptions) -> TestServer {
        let mut dirs = Vec::new();
        let coord_home = dir_or_temp(o.coord_home, &mut dirs);
        let dashboard_home = dir_or_temp(o.dashboard_home, &mut dirs);
        if o.seed {
            make_home(&coord_home);
        }
        let paths = Paths::new(&coord_home, Some(dashboard_home.clone()));
        patch_config(&paths, &o.config);
        let mut opts = ServeOptions::new(paths);
        opts.port = Some(0);
        opts.sync = o.sync;
        opts.shadow = o.shadow;
        opts.heartbeat = o.heartbeat;
        opts.web_root = Some(dashboard_home.join("no-web-build"));
        opts.procs.enabled = false;
        if let Some(tweak) = &o.tweak {
            tweak(&mut opts);
        }
        let handle = tokio::time::timeout(Duration::from_secs(30), serve(opts))
            .await
            .expect("the test server starts within 30 s")
            .expect("the test server starts");
        let svc = Arc::clone(&handle.svc);
        TestServer {
            url: handle.url.clone(),
            port: handle.port,
            token: svc.token(),
            coord_home,
            dashboard_home,
            svc,
            handle: Some(handle),
            dirs,
        }
    }

    /// Sends a request.
    pub async fn send(&self, req: Request) -> Reply {
        request(self.port, &req).await
    }

    /// A GET.
    pub async fn get(&self, path: &str) -> Reply {
        self.send(Request::new("GET", path)).await
    }

    /// A JSON write (`POST`, `PUT`...).
    pub async fn write(&self, method: &str, path: &str, body: &Value) -> Reply {
        self.send(Request::new(method, path).json(body)).await
    }

    /// The headers a browser page's write carries.
    #[must_use]
    pub fn page(&self, req: Request) -> Request {
        req.header("Origin", &self.url)
            .header("Sec-Fetch-Site", "same-origin")
    }

    /// The valid bearer added.
    #[must_use]
    pub fn bearer(&self, req: Request) -> Request {
        req.header("Authorization", &format!("Bearer {}", self.token))
    }

    /// Opens a stream.
    pub async fn sse(&self, path: &str) -> SseReader {
        self.sse_with(Request::new("GET", path)).await
    }

    /// Opens a stream with extra headers.
    pub async fn sse_with(&self, req: Request) -> SseReader {
        SseReader::open(self.port, &req.header("Accept", "text/event-stream")).await
    }

    /// The board file of a key, parsed.
    ///
    /// # Panics
    /// When there is no such board.
    #[must_use]
    pub fn board(&self, key: &str) -> Value {
        read_json(&self.coord_home.join(key).join("board.json"))
    }

    /// Stops the server; the homes are removed unless `keep`.
    ///
    /// # Panics
    /// When the server does not stop within 10 s.
    pub async fn stop(mut self, keep: bool) -> (PathBuf, PathBuf) {
        if let Some(h) = self.handle.take() {
            tokio::time::timeout(Duration::from_secs(10), h.stop())
                .await
                .expect("the server stops within 10 s");
        }
        if keep {
            for d in self.dirs.drain(..) {
                let _ = d.keep();
            }
        }
        (self.coord_home.clone(), self.dashboard_home.clone())
    }
}

/// A JSON file, parsed.
///
/// # Panics
/// When it is missing or not JSON.
#[must_use]
pub fn read_json(file: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(file).expect("json file")).expect("valid json")
}
