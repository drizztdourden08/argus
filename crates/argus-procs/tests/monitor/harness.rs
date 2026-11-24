//! A monitor over fake collaborators: a scripted sampler, a recording store, host and killer.

use argus_procs::{
    Activity, KillResult, Killer, Meta, NoResponder, Options, Ports, ProcDoc, ProcRow, ProcsConfig,
    ProcsHost, ProcsService, ProcsStore, ProjectFolders, ProjectRoot, Sample, Sampler, WakeRequest,
};
use serde_json::{Value, json};
use std::collections::HashMap;
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
    pub cwd: Option<&'static str>,
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
            cwd: None,
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
        cwd: spec.cwd.map(ToString::to_string),
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
    /// How many times the store was written.
    pub writes: Mutex<usize>,
    pub killed: Mutex<Vec<u32>>,
    /// Each kill as (pid, tree, the descendants it was given).
    pub trees: Mutex<Vec<(u32, bool, Vec<u32>)>>,
    /// The folders of the projects; none means the default (the roots).
    pub folders: Mutex<Option<Vec<ProjectFolders>>>,
    /// The folders `cwds` answers, for rows sampled without one.
    pub cwds: Mutex<HashMap<u32, String>>,
    /// Every kill fails.
    pub kill_fails: Mutex<bool>,
}

impl Fake {
    pub fn trees(&self) -> Vec<(u32, bool, Vec<u32>)> {
        lock(&self.trees).clone()
    }

    pub fn set_folders(&self, folders: &[(&str, &[&str])]) {
        let all = folders
            .iter()
            .map(|(key, list)| ProjectFolders {
                key: (*key).to_string(),
                folders: list.iter().map(ToString::to_string).collect(),
            })
            .collect();
        *lock(&self.folders) = Some(all);
    }

    pub fn set_cwd(&self, pid: u32, cwd: &str) {
        lock(&self.cwds).insert(pid, cwd.to_string());
    }

    pub fn fail_kills(&self) {
        *lock(&self.kill_fails) = true;
    }

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

    pub fn writes(&self) -> usize {
        *lock(&self.writes)
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

    fn cwds(&self, pids: &[u32]) -> HashMap<u32, String> {
        let known = lock(&self.cwds);
        pids.iter()
            .filter_map(|p| known.get(p).map(|c| (*p, c.clone())))
            .collect()
    }
}

impl ProcsStore for Fake {
    fn replace(&self, docs: &[ProcDoc], meta: Option<&Meta>) -> Result<(), String> {
        *lock(&self.published) = Some((docs.to_vec(), meta.cloned()));
        *lock(&self.writes) += 1;
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

    fn project_folders(&self) -> Vec<ProjectFolders> {
        lock(&self.folders).clone().unwrap_or_else(|| {
            self.project_roots()
                .into_iter()
                .map(|r| ProjectFolders {
                    key: r.key,
                    folders: vec![r.root],
                })
                .collect()
        })
    }

    fn lead_of(&self, project: &str) -> Option<String> {
        Some(format!("{project}-lead"))
    }

    fn tray_pid(&self) -> Option<u32> {
        *lock(&self.tray)
    }

    fn overlord_board(&self) -> String {
        "portfolio".to_string()
    }

    fn lead(&self) -> Option<String> {
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
    fn kill(&self, pid: u32, tree: bool, descendants: &[u32]) -> KillResult {
        if *lock(&self.kill_fails) {
            return KillResult {
                ok: false,
                detail: Some("access is denied".to_string()),
            };
        }
        lock(&self.killed).push(pid);
        lock(&self.trees).push((pid, tree, descendants.to_vec()));
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
    /// config.json `procs.reaper`.
    pub reaper: Option<Value>,
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
        let mut config = setup
            .repeat_ms
            .map_or(json!({}), |ms| json!({ "repeatMs": ms }));
        if let Some(reaper) = setup.reaper {
            config["reaper"] = reaper;
        }
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
