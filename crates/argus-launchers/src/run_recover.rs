//! Restart safety. At boot, before serving, nothing stays "running" or "queued": rows
//! `running` become `failed` and `orphan` (with a note when the pid still lives), rows
//! `queued` become `failed`, the run docs are rebuilt and the sweeper starts: the hourly
//! retention sweep, and every half minute the stop of the hidden runs of a build that closed.

use tokio::runtime::Handle;

use crate::host::ProcessHost;
use crate::record::{RunPatch, RunRow, RunStatus};
use crate::run_retired::CLOSED_PLAYS_EVERY;
use crate::run_service::RunService;

/// Retention runs once per this many ticks of the closed-plays check (an hour).
const SWEEP_TICKS: u32 = 120;

impl<H: ProcessHost> RunService<H> {
    fn interrupted_patch(&self, row: &RunRow) -> RunPatch {
        let ended_at = Some(Some(self.now_iso()));
        if row.status == RunStatus::Queued {
            return RunPatch {
                status: Some(RunStatus::Failed),
                ended_at,
                error: Some(Some("server restarted before start".to_owned())),
                ..RunPatch::default()
            };
        }
        let still = row
            .pid
            .filter(|pid| self.inner.deps.host.is_alive(*pid))
            .map(|pid| format!(" (pid {pid} still alive; Stop kills it)"))
            .unwrap_or_default();
        RunPatch {
            status: Some(RunStatus::Failed),
            orphan: Some(true),
            stdin: Some(false),
            ended_at,
            error: Some(Some(format!("server restarted while running{still}"))),
            ..RunPatch::default()
        }
    }

    /// Call once at boot, before serving. Starts the hourly retention sweep when a Tokio
    /// runtime is running.
    pub fn recover(&self) {
        for row in self.inner.deps.store.live_rows() {
            let patch = self.interrupted_patch(&row);
            self.set_state(&row.scope, &row.id, &patch);
        }
        for scope in self.inner.deps.store.scopes() {
            let rows = self.inner.deps.store.of_scope(&scope);
            self.inner.deps.events.reproject(&scope, &rows);
        }
        self.sweep();
        self.start_sweeper();
    }

    fn start_sweeper(&self) {
        let mut state = self.state();
        if state.sweeper.is_some() {
            return;
        }
        let Ok(runtime) = Handle::try_current() else {
            return;
        };
        let service = self.clone();
        state.sweeper = Some(runtime.spawn(async move {
            let mut every = tokio::time::interval(CLOSED_PLAYS_EVERY);
            every.tick().await;
            let mut until_sweep = SWEEP_TICKS;
            loop {
                every.tick().await;
                service.stop_closed_plays().await;
                until_sweep -= 1;
                if until_sweep == 0 {
                    service.sweep();
                    until_sweep = SWEEP_TICKS;
                }
            }
        }));
    }
}
