//! A monitor over fake collaborators: a scripted sampler, a recording store, host and killer.

use argus_procs::{
    Activity, KillResult, Killer, Meta, NoResponder, Options, Ports, ProcDoc, ProcRow, ProcsConfig,
    ProcsHost, ProcsService, ProcsStore, ProjectRoot, Sample, Sampler, WakeRequest,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use tempfile::TempDir;

pub const T0: i64 = 1_792_929_600_000;
pub const MIN: i64 = 60_000;

/// A sampled process; memory in MB, times in ms.
#[derive(Clone, Copy)]
pub struct Spec {
    pub pid: u32,
    pub ppid: u32,
    pub name: &'static str,
    pub cmd: &'static str,
    pub path: &'static str,
    pub mem: u64,
    pub start: i64,
    pub cpu: i64,
    pub responding: Option<bool>,
}

impl Spec {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            ppid: 100,
            name: "node.exe",
            cmd: "node x:/alpha/node_modules/vite/bin/vite.js --port 5173",
            path: "C:\\nvm4w\\nodejs\\node.exe",
            mem: 100,
            start: T0 - 10 * MIN,
            cpu: 0,
            responding: None,
        }
    }
}

pub fn proc(spec: Spec) -> ProcRow {
    ProcRow {
        pid: spec.pid,
        ppid: spec.ppid,
        name: spec.name.to_string(),
        cmd: spec.cmd.to_string(),
        path: Some(spec.path.to_string()),
        mem_bytes: spec.mem * 1_048_576,
        start_ms: spec.start,
        cpu_ms: spec.cpu,
        responding: spec.responding,
    }
}

pub fn root() -> ProcRow {
    proc(Spec {
        ppid: 0,
        name: "explorer.exe",
        cmd: "C:\\Windows\\explorer.exe",
        path: "C:\\Windows\\explorer.exe",
        start: T0 - 86_400_000,
        ..Spec::new(100)
    })
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Default)]
pub struct Fake {
    pub sampler_pid: Mutex<Option<u32>>,
    pub tray: Mutex<Option<u32>>,
    pub current: Mutex<Vec<ProcRow>>,
    pub at: Mutex<i64>,
    pub failure: Mutex<Option<String>>,
    pub sent: Mutex<Vec<WakeRequest>>,
    pub acts: Mutex<Vec<Activity>>,
    pub warns: Mutex<Vec<String>>,
    pub published: Mutex<Option<(Vec<ProcDoc>, Option<Meta>)>>,
    pub killed: Mutex<Vec<u32>>,
}

impl Fake {
    pub fn sent(&self) -> Vec<WakeRequest> {
        lock(&self.sent).clone()
    }

    pub fn acts(&self) -> Vec<Activity> {
        lock(&self.acts).clone()
    }

    pub fn warns(&self) -> Vec<String> {
        lock(&self.warns).clone()
    }

    pub fn killed(&self) -> Vec<u32> {
        lock(&self.killed).clone()
    }

    pub fn published(&self) -> Option<(Vec<ProcDoc>, Option<Meta>)> {
        lock(&self.published).clone()
    }

    pub fn wipe(&self) {
        *lock(&self.published) = None;
    }

    pub fn fail(&self, message: &str) {
        *lock(&self.failure) = Some(message.to_string());
    }
}

impl Sampler for Fake {
    fn sample(&self) -> Result<Sample, String> {
        if let Some(message) = lock(&self.failure).clone() {
            return Err(message);
        }
        Ok(Sample {
            at: *lock(&self.at),
            platform: "win32".to_string(),
            sampler_pid: *lock(&self.sampler_pid),
            procs: lock(&self.current).clone(),
        })
    }
}

impl ProcsStore for Fake {
    fn replace(&self, docs: &[ProcDoc], meta: Option<&Meta>) -> Result<(), String> {
        *lock(&self.published) = Some((docs.to_vec(), meta.cloned()));
        Ok(())
    }
}

impl ProcsHost for Fake {
    fn project_roots(&self) -> Vec<ProjectRoot> {
        vec![ProjectRoot {
            key: "alpha".to_string(),
            root: "X:\\alpha".to_string(),
        }]
    }

    fn tray_pid(&self) -> Option<u32> {
        *lock(&self.tray)
    }

    fn coordinator(&self) -> Option<String> {
        Some("boss".to_string())
    }

    fn submit_wake(&self, wake: &WakeRequest) -> Result<(), String> {
        lock(&self.sent).push(wake.clone());
        Ok(())
    }

    fn activity(&self, entry: &Activity) {
        lock(&self.acts).push(entry.clone());
    }

    fn warn(&self, message: &str, _detail: &str) {
        lock(&self.warns).push(message.to_string());
    }
}

impl Killer for Fake {
    fn kill(&self, pid: u32, _tree: bool, _descendants: &[u32]) -> KillResult {
        lock(&self.killed).push(pid);
        KillResult {
            ok: true,
            detail: None,
        }
    }

    fn is_alive(&self, _pid: u32) -> bool {
        false
    }
}

/// How a monitor under test starts.
#[derive(Default)]
pub struct Setup {
    pub rules: Option<Value>,
    pub repeat_ms: Option<u64>,
    pub self_pid: Option<u32>,
    pub tray: Option<u32>,
    pub sampler_pid: Option<u32>,
}

pub struct Harness {
    pub service: ProcsService,
    pub fake: Arc<Fake>,
    pub dir: TempDir,
}

impl Harness {
    pub fn new(setup: Setup) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let rules_file = dir.path().join("process-rules.json");
        if let Some(rules) = setup.rules {
            std::fs::write(&rules_file, json!({ "rules": rules }).to_string()).unwrap();
        }
        let fake = Arc::new(Fake::default());
        *lock(&fake.tray) = setup.tray;
        *lock(&fake.sampler_pid) = setup.sampler_pid;
        let ports = Ports {
            sampler: fake.clone(),
            responder: Arc::new(NoResponder),
            killer: fake.clone(),
            store: fake.clone(),
            host: fake.clone(),
        };
        let config = setup
            .repeat_ms
            .map_or(json!({}), |ms| json!({ "repeatMs": ms }));
        let options = Options {
            config: ProcsConfig::from_json(&config),
            rules_file,
            own_pid: setup.self_pid.unwrap_or(1),
            system_root: Some("C:\\Windows".to_string()),
            timer: false,
        };
        Self {
            service: ProcsService::new(ports, options),
            fake,
            dir,
        }
    }

    /// Sets what the next sample sees (the root explorer plus `procs`) and takes it.
    pub fn at(&self, ms: i64, procs: Vec<ProcRow>) -> Meta {
        *lock(&self.fake.at) = ms;
        let mut all = vec![root()];
        all.extend(procs);
        *lock(&self.fake.current) = all;
        self.service.sample_now()
    }

    pub fn doc(&self, pid: u32) -> ProcDoc {
        self.service
            .list(None)
            .into_iter()
            .find(|d| d.pid == pid)
            .unwrap()
    }

    pub fn rule_names(&self) -> Vec<String> {
        self.service
            .rules()
            .rules
            .into_iter()
            .map(|r| r.name)
            .collect()
    }
}
