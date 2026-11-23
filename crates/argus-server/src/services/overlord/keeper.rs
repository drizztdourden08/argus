//! The keeper of the hosted Overlord (I-196): it is on as long as the mode is live and a key is
//! saved, with nothing to start by hand. Every 30 seconds, when the mode is live, the key file
//! exists, no set budget is spent and no child runs, it starts one under the control lock. After a
//! failed start or a child that ended it waits 30 s, 60 s, 2 min, then 5 min at most (the schedule
//! is `argus_overlord::keeper`). A saved key kicks it at once. One activity line says how each
//! start attempt ended, never one per tick; a pass that has nothing to do writes nothing.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use argus_overlord::keeper::{Backoff, TICK};
use argus_overlord::mode::Mode;
use argus_store::{ActivityLevel, NewActivity};
use tokio::sync::Notify;

use super::config;
use super::live::{StartFault, budget_remaining, try_start};
use crate::Services;
use crate::adapters::overlord::key::DpapiKeyStore;
use crate::services::tasks::Timer;

const TASK: &str = "overlord-keeper";
const KIND: &str = "overlord";

/// What the keeper remembers between passes.
#[derive(Default)]
struct Memory {
    backoff: Backoff,
    /// Why the last start did not happen or why the child ended.
    last: Option<String>,
}

/// The keeper's memory and its wake-up, kept with the running supervisor.
#[derive(Default)]
pub(crate) struct Keeper {
    memory: Mutex<Memory>,
    kicked: Notify,
    /// Set while the timer runs, so the status can tell a retry is coming.
    watching: AtomicBool,
}

/// Seconds on a clock that only moves forward, from the first call.
fn clock() -> Duration {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed()
}

impl Keeper {
    fn memory(&self) -> MutexGuard<'_, Memory> {
        self.memory.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Starts now: forgets every wait and wakes the timer, as a saved key does.
    pub(crate) fn kick(&self) {
        let mut memory = self.memory();
        memory.backoff.reset();
        memory.last = None;
        drop(memory);
        self.kicked.notify_one();
    }

    /// Notes what the child is doing; true when a start may be tried now.
    fn observe(&self, up: bool) -> bool {
        let now = clock();
        let mut memory = self.memory();
        if up {
            memory.backoff.started(now);
            return false;
        }
        if memory.backoff.tracking() {
            memory.backoff.ended(now);
            memory.last = Some("the hosted Overlord ended".to_owned());
        }
        memory.backoff.may_try(now)
    }

    /// Notes how a start ended.
    fn settle(&self, result: Result<(), StartFault>) {
        let now = clock();
        let mut memory = self.memory();
        match result {
            Ok(()) | Err(StartFault::Running) => {
                memory.backoff.started(now);
                memory.last = None;
            }
            Err(fault) => {
                memory.backoff.ended(now);
                memory.last = Some(fault.sentence().to_owned());
            }
        }
    }

    /// The last reason with the time to the next try.
    fn waiting(&self) -> String {
        let memory = self.memory();
        let Some(last) = memory.last.as_deref() else {
            return "the hosted Overlord has not started yet".to_owned();
        };
        let secs = memory.backoff.retry_in_secs(clock()).unwrap_or(0);
        let retry = if self.watching.load(Ordering::SeqCst) {
            format!("retrying in {secs} s")
        } else {
            "the keeper is off".to_owned()
        };
        format!("{last}, {retry}")
    }
}

/// True when the key file exists. The key itself is never read here.
pub(crate) fn key_saved(svc: &Services) -> bool {
    DpapiKeyStore::new(&svc.paths.home).saved_at().is_some()
}

/// True only when a monthly budget is set and used up; no budget never blocks.
fn budget_spent(svc: &Services) -> bool {
    config::budget(svc)
        .budget_usd
        .is_some_and(|usd| budget_remaining(&svc.db, usd) <= 0.0)
}

/// Why the child cannot be started now, in one plain sentence; `None` when nothing stands in the
/// way. Blocks on the database.
pub(crate) fn blocker(svc: &Services) -> Option<&'static str> {
    if svc.shadow {
        return Some("this Argus is a shadow run");
    }
    match config::mode(svc) {
        Mode::Off => return Some("the mode is off"),
        Mode::Shadow => return Some("the mode is shadow"),
        Mode::Live => {}
    }
    if !key_saved(svc) {
        Some("no API key is saved")
    } else if svc.wake.is_none() {
        Some("the wake engine is off")
    } else if budget_spent(svc) {
        Some("the monthly budget is spent")
    } else {
        None
    }
}

/// Why no child runs, for the status: `None` while one runs.
pub(crate) fn problem(svc: &Services) -> Option<String> {
    if svc.overlord.up() {
        return None;
    }
    Some(blocker(svc).map_or_else(|| svc.overlord.keeper.waiting(), str::to_owned))
}

fn say(svc: &Services, level: ActivityLevel, text: String) {
    svc.activity.add(NewActivity::new(KIND, level, text));
}

fn report(svc: &Services, result: Result<(), StartFault>) {
    match result {
        Ok(()) => say(
            svc,
            ActivityLevel::Info,
            "overlord started by the keeper".to_owned(),
        ),
        Err(StartFault::Running) => {}
        Err(fault) => say(
            svc,
            ActivityLevel::Warn,
            format!("overlord did not start: {}", fault.sentence()),
        ),
    }
}

/// One pass under the control lock: starts the child when it is due and nothing stands in the way.
async fn pass(svc: &Arc<Services>) {
    let _one = svc.overlord.control.lock().await;
    if !svc.overlord.keeper.observe(svc.overlord.up()) {
        return;
    }
    let held = Arc::clone(svc);
    let blocked = tokio::task::spawn_blocking(move || blocker(&held).is_some()).await;
    if blocked.unwrap_or(true) {
        return;
    }
    let result = try_start(svc);
    svc.overlord.keeper.settle(result);
    report(svc, result);
}

async fn keep_loop(svc: Arc<Services>) {
    loop {
        tokio::select! {
            () = tokio::time::sleep(TICK) => {}
            () = svc.overlord.keeper.kicked.notified() => {}
        }
        pass(&svc).await;
    }
}

/// Starts the timer with the server; a shadow run and a run with timers off keep it down.
pub(crate) fn start(svc: &Arc<Services>) {
    if svc.shadow || !svc.tasks.runs(Timer::Wake) {
        return;
    }
    svc.overlord.keeper.watching.store(true, Ordering::SeqCst);
    svc.tasks
        .keep(TASK, tokio::spawn(keep_loop(Arc::clone(svc))));
}

pub(crate) fn stop(svc: &Services) {
    svc.overlord.keeper.watching.store(false, Ordering::SeqCst);
    svc.tasks.abort(TASK);
}
