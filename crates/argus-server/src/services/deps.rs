//! Cross-board dependencies in the server. The ledger and the two mirrors are written only by
//! `argus_board::deps`; this module keeps them true while the server runs. A board that was
//! exported (whoever wrote it) is reconciled, and every 5 minutes so is every open dependency.
//! A call that made transitions ticks the wake engine, so the other side hears at once.
//!
//! The deps calls lock the board files with the blocking lock, one board at a time, so they run
//! off the async workers. Nothing here runs when `deps.enabled` is false or in shadow mode.

use std::path::Path;
use std::sync::{Arc, Weak};
use std::time::Duration;

use argus_board::deps::{
    DepError, Done, Sweep, Transition, ack as ack_dep, off, reconcile_board, sweep,
};
use argus_core::ids::PORTFOLIO;
use argus_core::{ApiError, ErrorCode};
use argus_wake::service::driver::lock;
use serde_json::json;
use tokio::runtime::Handle;

use super::tasks::Timer;
use crate::Services;

const TASK: &str = "deps";
const SWEEP_EVERY: Duration = Duration::from_secs(300);

/// A refusal of the dependency calls as the server's usual error: 400, 404 and 409 carry the
/// message; any other failure is logged and answered as `internal`.
pub(crate) fn refusal(svc: &Services, e: &DepError) -> ApiError {
    let code = match e.status() {
        400 => ErrorCode::BadRequest,
        404 => ErrorCode::NotFound,
        409 => ErrorCode::Conflict,
        _ => {
            svc.log
                .error("dependency call failed", json!({ "err": e.to_string() }));
            return ApiError::internal();
        }
    };
    ApiError::new(code, e.to_string())
}

/// Whether dependencies are on.
pub(crate) fn enabled(svc: &Services) -> bool {
    svc.config().deps_enabled()
}

/// A write is refused in shadow mode (a preview writes boards).
fn writable(svc: &Services) -> Result<(), ApiError> {
    if !svc.board_writes {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow mode: Argus changes no dependency",
        ));
    }
    Ok(())
}

/// Runs a dependency call on a thread that may block. Answers the off refusal while
/// dependencies are off.
pub(crate) async fn run<T: Send + 'static>(
    svc: &Arc<Services>,
    f: impl FnOnce(&Path) -> Result<T, DepError> + Send + 'static,
) -> Result<T, ApiError> {
    if !enabled(svc) {
        return Err(refusal(svc, &off()));
    }
    let held = Arc::clone(svc);
    let done = tokio::task::spawn_blocking(move || f(&held.paths.coord_home))
        .await
        .map_err(|_| ApiError::internal())?;
    done.map_err(|e| refusal(svc, &e))
}

/// Runs a verb that may make transitions, and ticks the wake engine for them.
pub(crate) async fn act(
    svc: &Arc<Services>,
    f: impl FnOnce(&Path) -> Result<Done, DepError> + Send + 'static,
) -> Result<Done, ApiError> {
    writable(svc)?;
    let held = Arc::clone(svc);
    run(svc, move |home| {
        let done = f(home)?;
        tick_wakes(&held, &done.transitions);
        Ok(done)
    })
    .await
}

/// The dependencies a thread has read in its inbox are acknowledged: those among `ids` that are open, asked
/// of `project`, and not acknowledged yet. Answers the ids acknowledged now. Best effort, as the verb that
/// asks is: off, shadow mode and every failure answer what was done, never a refusal.
pub(crate) async fn ack(
    svc: &Arc<Services>,
    project: String,
    ids: Vec<String>,
    actor: String,
) -> Vec<String> {
    if !svc.board_writes {
        return Vec::new();
    }
    let held = Arc::clone(svc);
    let done = run(svc, move |home| {
        Ok(acked(&held, home, &project, &ids, &actor))
    })
    .await;
    done.unwrap_or_default()
}

fn acked(svc: &Services, home: &Path, project: &str, ids: &[String], actor: &str) -> Vec<String> {
    let mut done = Vec::new();
    for id in ids {
        match ack_dep(home, id, project, actor) {
            Ok(true) => done.push(id.clone()),
            Ok(false) => {}
            Err(e) => svc.log.error(
                "dependency ack failed",
                json!({ "dep": id, "err": e.to_string() }),
            ),
        }
    }
    done
}

/// Lets the wake engine look at the ledger now. Takes the engine's lock: blocking.
fn tick_wakes(svc: &Services, transitions: &[Transition]) {
    if transitions.is_empty() {
        return;
    }
    if let Some(wake) = &svc.wake {
        lock(wake).dep_tick();
    }
}

/// What a background reconcile found: a failure is one log line, transitions a wake tick.
fn settle(svc: &Services, what: &str, done: Result<Sweep, DepError>) {
    match done {
        Ok(found) => {
            for (dep, message) in &found.failed {
                svc.log.error(
                    "dependency reconcile failed",
                    json!({ "what": what, "dep": dep, "err": message }),
                );
            }
            tick_wakes(svc, &found.transitions);
        }
        Err(e) => svc.log.error(
            "dependency reconcile failed",
            json!({ "what": what, "err": e.to_string() }),
        ),
    }
}

fn reconcile_exported(svc: &Services, key: &str) {
    if enabled(svc) && !svc.stop.is_cancelled() {
        settle(svc, key, reconcile_board(&svc.paths.coord_home, key));
    }
}

fn sweep_all(svc: &Services) {
    if enabled(svc) && !svc.stop.is_cancelled() {
        settle(svc, "sweep", sweep(&svc.paths.coord_home));
    }
}

/// Reconciles each exported project board. The listener runs on the exporting thread, so it
/// only hands the key to a blocking task. It holds the services weakly: the sync service lives
/// inside them.
fn listen(svc: &Arc<Services>, handle: Handle) {
    let Some(sync) = &svc.sync else {
        return;
    };
    let me: Weak<Services> = Arc::downgrade(svc);
    sync.on_board(move |key| {
        if key == PORTFOLIO {
            return;
        }
        if let Some(svc) = me.upgrade() {
            let key = key.to_owned();
            handle.spawn_blocking(move || reconcile_exported(&svc, &key));
        }
    });
}

async fn sweep_loop(svc: Arc<Services>) {
    loop {
        tokio::time::sleep(SWEEP_EVERY).await;
        let held = Arc::clone(&svc);
        if tokio::task::spawn_blocking(move || sweep_all(&held))
            .await
            .is_err()
        {
            svc.log.error(
                "dependency sweep failed",
                json!({ "err": "the sweep stopped on a panic" }),
            );
        }
    }
}

pub(crate) fn start(svc: &Arc<Services>) {
    if svc.shadow {
        return;
    }
    listen(svc, Handle::current());
    if svc.tasks.runs(Timer::Deps) {
        svc.tasks
            .keep(TASK, tokio::spawn(sweep_loop(Arc::clone(svc))));
    }
}

pub(crate) fn stop(svc: &Services) {
    svc.tasks.abort(TASK);
}
