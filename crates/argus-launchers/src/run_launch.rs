//! Putting a queued run on a process: the working directory check, the environment, the
//! spawn, the first status change and the supervisor task.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio::sync::{Notify, watch};

use crate::def::{RunDef, RunKind};
use crate::host::{ProcessHost, SpawnRequest, SpawnedProcess};
use crate::live_run::{Control, LiveRun};
use crate::record::{RunPatch, RunRow, RunStatus};
use crate::run_service::{RunService, State, run_key};
use crate::run_supervise::supervise;

/// The environment a run gets on top of the server's: a launcher's output goes to the
/// dashboard's terminal, which draws colour, and tools only colour a pipe when told to;
/// then the definition's own; then the variables that name the run.
pub(crate) fn run_env(def: &RunDef, row: &RunRow) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    if def.is_playable() && !row.visible {
        env.insert("FORCE_COLOR".to_owned(), "1".to_owned());
        env.insert("TERM".to_owned(), "xterm-256color".to_owned());
    }
    env.extend(def.env.clone());
    env.insert("COORD_PROJECT".to_owned(), row.scope.clone());
    env.insert("COORD_RUN_ID".to_owned(), row.id.clone());
    env.insert("COORD_DASHBOARD".to_owned(), "1".to_owned());
    env
}

/// A launch (and every launcher and bundle) may be an interactive program: it keeps a
/// stdin pipe for the terminal panel's input row.
pub(crate) fn is_interactive(def: &RunDef) -> bool {
    def.is_playable() || def.kind == RunKind::Launch
}

/// The output side of a started process, handed to the supervisor.
pub(crate) struct ProcessIo {
    pub stdout: crate::host::BoxRead,
    pub stderr: crate::host::BoxRead,
    pub exit: tokio::sync::oneshot::Receiver<crate::host::ExitInfo>,
}

fn directory_problem(cwd: &str) -> Option<String> {
    match std::fs::metadata(cwd) {
        Ok(meta) if meta.is_dir() => None,
        Ok(_) => Some("not a directory".to_owned()),
        Err(err) => Some(err.to_string()),
    }
}

impl<H: ProcessHost> RunService<H> {
    /// Starts the process of a queued run. `Err` is the message of a start that cannot
    /// happen (a bad working directory, a refused spawn); the caller fails the run with it
    /// once it holds no lock. The caller has taken the run out of `waiting`.
    pub(crate) fn try_launch(
        &self,
        state: &mut State,
        row: &RunRow,
        def: &RunDef,
    ) -> Result<RunRow, String> {
        if let Some(problem) = directory_problem(&def.cwd) {
            return Err(format!(
                "failed to start: working directory is not usable: {} ({problem})",
                def.cwd
            ));
        }
        let interactive = is_interactive(def);
        let request = SpawnRequest {
            cmd: def.cmd.clone(),
            cwd: PathBuf::from(&def.cwd),
            env: run_env(def, row),
            visible: row.visible,
            stdin: interactive,
        };
        let process = self
            .inner
            .deps
            .host
            .spawn(&request)
            .map_err(|err| format!("failed to start: {err}"))?;
        Ok(self.adopt(state, row, def, process, interactive))
    }

    fn adopt(
        &self,
        state: &mut State,
        row: &RunRow,
        def: &RunDef,
        process: SpawnedProcess,
        interactive: bool,
    ) -> RunRow {
        let redact = self.redactor_for(Some(def));
        let SpawnedProcess {
            pid,
            stdout,
            stderr,
            stdin,
            exit,
        } = process;
        let has_stdin = stdin.is_some();
        let (done, _) = watch::channel(false);
        let live = Arc::new(LiveRun {
            id: run_key(&row.scope, &row.id),
            scope: row.scope.clone(),
            rid: row.id.clone(),
            name: row.name.clone(),
            def: def.clone(),
            visible: row.visible,
            interactive,
            launched_at_ms: self.inner.deps.clock.now_ms(),
            pid,
            redact: redact.clone(),
            writer: Mutex::new(self.writer_for(row, redact.clone())),
            control: Mutex::new(Control::new()),
            stdin: tokio::sync::Mutex::new(stdin),
            force: Notify::new(),
            release: Notify::new(),
            done,
        });
        state.live.insert(live.id.clone(), Arc::clone(&live));
        let started = self.set_state(
            &row.scope,
            &row.id,
            &RunPatch {
                status: Some(RunStatus::Running),
                pid: Some(pid),
                started_at: Some(Some(self.now_iso())),
                stdin: Some(has_stdin),
                ..RunPatch::default()
            },
        );
        live.sys(&format!(
            "started by {} ({}{}): {}",
            row.by,
            row.origin.as_str(),
            if row.visible { ", visible" } else { "" },
            redact.redact(&def.cmd)
        ));
        tokio::spawn(supervise(
            self.clone(),
            live,
            ProcessIo {
                stdout,
                stderr,
                exit,
            },
        ));
        started.unwrap_or_else(|| row.clone())
    }
}
