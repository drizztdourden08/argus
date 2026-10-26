//! Shared setup of the run suites: a service on the in-memory store with the fake host, a
//! clock that follows Tokio's (pausable) time, and helpers to wait for conditions.

use std::sync::Arc;
use std::time::Duration;

use argus_launchers::{
    Clock, FakeHost, ListFilter, LogLine, LogOptions, MemoryDefs, MemoryStore, NewRun, Origin,
    RecordingEvents, RunConfig, RunDef, RunDeps, RunEvents, RunKind, RunLimits, RunPatch, RunRow,
    RunService, RunStatus, RunStore, StartOptions,
};
use tempfile::TempDir;
use tokio::time::Instant;

/// The signature and a header of a PNG: enough to pass the magic-number check.
pub const PNG: [u8; 12] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0];

/// Wall-clock milliseconds at the start of the test plus Tokio's (possibly paused) time.
pub struct TokioClock {
    base_ms: i64,
    start: Instant,
}

impl TokioClock {
    pub fn new() -> Self {
        Self {
            base_ms: argus_launchers::system_now_ms(),
            start: Instant::now(),
        }
    }
}

impl Clock for TokioClock {
    fn now_ms(&self) -> i64 {
        self.base_ms + i64::try_from(self.start.elapsed().as_millis()).unwrap_or(0)
    }
}

pub struct EnvOptions {
    pub limits: RunLimits,
    pub token: Option<String>,
    pub attach_max_bytes: usize,
    pub events: Option<Arc<dyn RunEvents>>,
    pub store: Option<Arc<MemoryStore>>,
    pub host: Option<FakeHost>,
    pub boot: bool,
}

impl Default for EnvOptions {
    fn default() -> Self {
        Self {
            limits: RunLimits::default(),
            token: None,
            attach_max_bytes: 10 * 1024 * 1024,
            events: None,
            store: None,
            host: None,
            boot: true,
        }
    }
}

pub struct Env {
    pub host: FakeHost,
    pub store: Arc<MemoryStore>,
    pub events: Arc<RecordingEvents>,
    pub defs: Arc<MemoryDefs>,
    pub svc: RunService<FakeHost>,
    pub dir: TempDir,
}

pub fn cli() -> StartOptions {
    StartOptions {
        origin: Origin::Cli,
        by: Some("tester".to_owned()),
        ..StartOptions::default()
    }
}

pub fn page() -> StartOptions {
    StartOptions {
        origin: Origin::Page,
        by: Some("human".to_owned()),
        ..StartOptions::default()
    }
}

impl Env {
    pub fn new() -> Self {
        Self::build(EnvOptions::default())
    }

    pub fn with_limits(limits: RunLimits) -> Self {
        Self::build(EnvOptions {
            limits,
            ..EnvOptions::default()
        })
    }

    pub fn build(options: EnvOptions) -> Self {
        let dir = TempDir::new().unwrap();
        let host = options.host.unwrap_or_default();
        let store = options.store.unwrap_or_default();
        let recording = Arc::new(RecordingEvents::new());
        let events: Arc<dyn RunEvents> = options
            .events
            .unwrap_or_else(|| Arc::clone(&recording) as Arc<dyn RunEvents>);
        let defs = Arc::new(MemoryDefs::new());
        let svc = RunService::new(RunDeps {
            host: host.clone(),
            store: Arc::clone(&store) as _,
            events,
            defs: Arc::clone(&defs) as _,
            config: RunConfig {
                limits: options.limits,
                token: options.token,
                attachments_dir: dir.path().join("attachments"),
                attach_max_bytes: options.attach_max_bytes,
            },
            clock: Arc::new(TokioClock::new()),
        });
        if options.boot {
            svc.recover();
        }
        Self {
            host,
            store,
            events: recording,
            defs,
            svc,
            dir,
        }
    }

    /// A definition that runs in the temp folder.
    pub fn def(&self, name: &str, cmd: &str) -> RunDef {
        RunDef {
            name: name.to_owned(),
            cmd: cmd.to_owned(),
            cwd: self.dir.path().to_string_lossy().into_owned(),
            ..RunDef::default()
        }
    }

    pub fn define(&self, key: &str, defs: Vec<RunDef>) {
        self.defs.set(key, defs);
    }

    /// The pid the fake host handed out as the `n`th (0-based).
    pub fn pid(&self, n: usize) -> u32 {
        self.host.pids()[n]
    }

    /// Starts a launch as the CLI and returns the run.
    pub fn start(&self, key: &str, name: &str) -> RunRow {
        self.svc.start(key, name, &cli()).unwrap().run
    }

    pub async fn finished(&self, key: &str, rid: &str) -> RunRow {
        until("the run to finish", || {
            self.svc.get(key, rid).filter(|r| !r.status.is_live())
        })
        .await
    }

    pub fn lines(&self, key: &str, rid: &str, stream: Option<&str>) -> Vec<LogLine> {
        self.svc
            .logs(
                key,
                rid,
                &LogOptions {
                    stream: stream.map(str::to_owned),
                    limit: Some(5000),
                    ..LogOptions::default()
                },
            )
            .unwrap()
            .lines
    }

    pub fn texts(&self, key: &str, rid: &str, stream: Option<&str>) -> Vec<String> {
        self.lines(key, rid, stream)
            .into_iter()
            .map(|l| l.text)
            .collect()
    }

    pub fn text(&self, key: &str, rid: &str, stream: Option<&str>) -> String {
        self.texts(key, rid, stream).join("\n")
    }

    pub fn list(&self, key: &str) -> Vec<RunRow> {
        self.svc.list(key, &ListFilter::default())
    }
}

/// Polls every 10 ms (of Tokio time) until the check gives a value; fails after a minute.
pub async fn until<T>(what: &str, mut check: impl FnMut() -> Option<T>) -> T {
    for _ in 0..6000 {
        if let Some(value) = check() {
            return value;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("timed out waiting for {what}");
}

/// A new `alpha` run as a crashed server would have stored it.
pub fn new_run(rid: &str, name: &str) -> NewRun {
    NewRun {
        scope: "alpha".to_owned(),
        rid: rid.to_owned(),
        name: name.to_owned(),
        kind: RunKind::Task,
        cmd: "cmd".to_owned(),
        cwd: ".".to_owned(),
        visible: false,
        origin: Origin::Cli,
        by: "tester".to_owned(),
        agent: None,
        reason: None,
        queued_at: "2026-01-01T00:00:00.000Z".to_owned(),
    }
}

/// Stores a run in `status` with `pid`, the way a crashed server left it.
pub fn left_behind(store: &MemoryStore, rid: &str, status: RunStatus, pid: Option<u32>) -> RunRow {
    store.insert(new_run(rid, "slow"));
    let patch = RunPatch {
        status: Some(status),
        pid: Some(pid),
        ..RunPatch::default()
    };
    store.update("alpha", rid, &patch).unwrap()
}

/// A service over `store` and `host` that has not booted yet.
pub fn unbooted(store: &Arc<MemoryStore>, host: &FakeHost) -> Env {
    Env::build(EnvOptions {
        store: Some(Arc::clone(store)),
        host: Some(host.clone()),
        boot: false,
        ..EnvOptions::default()
    })
}
