//! A scripted [`ChildHost`] for the supervisor specs: the test plays the child by saying stdout
//! lines and by dying, and reads what the supervisor wrote to stdin.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::ports::{ChildHost, ChildSpec, HostFault, LineStream, SpawnedChild};

#[derive(Default)]
struct Inner {
    spawned: Vec<u32>,
    outputs: HashMap<u32, UnboundedSender<String>>,
    written: Vec<(u32, String)>,
    stopped: Vec<u32>,
    killed: Vec<u32>,
}

#[derive(Clone, Default)]
pub(super) struct FakeHost(Arc<Mutex<Inner>>);

pub(super) struct FakeLines(UnboundedReceiver<String>);

impl LineStream for FakeLines {
    async fn next_line(&mut self) -> Option<String> {
        self.0.recv().await
    }
}

impl FakeHost {
    fn inner(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.0.lock().unwrap()
    }

    /// The pids started so far, oldest first.
    pub(super) fn pids(&self) -> Vec<u32> {
        self.inner().spawned.clone()
    }

    /// The child prints a line.
    pub(super) fn say(&self, pid: u32, line: &str) {
        if let Some(out) = self.inner().outputs.get(&pid) {
            let _ = out.send(line.to_owned());
        }
    }

    /// The child dies: its stdout closes.
    pub(super) fn die(&self, pid: u32) {
        self.inner().outputs.remove(&pid);
    }

    /// The texts of the user messages written to `pid`, oldest first.
    pub(super) fn texts(&self, pid: u32) -> Vec<String> {
        self.inner()
            .written
            .iter()
            .filter(|(p, _)| *p == pid)
            .map(|(_, line)| {
                let value: serde_json::Value = serde_json::from_str(line).unwrap();
                value["message"]["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }

    pub(super) fn killed(&self) -> Vec<u32> {
        self.inner().killed.clone()
    }

    pub(super) fn stopped(&self) -> Vec<u32> {
        self.inner().stopped.clone()
    }
}

impl ChildHost for FakeHost {
    type Lines = FakeLines;

    fn spawn(&self, _spec: &ChildSpec) -> Result<SpawnedChild<FakeLines>, HostFault> {
        let mut inner = self.inner();
        let pid = 100 + u32::try_from(inner.spawned.len()).unwrap();
        let (tx, rx) = unbounded_channel();
        inner.spawned.push(pid);
        inner.outputs.insert(pid, tx);
        Ok(SpawnedChild {
            pid,
            stdout: FakeLines(rx),
        })
    }

    fn write_line(
        &self,
        pid: u32,
        line: &str,
    ) -> impl Future<Output = Result<(), HostFault>> + Send {
        let mut inner = self.inner();
        let wrote = if inner.outputs.contains_key(&pid) {
            inner.written.push((pid, line.to_owned()));
            Ok(())
        } else {
            Err(HostFault("not running".to_owned()))
        };
        std::future::ready(wrote)
    }

    async fn close_stdin(&self, _pid: u32) {}

    async fn stop(&self, pid: u32) {
        let mut inner = self.inner();
        inner.stopped.push(pid);
        inner.outputs.remove(&pid);
    }

    async fn kill_tree(&self, pid: u32) {
        let mut inner = self.inner();
        inner.killed.push(pid);
        inner.outputs.remove(&pid);
    }
}
