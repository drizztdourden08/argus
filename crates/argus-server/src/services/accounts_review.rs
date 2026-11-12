//! The sync review of the accounts service: the review rows, the cleanup candidates and the
//! run of the owner's choices and edits. A run holds the sync turn, closes Claude Desktop
//! politely when it runs (and starts it again after, also when the run failed), then writes
//! with the sync job's backup and verify. The `run` doc follows its phases and the `SyncSteps`
//! docs its steps; the drift and the chat counts are scanned again when it ends.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use argus_accounts::desktop::ClaudeDesktop;
use argus_accounts::desktop::app::wait_until;
use argus_accounts::find_app_data;
use argus_accounts::sync::review::{Cleanup, Request, Review, build_review, cleanup, run_review};
use argus_accounts::sync::{Progress, Summary, scan_and_plan};
use argus_accounts::{DesktopApp, QUIT_BY_HAND, Quit};
use argus_core::clock::now_ms;
use serde_json::{Value, json};

use super::AccountsService;
use super::accounts_switch;
use super::accounts_sync::{OwnedTurn, merge};

/// What a review answers while a switch runs; the route turns it into a 409.
pub const SWITCH_ACTIVE: &str = "a Desktop switch is running";
/// The age limit of the cleanup when none is asked.
pub const CLEANUP_DAYS: u32 = 30;
/// How long a run waits for Claude's storage lock to come free after the close.
const LOCK_WAIT: Duration = Duration::from_secs(20);
const LOCK_POLL: Duration = Duration::from_millis(250);

/// Set by [`cancel`] while a run waits for the owner to quit Desktop.
static CANCELLED: AtomicBool = AtomicBool::new(false);
/// Whether a run waits for the owner to quit Desktop from its tray.
static WAITING: AtomicBool = AtomicBool::new(false);

/// Cancels a review run that waits for the owner to quit Desktop. True when it was stopped;
/// false when no run waits (a run that is writing is never cut short).
pub fn cancel() -> bool {
    let waiting = WAITING.load(Ordering::SeqCst);
    if waiting {
        CANCELLED.store(true, Ordering::SeqCst);
    }
    waiting
}

impl AccountsService {
    /// The rows of the sync review, with a choice per difference.
    ///
    /// # Errors
    /// When the chats or the sidebar cannot be read.
    pub fn review(&self) -> Result<Review, String> {
        let accounts = self.config().accounts;
        self.with_env(|env| scan_and_plan(env).map(|p| build_review(&p, &accounts)))?
    }

    /// The archived chats of each account untouched for more than `days` days.
    ///
    /// # Errors
    /// When the chats or the sidebar cannot be read.
    pub fn cleanup(&self, days: u32) -> Result<Cleanup, String> {
        self.with_env(|env| {
            argus_accounts::sync::job::scan(env)
                .map(|s| cleanup(&s, env.accounts, env.clock.now_ms(), days))
        })?
    }
}

/// A review run that holds the turn, ready to go.
pub struct ReviewRun {
    id: String,
    accounts: Arc<AccountsService>,
    turn: OwnedTurn,
    request: Request,
    close_desktop: bool,
}

/// Takes the turn for a review run.
///
/// # Errors
/// [`SWITCH_ACTIVE`] while a switch runs, `SYNC_BUSY` while a sync or a review runs.
pub fn prepare(
    accounts: &Arc<AccountsService>,
    request: Request,
    close_desktop: bool,
) -> Result<ReviewRun, String> {
    if accounts_switch::is_running() {
        return Err(SWITCH_ACTIVE.to_owned());
    }
    let turn = accounts.take_owned()?;
    Ok(ReviewRun {
        id: format!("RV-{}", now_ms()),
        accounts: Arc::clone(accounts),
        turn,
        request,
        close_desktop,
    })
}

impl ReviewRun {
    /// The run's id, as the `run` doc names it.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Runs the review. Blocking.
    ///
    /// # Errors
    /// Why the run failed; the `run` doc says it too.
    pub fn run(self) -> Result<Summary, String> {
        let ReviewRun {
            id,
            accounts,
            turn,
            request,
            close_desktop,
        } = self;
        let doc = RefCell::new(json!({
            "id": id, "kind": "review", "phase": "scan", "step": 0, "total": 0,
            "backupDir": null, "startedAt": now_ms(), "finishedAt": null, "result": null,
            "desktop": null,
        }));
        let update = |fields: Value| {
            let mut doc = doc.borrow_mut();
            set(&mut doc, fields);
            accounts.put_run(&doc);
        };
        update(json!({}));
        let result = match find_app_data(accounts.config().app_data.as_deref()) {
            Some(app) => {
                let desktop = ClaudeDesktop::new(&app);
                let mut note = |state: &str| match state {
                    "waiting_for_quit" => update(
                        json!({ "desktop": state, "phase": "waiting_for_quit", "detail": QUIT_BY_HAND }),
                    ),
                    "closed" => {
                        update(json!({ "desktop": state, "phase": "scan", "detail": null }));
                    }
                    _ => update(json!({ "desktop": state })),
                };
                let work = || write(&accounts, &request, &update);
                let cancelled = || CANCELLED.load(Ordering::SeqCst);
                let quit = (
                    accounts.config().force_quit,
                    &cancelled as &dyn Fn() -> bool,
                );
                CANCELLED.store(false, Ordering::SeqCst);
                let closed = with_desktop_closed(&desktop, close_desktop, quit, &mut note, work);
                WAITING.store(false, Ordering::SeqCst);
                closed
            }
            None => Err("the Claude Desktop data folder was not found".to_owned()),
        };
        let end = match &result {
            Ok(summary) => json!(summary),
            Err(e) => json!({ "error": e }),
        };
        let phase = if result.is_ok() { "done" } else { "failed" };
        update(json!({ "phase": phase, "finishedAt": now_ms(), "result": end }));
        drop(turn);
        accounts.scan_drift();
        result
    }
}

