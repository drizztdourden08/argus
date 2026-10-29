//! `RunService` (DESIGN.md section 5): starts the processes a lead defined, keeps their
//! lifecycle in the run store and as run docs, captures their logs, stops them as a tree,
//! and fails honestly what a restart interrupted. Only names reach this service: the command
//! line, cwd and env always come from the definitions.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tokio::task::JoinHandle;

use crate::def::RunDef;
use crate::defs_source::DefsSource;
use crate::events::RunEvents;
use crate::host::ProcessHost;
use crate::live_run::LiveRun;
use crate::log_writer::{LogContext, LogWriter};
use crate::record::{RunLimits, RunPatch, RunRow};
use crate::redact::{Redactor, secret_env_values};
use crate::store::Storage;
use crate::time::format_iso;
use argus_core::clock::Clock;

/// A listener of a run's end; it only watches.
pub(crate) type EndListener = Box<dyn Fn(&RunRow) + Send + Sync>;

/// A listener of a run whose output is complete but whose end is not yet reported. It gets the
/// service, so it can read the log.
pub(crate) type FinishListener<H> = Box<dyn Fn(&RunService<H>, &RunRow) + Send + Sync>;

#[derive(Clone, Debug)]
pub struct RunConfig {
    pub limits: RunLimits,
    /// The dashboard token, scrubbed from every log.
    pub token: Option<String>,
    pub attachments_dir: PathBuf,
    pub attach_max_bytes: usize,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            limits: RunLimits::default(),
            token: None,
            attachments_dir: PathBuf::from("attachments"),
            attach_max_bytes: 10 * 1024 * 1024,
        }
    }
}

impl RunConfig {
    /// A source that always answers this config.
    #[must_use]
    pub fn fixed(self) -> ConfigSource {
        Arc::new(move || self.clone())
    }
}

/// Where the service reads its settings: asked on every call, so a change to the dashboard
/// config applies to the next run and the next attachment without a restart.
pub type ConfigSource = Arc<dyn Fn() -> RunConfig + Send + Sync>;

/// What a run service is built from.
pub struct RunDeps<H> {
    pub host: H,
    pub store: Arc<dyn Storage>,
    pub events: Arc<dyn RunEvents>,
    pub defs: Arc<dyn DefsSource>,
    pub config: ConfigSource,
    pub clock: Arc<dyn Clock>,
}

#[derive(Default)]
pub(crate) struct State {
    pub live: HashMap<String, Arc<LiveRun>>,
    pub waiting: HashMap<String, RunDef>,
    pub closing: bool,
    pub sweeper: Option<JoinHandle<()>>,
}

pub(crate) struct Inner<H: ProcessHost> {
    pub deps: RunDeps<H>,
    pub state: Mutex<State>,
    pub listeners: Mutex<Vec<(u64, EndListener)>>,
    pub finish_listeners: Mutex<Vec<FinishListener<H>>>,
    pub next_listener: AtomicU64,
}

pub struct RunService<H: ProcessHost> {
    pub(crate) inner: Arc<Inner<H>>,
}

impl<H: ProcessHost> Clone for RunService<H> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// The id of a run in the live and waiting maps: `<scope>:<rid>`.
pub(crate) fn run_key(scope: &str, rid: &str) -> String {
    format!("{scope}:{rid}")
}

impl<H: ProcessHost> RunService<H> {
    pub fn new(deps: RunDeps<H>) -> Self {
        Self {
            inner: Arc::new(Inner {
                deps,
                state: Mutex::new(State::default()),
                listeners: Mutex::new(Vec::new()),
                finish_listeners: Mutex::new(Vec::new()),
                next_listener: AtomicU64::new(1),
            }),
        }
    }

    pub(crate) fn state(&self) -> MutexGuard<'_, State> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn now_iso(&self) -> String {
        format_iso(self.inner.deps.clock.now_ms())
    }

    /// The settings as they are right now.
    pub(crate) fn config(&self) -> RunConfig {
        (self.inner.deps.config)()
    }

    pub(crate) fn limits(&self) -> RunLimits {
        self.config().limits
    }

    pub(crate) fn redactor_for(&self, def: Option<&RunDef>) -> Redactor {
        let mut secrets: Vec<String> = self.config().token.into_iter().collect();
        if let Some(def) = def {
            secrets.extend(secret_env_values(&def.env));
        }
        Redactor::new(&secrets)
    }

    pub(crate) fn writer_for(&self, row: &RunRow, redact: Redactor) -> LogWriter {
        LogWriter::new(
            LogContext {
                store: Arc::clone(&self.inner.deps.store),
                events: Arc::clone(&self.inner.deps.events),
                clock: Arc::clone(&self.inner.deps.clock),
                scope: row.scope.clone(),
                rid: row.id.clone(),
                limits: self.limits(),
                redact,
            },
            row,
        )
    }

    /// A status change: the store update and the run doc together.
    pub(crate) fn set_state(&self, scope: &str, rid: &str, patch: &RunPatch) -> Option<RunRow> {
        let row = self.inner.deps.store.update(scope, rid, patch)?;
        self.inner.deps.events.project(&row);
        Some(row)
    }

    /// Listeners of a run's end (the wake service tells the lead when one failed). They only
    /// watch: a listener that panics is ignored. The returned id removes the listener.
    pub fn on_end(&self, listener: impl Fn(&RunRow) + Send + Sync + 'static) -> u64 {
        let id = self.inner.next_listener.fetch_add(1, Ordering::SeqCst);
        self.inner
            .listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((id, Box::new(listener)));
        id
    }

    pub fn off_end(&self, id: u64) {
        self.inner
            .listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|(listener, _)| *listener != id);
    }

    pub(crate) fn notify_ended(&self, row: &RunRow) {
        let listeners = self
            .inner
            .listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        for (_, listener) in listeners.iter() {
            self.guarded("run end listener failed", row, || listener(row));
        }
    }

    /// Listeners of a run whose output is complete and whose end is not yet reported: the run
    /// is still live in its record and its log is whole, so a listener sees every line before
    /// anyone is told the run ended. They only watch, and are never removed.
    pub fn on_finish(&self, listener: impl Fn(&Self, &RunRow) + Send + Sync + 'static) {
        self.inner
            .finish_listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Box::new(listener));
    }

    pub(crate) fn notify_finishing(&self, row: &RunRow) {
        let listeners = self
            .inner
            .finish_listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        for listener in listeners.iter() {
            self.guarded("run finish listener failed", row, || listener(self, row));
        }
    }

    /// Runs a listener; one that panics is warned about and ignored.
    fn guarded(&self, warning: &str, row: &RunRow, call: impl FnOnce()) {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(call));
        if outcome.is_err() {
            let key = run_key(&row.scope, &row.id);
            self.inner.deps.events.warn(warning, &key);
        }
    }

    /// The number of runs with a process right now.
    pub fn live_count(&self) -> usize {
        self.state().live.len()
    }
}
