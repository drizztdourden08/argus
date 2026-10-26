//! Starting a run: the checks, the visible rule, the parallel limit (`already`) and the
//! queue.

use crate::def::RunDef;
use crate::error::RunError;
use crate::host::ProcessHost;
use crate::names::{check_agent, check_key, check_name};
use crate::record::{NewRun, Origin, RunRow, RunStatus};
use crate::run_service::{RunService, run_key};

const REASON_MAX: usize = 120;
const BY_MAX: usize = 60;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StartOptions {
    pub origin: Origin,
    pub by: Option<String>,
    pub agent: Option<String>,
    pub reason: Option<String>,
    /// The page's wish (`false` keeps a handed build hidden) or the CLI's `--visible`.
    pub visible: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartOutcome {
    pub run: RunRow,
    /// True when the definition was at its parallel limit and the live run is returned.
    pub already: bool,
}

/// Visible means a console window: only a handed build whose definition asks for one, and
/// only from the page, or the CLI with `--visible`. MCP launches are always hidden.
pub(crate) fn visible_for(
    def: &RunDef,
    origin: Origin,
    wanted: Option<bool>,
) -> Result<bool, RunError> {
    let eligible = def.hand && def.visible;
    match origin {
        Origin::Page => Ok(wanted != Some(false) && eligible),
        Origin::Cli if wanted == Some(true) => {
            if eligible {
                Ok(true)
            } else {
                Err(RunError::bad_request(format!(
                    "--visible is only for a launch defined with --hand --visible; \"{}\" is not",
                    def.name
                )))
            }
        }
        Origin::Cli | Origin::Mcp => Ok(false),
    }
}

fn reason_of(reason: Option<&str>) -> Option<String> {
    let clipped: String = reason?.trim().chars().take(REASON_MAX).collect();
    (!clipped.is_empty()).then_some(clipped)
}

fn by_of(options: &StartOptions) -> String {
    let by = options.by.clone().unwrap_or_else(|| match options.origin {
        Origin::Page => "human".to_owned(),
        other => other.as_str().to_owned(),
    });
    by.chars().take(BY_MAX).collect()
}

impl<H: ProcessHost> RunService<H> {
    /// Starts the launch `name` of project `key`.
    pub fn start(
        &self,
        key: &str,
        name: &str,
        options: &StartOptions,
    ) -> Result<StartOutcome, RunError> {
        check_key(key)?;
        check_name(name)?;
        if self.state().closing {
            return Err(RunError::unavailable("the dashboard is shutting down"));
        }
        let def = self
            .inner
            .deps.defs
            .defs(key)
            .into_iter()
            .find(|d| d.name == name)
            .ok_or_else(|| {
                RunError::unknown_run(format!(
                    "no launch \"{name}\" is defined in {key} (a lead defines it with coord run define)"
                ))
            })?;
        if let Some(agent) = &options.agent {
            check_agent(agent)?;
        }
        let visible = visible_for(&def, options.origin, options.visible)?;
        let admitted = self.admit(key, &def, options, visible);
        if let Some(message) = admitted.failure {
            self.fail_now(&admitted.outcome.run, &message, RunStatus::Failed, None);
            let run = self
                .inner
                .deps
                .store
                .get(key, &admitted.outcome.run.id)
                .unwrap_or(admitted.outcome.run);
            return Ok(StartOutcome {
                run,
                already: false,
            });
        }
        Ok(admitted.outcome)
    }

    /// Everything from the parallel check to the insert happens under the state lock, so
    /// two Play clicks cannot both pass. A launch that cannot start is reported for the
    /// caller to fail once the lock is released.
    fn admit(&self, key: &str, def: &RunDef, options: &StartOptions, visible: bool) -> Admitted {
        let mut state = self.state();
        let running = self.inner.deps.store.live_by_name(key, &def.name);
        if running.len() >= usize::try_from(def.parallel).unwrap_or(1) {
            let run = running.into_iter().next().expect("a live run was counted");
            return Admitted::already(run);
        }
        let rid = format!("R-{}", self.inner.deps.store.next_number(key));
        let redact = self.redactor_for(Some(def));
        let row = self.inner.deps.store.insert(NewRun {
            scope: key.to_owned(),
            rid: rid.clone(),
            name: def.name.clone(),
            kind: def.kind,
            cmd: redact.redact(&def.cmd),
            cwd: def.cwd.clone(),
            visible,
            origin: options.origin,
            by: by_of(options),
            agent: options.agent.clone(),
            reason: reason_of(options.reason.as_deref()),
            queued_at: self.now_iso(),
        });
        self.inner.deps.events.project(&row);
        if state.live.len() >= self.limits().max_concurrent {
            state.waiting.insert(run_key(key, &rid), def.clone());
            return Admitted::started(row, None);
        }
        let failure = self.try_launch(&mut state, &row, def).err();
        drop(state);
        let run = self.inner.deps.store.get(key, &rid).unwrap_or(row);
        Admitted::started(run, failure)
    }
}

struct Admitted {
    outcome: StartOutcome,
    failure: Option<String>,
}

impl Admitted {
    fn already(run: RunRow) -> Self {
        Self {
            outcome: StartOutcome { run, already: true },
            failure: None,
        }
    }

    fn started(run: RunRow, failure: Option<String>) -> Self {
        Self {
            outcome: StartOutcome {
                run,
                already: false,
            },
            failure,
        }
    }
}
