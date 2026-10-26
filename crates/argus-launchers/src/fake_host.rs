//! A process host for tests: processes are in-memory pipes the test writes to and ends by
//! hand, and a kill ends the fake process at once. Nothing real is started.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream, duplex};
use tokio::sync::oneshot;

use crate::host::{ExitInfo, ProcessHost, SpawnRequest, SpawnedProcess};

const PIPE_BYTES: usize = 1 << 20;

struct FakeProc {
    stdout: tokio::sync::Mutex<Option<DuplexStream>>,
    stderr: tokio::sync::Mutex<Option<DuplexStream>>,
    stdin: Option<tokio::sync::Mutex<DuplexStream>>,
    report: Mutex<Option<oneshot::Sender<ExitInfo>>>,
    alive: AtomicBool,
}

#[derive(Default)]
struct FakeState {
    next_pid: u32,
    procs: HashMap<u32, Arc<FakeProc>>,
    requests: Vec<(u32, SpawnRequest)>,
    spawn_error: Option<String>,
    killed: Vec<u32>,
    foreign: HashSet<u32>,
}

/// Cloning shares the same fake machine.
#[derive(Clone, Default)]
pub struct FakeHost {
    state: Arc<Mutex<FakeState>>,
}

impl FakeHost {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, FakeState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn proc(&self, pid: u32) -> Option<Arc<FakeProc>> {
        self.lock().procs.get(&pid).cloned()
    }

    /// Every following spawn fails with this message; `None` makes them work again.
    pub fn fail_spawns(&self, message: Option<&str>) {
        self.lock().spawn_error = message.map(str::to_owned);
    }

    /// The requests spawned so far, oldest first.
    pub fn requests(&self) -> Vec<SpawnRequest> {
        self.lock()
            .requests
            .iter()
            .map(|(_, r)| r.clone())
            .collect()
    }

    /// The pids handed out so far, oldest first.
    pub fn pids(&self) -> Vec<u32> {
        self.lock().requests.iter().map(|(pid, _)| *pid).collect()
    }

    /// The pids that `kill_tree` was asked to end, in order.
    pub fn killed(&self) -> Vec<u32> {
        self.lock().killed.clone()
    }

    /// A pid that is alive but is not the shell of a run (a recycled pid).
    pub fn add_foreign(&self, pid: u32) {
        self.lock().foreign.insert(pid);
    }

    /// Output on the process's stdout.
    pub async fn say(&self, pid: u32, text: &str) {
        self.say_bytes(pid, text.as_bytes()).await;
    }

    pub async fn say_bytes(&self, pid: u32, bytes: &[u8]) {
        if let Some(proc) = self.proc(pid)
            && let Some(pipe) = proc.stdout.lock().await.as_mut()
        {
            let _ = pipe.write_all(bytes).await;
        }
    }

    /// Output on the process's stderr.
    pub async fn complain(&self, pid: u32, text: &str) {
        if let Some(proc) = self.proc(pid)
            && let Some(pipe) = proc.stderr.lock().await.as_mut()
        {
            let _ = pipe.write_all(text.as_bytes()).await;
        }
    }

    /// Closes the output pipes only: the process lives on (a grandchild holds nothing open).
    pub async fn close_pipes(&self, pid: u32) {
        if let Some(proc) = self.proc(pid) {
            proc.stdout.lock().await.take();
            proc.stderr.lock().await.take();
        }
    }

    /// The process ends: pipes close and the exit is reported.
    pub async fn exit(&self, pid: u32, code: Option<i32>) {
        self.finish(pid, ExitInfo { code, signal: None }).await;
    }

    async fn finish(&self, pid: u32, info: ExitInfo) {
        let Some(proc) = self.proc(pid) else { return };
        proc.alive.store(false, Ordering::SeqCst);
        proc.stdout.lock().await.take();
        proc.stderr.lock().await.take();
        let report = proc
            .report
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(report) = report {
            let _ = report.send(info);
        }
    }

    /// The exit is reported but the pipes stay open (a grandchild holds them).
    pub fn exit_keeping_pipes(&self, pid: u32, code: Option<i32>) {
        let Some(proc) = self.proc(pid) else { return };
        proc.alive.store(false, Ordering::SeqCst);
        let report = proc
            .report
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(report) = report {
            let _ = report.send(ExitInfo { code, signal: None });
        }
    }

    /// What the dashboard has typed into the process's stdin so far.
    pub async fn stdin_text(&self, pid: u32) -> String {
        let Some(proc) = self.proc(pid) else {
            return String::new();
        };
        let Some(pipe) = &proc.stdin else {
            return String::new();
        };
        let mut pipe = pipe.lock().await;
        let mut text = Vec::new();
        let mut chunk = [0_u8; 4096];
        while let Ok(Ok(n)) =
            tokio::time::timeout(Duration::from_millis(5), pipe.read(&mut chunk)).await
        {
            if n == 0 {
                break;
            }
            text.extend_from_slice(&chunk[..n]);
        }
        String::from_utf8_lossy(&text).into_owned()
    }

    fn start(&self, request: &SpawnRequest) -> io::Result<SpawnedProcess> {
        let mut state = self.lock();
        if let Some(message) = &state.spawn_error {
            return Err(io::Error::other(message.clone()));
        }
        state.next_pid += 1;
        let pid = 1000 + state.next_pid;
        let (stdout_read, stdout_write) = duplex(PIPE_BYTES);
        let (stderr_read, stderr_write) = duplex(PIPE_BYTES);
        let (stdin_dashboard, stdin_process) = duplex(PIPE_BYTES);
        let (report, exit) = oneshot::channel();
        let proc = FakeProc {
            stdout: tokio::sync::Mutex::new(Some(stdout_write)),
            stderr: tokio::sync::Mutex::new(Some(stderr_write)),
            stdin: request
                .stdin
                .then(|| tokio::sync::Mutex::new(stdin_process)),
            report: Mutex::new(Some(report)),
            alive: AtomicBool::new(true),
        };
        state.procs.insert(pid, Arc::new(proc));
        state.requests.push((pid, request.clone()));
        Ok(SpawnedProcess {
            pid: Some(pid),
            stdout: Box::new(stdout_read),
            stderr: Box::new(stderr_read),
            stdin: request.stdin.then(|| Box::new(stdin_dashboard) as _),
            exit,
        })
    }
}

impl ProcessHost for FakeHost {
    fn spawn(&self, request: &SpawnRequest) -> io::Result<SpawnedProcess> {
        self.start(request)
    }

    async fn kill_tree(&self, pid: u32, _grace: Duration) {
        self.lock().killed.push(pid);
        let alive = self
            .proc(pid)
            .is_some_and(|p| p.alive.load(Ordering::SeqCst));
        if alive {
            self.finish(
                pid,
                ExitInfo {
                    code: Some(1),
                    signal: None,
                },
            )
            .await;
        }
    }

    fn is_alive(&self, pid: u32) -> bool {
        let state = self.lock();
        state.foreign.contains(&pid)
            || state
                .procs
                .get(&pid)
                .is_some_and(|p| p.alive.load(Ordering::SeqCst))
    }

    fn looks_like_ours(&self, pid: u32) -> impl Future<Output = bool> + Send {
        std::future::ready(
            self.proc(pid)
                .is_some_and(|p| p.alive.load(Ordering::SeqCst)),
        )
    }
}
