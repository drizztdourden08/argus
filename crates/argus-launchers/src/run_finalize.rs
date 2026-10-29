//! The end of a run: the final status, the closing log lines, the retention sweep, the
//! end listeners and the next queued run taking the slot. Also a run that never got a
//! process, and a visible run the dashboard lets go of.

use std::sync::Arc;

use crate::host::{ExitInfo, ProcessHost};
use crate::live_run::LiveRun;
use crate::record::{RunPatch, RunRow, RunStatus};
use crate::run_service::{RunService, run_key};
use crate::run_verdict::{EndInput, verdict};

/// A command that returns this fast with code 0 has probably only launched a program that
/// keeps running.
const QUICK_EXIT_MS: i64 = 5000;
const RETURNED_NOTE: &str = "the command returned; a program it started may still be running";

/// How a supervisor stopped watching.
pub(crate) enum Ending {
    /// The process is over (or given up on); the exit is `None` when it never reported one.
    Done(Option<ExitInfo>),
    /// The dashboard lets a visible run go on without it.
    Released,
}

impl<H: ProcessHost> RunService<H> {
    pub(crate) async fn conclude(&self, live: &Arc<LiveRun>, ending: Ending) {
        match ending {
            Ending::Done(exit) => self.finalize(live, exit).await,
            Ending::Released => self.release(live).await,
        }
        live.done.send_replace(true);
        self.drain();
    }

    async fn finalize(&self, live: &Arc<LiveRun>, exit: Option<ExitInfo>) {
        let (stop_by, timed_out) = {
            let control = live.control();
            (control.stop_by.clone(), control.timed_out)
        };
        let outcome = verdict(&EndInput {
            stop_by: stop_by.clone(),
            timed_out,
            timeout_sec: live.def.timeout_sec,
            exit: exit.clone(),
        });
        {
            let mut writer = live.writer();
            writer.release_partials();
            writer.sys(&outcome.note);
            let quick = self.inner.deps.clock.now_ms() - live.launched_at_ms < QUICK_EXIT_MS;
            if outcome.status == RunStatus::Exited && live.interactive && quick {
                writer.sys(RETURNED_NOTE);
            }
            writer.end();
        }
        live.stdin.lock().await.take();
        let exit = exit.unwrap_or_default();
        let patch = RunPatch {
            status: Some(outcome.status),
            exit_code: Some(exit.code),
            signal: Some(exit.signal),
            ended_at: Some(Some(self.now_iso())),
            error: Some(outcome.error),
            stdin: Some(false),
            by: stop_by,
            ..RunPatch::default()
        };
        self.state().live.remove(&live.id);
        let ended = self.set_state(&live.scope, &live.rid, &patch);
        self.prune(&live.scope, &live.name);
        if let Some(row) = &ended {
            self.notify_ended(row);
        }
    }

    /// A visible run the owner is using is left alone: noted, its pipes let go, its row
    /// marked as an orphan that Stop can still kill.
    async fn release(&self, live: &Arc<LiveRun>) {
        live.writer().release_partials();
        live.sys("the dashboard stopped; this visible run was left running");
        live.writer().end();
        live.stdin.lock().await.take();
        self.state().live.remove(&live.id);
        let still = live
            .pid
            .map(|pid| format!(" (pid {pid} still alive; Stop kills it)"))
            .unwrap_or_default();
        self.set_state(
            &live.scope,
            &live.rid,
            &RunPatch {
                status: Some(RunStatus::Failed),
                orphan: Some(true),
                stdin: Some(false),
                ended_at: Some(Some(self.now_iso())),
                error: Some(Some(format!("server stopped while running{still}"))),
                ..RunPatch::default()
            },
        );
    }

    /// A run that never got a process (bad cwd, refused spawn, stopped while queued,
    /// shutdown): straight to a final state with the reason in its log.
    pub(crate) fn fail_now(
        &self,
        row: &RunRow,
        message: &str,
        status: RunStatus,
        by: Option<&str>,
    ) -> RunRow {
        let mut writer = self.writer_for(row, self.redactor_for(None));
        writer.sys(message);
        writer.end();
        self.state().waiting.remove(&run_key(&row.scope, &row.id));
        let patch = RunPatch {
            status: Some(status),
            ended_at: Some(Some(self.now_iso())),
            error: Some((status == RunStatus::Failed).then(|| message.to_owned())),
            by: by.map(str::to_owned),
            ..RunPatch::default()
        };
        let ended = self
            .set_state(&row.scope, &row.id, &patch)
            .unwrap_or_else(|| row.clone());
        self.prune(&row.scope, &row.name);
        self.notify_ended(&ended);
        ended
    }

    /// An ended run the dashboard found harmful after the fact (a check run that changed tracked
    /// files): failed, with `error` as its reason. `None` for an unknown or a still live run.
    pub fn fail_ended(&self, key: &str, rid: &str, error: &str) -> Option<RunRow> {
        let row = self.get(key, rid).filter(|r| !r.status.is_live())?;
        let patch = RunPatch {
            status: Some(RunStatus::Failed),
            error: Some(Some(error.to_owned())),
            ..RunPatch::default()
        };
        self.set_state(&row.scope, &row.id, &patch)
    }

    /// A slot freed: the oldest queued run (any scope) takes it.
    pub(crate) fn drain(&self) {
        while let Some(outcome) = self.start_next_queued() {
            if let Err((row, message)) = outcome {
                self.fail_now(&row, &message, RunStatus::Failed, None);
            }
        }
    }

    /// `None`: nothing more to start (shutting down, no free slot, nothing queued).
    /// `Some(Ok)`: a queued run was started. `Some(Err)`: the oldest queued run cannot
    /// start, to be failed once the lock is released.
    fn start_next_queued(&self) -> Option<Result<(), (RunRow, String)>> {
        let mut state = self.state();
        if state.closing || state.live.len() >= self.limits().max_concurrent {
            return None;
        }
        let row = self.inner.deps.store.queued_rows().into_iter().next()?;
        let outcome = match state.waiting.remove(&run_key(&row.scope, &row.id)) {
            None => Err((
                row,
                "the queued run lost its definition (server restarted)".to_owned(),
            )),
            Some(def) => self
                .try_launch(&mut state, &row, &def)
                .map(|_| ())
                .map_err(|message| (row, message)),
        };
        Some(outcome)
    }
}