/// The review itself, over the sync job's environment; `update` puts each progress into the
/// `run` doc.
fn write(
    accounts: &AccountsService,
    request: &Request,
    update: &dyn Fn(Value),
) -> Result<Summary, String> {
    accounts.with_env(|env| {
        let mut on_progress = |p: &Progress| update(serde_json::to_value(p).unwrap_or_default());
        run_review(env, request, &accounts.docs, &mut on_progress)
    })?
}

fn set(doc: &mut Value, fields: Value) {
    if let Value::Object(map) = fields {
        merge(doc, map);
    }
}

/// Runs `work` with Claude Desktop closed. When Desktop runs it is closed politely (refused
/// without `close`), the storage lock must come free, and it is started again after `work`,
/// whatever `work` answered. `note` hears `closing`, `waiting_for_quit`, `closed`, `starting`, `started` or why the
/// start failed.
///
/// # Errors
/// When Desktop runs and may not be closed, does not close, keeps its lock, or `work` fails.
pub(crate) fn with_desktop_closed<T>(
    desktop: &dyn DesktopApp,
    close: bool,
    (force, cancelled): (bool, &dyn Fn() -> bool),
    note: &mut dyn FnMut(&str),
    work: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let was_running = desktop.is_running();
    if was_running {
        if !close {
            return Err("Claude Desktop is running; close it or let Argus close it".to_owned());
        }
        note("closing");
        let mut waiting = |_: &str| {
            WAITING.store(true, Ordering::SeqCst);
            note("waiting_for_quit");
        };
        desktop.close(&mut Quit {
            force,
            cancelled,
            waiting: &mut waiting,
        })?;
        WAITING.store(false, Ordering::SeqCst);
        if !wait_until(LOCK_WAIT, LOCK_POLL, || !desktop.is_running()) {
            return Err("Claude Desktop still holds its storage; nothing was written".to_owned());
        }
        note("closed");
    }
    let result = work();
    if was_running {
        note("starting");
        match desktop.start() {
            Ok(()) => note("started"),
            Err(e) => note(&format!("Claude did not start again: {e}")),
        }
    }
    result
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    const NO_QUIT: (bool, &dyn Fn() -> bool) = (false, &|| false);

    #[derive(Default)]
    struct Fake {
        running: AtomicBool,
        calls: Mutex<Vec<&'static str>>,
        refuse_close: bool,
    }

    impl Fake {
        fn log(&self, what: &'static str) {
            self.calls.lock().unwrap().push(what);
        }
    }

    impl DesktopApp for Fake {
        fn is_running(&self) -> bool {
            self.running.load(Ordering::SeqCst)
        }
        fn turn_in_progress(&self) -> bool {
            false
        }
        fn close(&self, _: &mut Quit) -> Result<(), String> {
            self.log("close");
            if self.refuse_close {
                return Err("still open".to_owned());
            }
            self.running.store(false, Ordering::SeqCst);
            Ok(())
        }
        fn start(&self) -> Result<(), String> {
            self.log("start");
            self.running.store(true, Ordering::SeqCst);
            Ok(())
        }
        fn config_touched_since(&self, _since_ms: i64) -> bool {
            false
        }
        fn version(&self) -> Option<String> {
            None
        }
    }

    fn running() -> Fake {
        let fake = Fake::default();
        fake.running.store(true, Ordering::SeqCst);
        fake
    }

    #[test]
    fn a_running_desktop_is_closed_for_the_work_and_started_again() {
        let fake = running();
        let mut notes = Vec::new();
        let seen = with_desktop_closed(
            &fake,
            true,
            NO_QUIT,
            &mut |n| notes.push(n.to_owned()),
            || Ok::<bool, String>(fake.is_running()),
        );
        assert_eq!(seen, Ok(false), "the work ran with Desktop closed");
        assert_eq!(*fake.calls.lock().unwrap(), ["close", "start"]);
        assert_eq!(notes, ["closing", "closed", "starting", "started"]);
    }

    #[test]
    fn desktop_starts_again_when_the_work_fails() {
        let fake = running();
        let result = with_desktop_closed(&fake, true, NO_QUIT, &mut |_| {}, || {
            Err::<(), _>("boom".to_owned())
        });
        assert_eq!(result, Err("boom".to_owned()));
        assert!(fake.is_running());
    }

    #[test]
    fn a_closed_desktop_is_neither_closed_nor_started() {
        let fake = Fake::default();
        let result = with_desktop_closed(&fake, true, NO_QUIT, &mut |_| {}, || Ok::<u8, String>(1));
        assert_eq!(result, Ok(1));
        assert!(fake.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn without_leave_to_close_a_running_desktop_nothing_runs() {
        let fake = running();
        let mut ran = false;
        let result = with_desktop_closed(&fake, false, NO_QUIT, &mut |_| {}, || {
            ran = true;
            Ok::<(), String>(())
        });
        assert!(result.is_err() && !ran);
        assert!(fake.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn a_close_that_fails_writes_nothing() {
        let fake = Fake {
            refuse_close: true,
            ..running()
        };
        let mut ran = false;
        let result = with_desktop_closed(&fake, true, NO_QUIT, &mut |_| {}, || {
            ran = true;
            Ok::<(), String>(())
        });
        assert_eq!(result, Err("still open".to_owned()));
        assert!(!ran);
    }
}
