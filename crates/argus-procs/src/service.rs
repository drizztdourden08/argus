//! The process monitor service: the one object the HTTP server mounts.

use crate::alerts::AlertBook;
use crate::config::ProcsConfig;
use crate::defaults::PROTECTED_NAMES;
use crate::error::ProcsError;
use crate::killing::{KillOutcome, KillRequest, KillRuleOutcome, KillRuleRequest};
use crate::matcher::CompiledRule;
use crate::ports::{Killer, ProcsHost, ProcsStore, Responder, Sampler};
use crate::rule::Rule;
use crate::rulebook::{RuleBook, RuleChange};
use crate::timer::Timer;
use crate::tracker::Tracker;
use crate::types::{Meta, ProcDoc, Sample};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The collaborators the monitor works through.
#[derive(Clone)]
pub struct Ports {
    pub sampler: Arc<dyn Sampler>,
    pub responder: Arc<dyn Responder>,
    pub killer: Arc<dyn Killer>,
    pub store: Arc<dyn ProcsStore>,
    pub host: Arc<dyn ProcsHost>,
}

/// How the monitor runs.
#[derive(Debug, Clone)]
pub struct Options {
    pub config: ProcsConfig,
    pub rules_file: PathBuf,
    pub own_pid: u32,
    pub system_root: Option<String>,
    pub timer: bool,
}

/// The rules, the file they live in and the always-protected names.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RulesView {
    pub rules: Vec<Rule>,
    pub file: String,
    pub error: Option<String>,
    pub protected_names: Vec<String>,
}

/// The rules as one sample sees them.
#[derive(Debug, Clone)]
pub(crate) struct RulesSnapshot {
    pub rules: Vec<Rule>,
    pub compiled: Vec<CompiledRule>,
    pub error: Option<String>,
    pub file: String,
}

/// What a sample leaves behind for the next one. Samples and kills hold it, so they run one at a time.
#[derive(Debug, Default)]
pub(crate) struct Engine {
    pub tracker: Tracker,
    pub alerts: AlertBook,
    pub last_error: Option<String>,
}

/// What readers see.
#[derive(Debug, Default)]
pub(crate) struct View {
    pub listing: Vec<ProcDoc>,
    pub raw: Option<Sample>,
    pub meta: Option<Meta>,
}

pub(crate) struct Inner {
    pub ports: Ports,
    pub options: Options,
    pub engine: Mutex<Engine>,
    pub book: Mutex<RuleBook>,
    pub view: Mutex<View>,
}

pub(crate) fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Inner {
    /// Loads the rules file when it changed and reports what that cost.
    pub(crate) fn rules_snapshot(&self) -> RulesSnapshot {
        let mut book = locked(&self.book);
        if let Some(warning) = book.load() {
            self.ports.host.warn(warning.message, &warning.detail);
        }
        RulesSnapshot {
            rules: book.rules().to_vec(),
            compiled: book.compiled().to_vec(),
            error: book.error().map(ToString::to_string),
            file: book.file().display().to_string(),
        }
    }
}

/// The process monitor: samples on a timer, lists what the rules take, wakes the Overlord, kills on request.
pub struct ProcsService {
    inner: Arc<Inner>,
    timer: Mutex<Option<Timer>>,
}

impl ProcsService {
    /// A monitor that has not started; `start` begins the timer.
    #[must_use]
    pub fn new(ports: Ports, options: Options) -> Self {
        let book = RuleBook::new(options.rules_file.clone());
        let inner = Inner {
            ports,
            options,
            engine: Mutex::new(Engine::default()),
            book: Mutex::new(book),
            view: Mutex::new(View::default()),
        };
        Self {
            inner: Arc::new(inner),
            timer: Mutex::new(None),
        }
    }

    /// Loads the rules and, unless the timer is off, samples after the start delay and then every `sample_ms`.
    pub fn start(&self) {
        let mut timer = locked(&self.timer);
        if timer.is_some() {
            return;
        }
        self.inner.rules_snapshot();
        let config = self.inner.options.config;
        if self.inner.options.timer {
            let inner = Arc::clone(&self.inner);
            *timer = Some(Timer::start(
                config.start_delay_ms,
                config.sample_ms,
                move || {
                    inner.sample_serial();
                },
            ));
        }
    }

    /// Stops the timer; a sample in progress finishes.
    pub fn stop(&self) {
        if let Some(timer) = locked(&self.timer).take() {
            timer.stop();
        }
    }

    /// One sample now, its docs and its alert.
    #[must_use]
    pub fn sample_now(&self) -> Meta {
        self.inner.sample_serial()
    }

    /// The listed processes, the biggest memory first; only one rule's with `rule`.
    #[must_use]
    pub fn list(&self, rule: Option<&str>) -> Vec<ProcDoc> {
        let view = locked(&self.inner.view);
        let mut docs: Vec<ProcDoc> = view
            .listing
            .iter()
            .filter(|d| rule.is_none_or(|r| d.rule == r))
            .cloned()
            .collect();
        docs.sort_by_key(|d| std::cmp::Reverse(d.mem_mb));
        docs
    }

    /// The summary of the last sample; none before the first.
    #[must_use]
    pub fn meta(&self) -> Option<Meta> {
        locked(&self.inner.view).meta.clone()
    }

    /// The last raw sample, for the worktree removal's "is the folder in use" check.
    #[must_use]
    pub fn snapshot(&self) -> Option<Sample> {
        locked(&self.inner.view).raw.clone()
    }

    /// Writes the docs again, after a hard resync wiped the scope.
    pub fn republish(&self) {
        self.inner.publish();
    }

    /// The rules in force.
    #[must_use]
    pub fn rules(&self) -> RulesView {
        let snapshot = self.inner.rules_snapshot();
        RulesView {
            rules: snapshot.rules,
            file: snapshot.file,
            error: snapshot.error,
            protected_names: PROTECTED_NAMES.iter().map(ToString::to_string).collect(),
        }
    }

    /// Creates or changes one rule; `null` (or `""`) for a field removes it.
    ///
    /// # Errors
    /// `bad_request` for an invalid rule, `internal` when the file cannot be written.
    pub fn set_rule(&self, input: &Value) -> Result<RuleChange, ProcsError> {
        self.inner.rules_snapshot();
        locked(&self.inner.book).set_rule(input)
    }

    /// Deletes a rule by name; `false` when there is none of that name.
    ///
    /// # Errors
    /// `internal` when the file cannot be written.
    pub fn remove_rule(&self, name: &str) -> Result<bool, ProcsError> {
        self.inner.rules_snapshot();
        locked(&self.inner.book).remove_rule(name)
    }

    /// Ends one listed, unprotected process of a killable rule.
    ///
    /// # Errors
    /// `bad_request`, `not_found`, `conflict` or `forbidden`, each with the reason.
    pub fn kill(&self, request: &KillRequest) -> Result<KillOutcome, ProcsError> {
        self.inner.kill_one(request)
    }

    /// Ends every killable, unprotected process of a rule.
    ///
    /// # Errors
    /// `not_found` for an unknown rule.
    pub fn kill_rule(&self, request: &KillRuleRequest) -> Result<KillRuleOutcome, ProcsError> {
        self.inner.kill_rule(request)
    }

    /// Milliseconds between samples.
    #[must_use]
    pub fn sample_ms(&self) -> u64 {
        self.inner.options.config.sample_ms
    }

    /// Milliseconds before a sent alert is due again.
    #[must_use]
    pub fn repeat_ms(&self) -> u64 {
        self.inner.options.config.repeat_ms
    }
}

impl Drop for ProcsService {
    fn drop(&mut self) {
        self.stop();
    }
}
