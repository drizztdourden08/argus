//! The Desktop switch of the accounts service: one switch at a time, from the Switch now button
//! or an auto-switch rule. It asks the leads for handovers first when told to, then runs the
//! steps of [`argus_accounts::desktop`] and writes the `switch` state doc after every change.
//! `cancel` works until Claude is about to close. The saved Desktop logins live in the vault
//! under `<DASHBOARD_HOME>\accounts\logins`; their state is shown in the `Accounts` docs.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use argus_accounts::desktop::{
    ClaudeDesktop, Done, FsVault, Plan, Stages, Step, Stop, Timing, last_known_account,
};
use argus_accounts::handover::{CycleEvent, CycleMode, StartRequest};
use argus_accounts::{Account, LoginVault, Scope, find_app_data};
use argus_core::Coll;
use argus_core::clock::now_ms;
use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};
use tokio::sync::broadcast::Receiver;

use super::AccountsService;
use super::accounts_handover::{Handovers, use_now};
use crate::Services;

/// What a second switch answers while one runs; the route turns it into a 409.
pub const SWITCH_BUSY: &str = "a switch is already running";
const SWITCH_DOC: &str = "switch";
/// How long the switch waits for the handovers.
const HANDOVER_WAIT: Duration = Duration::from_secs(600);
const HANDOVER_POLL: Duration = Duration::from_millis(250);

/// Who started the switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// The Switch now button.
    Owner,
    /// An auto-switch rule.
    Rule,
}

impl Origin {
    fn name(self) -> &'static str {
        match self {
            Origin::Owner => "owner",
            Origin::Rule => "rule",
        }
    }
}

#[derive(Default)]
struct Gate {
    cancelled: bool,
    closing: bool,
}

/// The cancel and "anyway" switches of the running switch.
#[derive(Clone, Default)]
struct Control {
    gate: Arc<Mutex<Gate>>,
    anyway: Arc<AtomicBool>,
}

impl Control {
    /// The owner cancels; false once Claude is closing.
    fn cancel(&self) -> bool {
        let mut gate = self.gate.lock().unwrap_or_else(PoisonError::into_inner);
        if gate.closing {
            return false;
        }
        gate.cancelled = true;
        true
    }

    fn cancelled(&self) -> bool {
        self.gate
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .cancelled
    }

    /// The runner's last look before it closes Claude: from here on cancel is refused.
    fn last_call(&self) -> bool {
        let mut gate = self.gate.lock().unwrap_or_else(PoisonError::into_inner);
        if !gate.cancelled {
            gate.closing = true;
        }
        gate.cancelled
    }
}

/// The control of the switch that runs now, if one does.
static RUNNING: Mutex<Option<Control>> = Mutex::new(None);

/// Holds the one switch slot; released when dropped.
struct Slot;

impl Slot {
    fn take(control: &Control) -> Result<Slot, String> {
        let mut held = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
        if held.is_some() {
            return Err(SWITCH_BUSY.to_owned());
        }
        *held = Some(control.clone());
        Ok(Slot)
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}

/// Stops waiting for the handovers: the leads that are not ready are cut off and the switch goes
/// on. False when no switch runs.
pub fn request_anyway() -> bool {
    let held = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
    held.as_ref()
        .map(|control| control.anyway.store(true, Ordering::SeqCst))
        .is_some()
}

/// Whether a switch runs now.
pub fn is_running() -> bool {
    RUNNING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_some()
}

/// Cancels the running switch. True when it was stopped; false when none runs or Claude is
/// already closing.
pub fn cancel() -> bool {
    let held = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
    held.as_ref().is_some_and(Control::cancel)
}

/// A switch that passed its checks and holds the slot, ready to run.
pub struct Prepared {
    svc: Arc<Services>,
    accounts: Arc<AccountsService>,
    from: Account,
    to: Account,
    app: PathBuf,
    origin: Origin,
    handover: bool,
    id: String,
    control: Control,
    _slot: Slot,
}

/// Checks a switch to `to` and takes the one switch slot.
///
/// # Errors
/// [`SWITCH_BUSY`], or why the switch cannot start: no such account, it cannot receive, Desktop
/// is on no known account, or it is already on `to`.
pub fn prepare(
    svc: &Arc<Services>,
    to: &str,
    origin: Origin,
    handover: bool,
) -> Result<Prepared, String> {
    let accounts = svc
        .accounts
        .clone()
        .ok_or("the accounts service is not running")?;
    let config = accounts.config();
    let app = find_app_data(config.app_data.as_deref())
        .ok_or("the Claude Desktop data folder was not found")?;
    let target = config
        .accounts
        .iter()
        .find(|a| a.key == to)
        .cloned()
        .ok_or_else(|| format!("no account named {to}"))?;
    if !target.can_receive || target.desktop_account.is_none() || target.desktop_org.is_none() {
        return Err(format!("{} cannot be switched to", target.label));
    }
    let active = last_known_account(&app);
    let from = config
        .accounts
        .iter()
        .find(|a| a.desktop_account.is_some() && a.desktop_account == active)
        .cloned()
        .ok_or("Claude Desktop is not on a known account")?;
    if from.key == target.key {
        return Err(format!("Claude Desktop is already on {}", target.label));
    }
    let control = Control::default();
    let slot = Slot::take(&control)?;
    let id = switch_id(svc);
    Ok(Prepared {
        svc: Arc::clone(svc),
        accounts,
        from,
        to: target,
        app,
        origin,
        handover,
        id,
        control,
        _slot: slot,
    })
}

/// The id of the switch: the one a countdown already shows, else a new one.
fn switch_id(svc: &Services) -> String {
    let shown = svc
        .db
        .docs()
        .get(PORTFOLIO, Coll::AccountsState, SWITCH_DOC)
        .ok()
        .flatten()
        .map(|row| row.data);
    shown
        .filter(|doc| doc["phase"] == "countdown")
        .and_then(|doc| doc["id"].as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("SW-{}", now_ms()))
}

/// Switches Claude Desktop to the account `to`: handovers first when `handover`, then the steps.
/// Returns when the switch ended; the `switch` doc shows how it went.
///
/// # Errors
/// [`SWITCH_BUSY`], a failed check, a failed step (with whether the old account is back), or
/// "the switch was cancelled".
pub async fn switch_to(
    svc: &Arc<Services>,
    to: &str,
    origin: Origin,
    handover: bool,
) -> Result<(), String> {
    prepare(svc, to, origin, handover)?.run().await
}

impl Prepared {
    /// The id of the switch, `SW-n`.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Runs the switch; the slot is held until it ends.
    ///
    /// # Errors
    /// See [`switch_to`].
    pub async fn run(self) -> Result<(), String> {
        let accounts = Arc::clone(&self.accounts);
        let svc = Arc::clone(&self.svc);
        let result = tokio::task::spawn_blocking(move || self.run_blocking())
            .await
            .map_err(|e| e.to_string())?;
        // The active account and the logins changed; show them now rather than at the next poll.
        let _ = accounts.poll_now().await;
        let _ = tokio::task::spawn_blocking(move || publish_logins(&svc, &accounts)).await;
        result
    }

    fn put(&self, phase: &str, steps: &[Step], error: Option<&str>) {
        let doc = switch_doc(
            &self.id,
            (&self.from.key, &self.to.key),
            self.origin,
            phase,
            steps,
        );
        let doc = with_error(doc, error);
        if let Err(e) = self.accounts.docs.put(Coll::AccountsState, SWITCH_DOC, doc) {
            self.svc
                .log
                .error("accounts switch doc not written", json!({ "error": e }));
        }
    }

    fn run_blocking(self) -> Result<(), String> {
        let desktop = ClaudeDesktop::new(&self.app);
        let vault = FsVault::new(&self.app, &logins_root(&self.svc));
        let stages = ServerStages::new(&self);
        let plan = Plan {
            from: &self.from,
            to: &self.to,
            handover: self.handover,
        };
        let mut last = (String::from("idle"), Vec::new());
        let done = argus_accounts::desktop::run(
            &plan,
            &desktop,
            &vault,
            &stages,
            &|| self.control.last_call(),
            Timing::default(),
            &mut |phase, steps| {
                self.put(phase, steps, None);
                last = (phase.to_owned(), steps.to_vec());
            },
        );
        let outcome = outcome_of(&done);
        if let Err(why) = &outcome {
            self.put(&last.0, &last.1, Some(why));
        }
        outcome
    }
}

/// The result a [`Done`] makes, with the reason in plain words.
fn outcome_of(done: &Done) -> Result<(), String> {
    match done {
        Done::Switched => Ok(()),
        Done::Cancelled => Err("the switch was cancelled".to_owned()),
        Done::Failed { reason, rollback } => Err(match rollback {
            Ok(()) => format!("{reason}; Claude was started again as before"),
            Err(why) => format!("{reason}; going back failed too: {why}"),
        }),
    }
}

/// The `switch` state doc.
fn switch_doc(
    id: &str,
    (from, to): (&str, &str),
    origin: Origin,
    phase: &str,
    steps: &[Step],
) -> Value {
    json!({
        "id": id,
        "from": from,
        "to": to,
        "startedBy": origin.name(),
        "phase": phase,
        "countdownEndsAt": null,
        "steps": steps,
        "error": null,
    })
}

fn with_error(mut doc: Value, error: Option<&str>) -> Value {
    if let (Some(text), Some(object)) = (error, doc.as_object_mut()) {
        object.insert("error".to_owned(), json!(text));
    }
    doc
}

/// The stages of one switch that need the server.
struct ServerStages<'a> {
    run: &'a Prepared,
}

impl<'a> ServerStages<'a> {
    fn new(run: &'a Prepared) -> Self {
        ServerStages { run }
    }

    fn wait_ready(
        &self,
        desk: &Handovers,
        events: &mut Receiver<CycleEvent>,
        cycle: &str,
        now_use: Option<f64>,
    ) -> Result<String, Stop> {
        let end = Instant::now() + HANDOVER_WAIT;
        loop {
            if self.run.control.cancelled() {
                let _ = desk.desk().cancel();
                return Err(Stop::Cancelled);
            }
            while let Ok(event) = events.try_recv() {
                if event.cycle == cycle {
                    return Ok(ready_note(&event));
                }
            }
            if self.run.control.anyway.load(Ordering::SeqCst) || Instant::now() >= end {
                let out = desk
                    .cut_off_rest(now_use)
                    .map_err(|e| Stop::Failed(e.to_string()))?;
                return Ok(out
                    .event
                    .as_ref()
                    .map_or_else(|| "cut off the rest".to_owned(), ready_note));
            }
            std::thread::sleep(HANDOVER_POLL);
        }
    }
}

fn ready_note(event: &CycleEvent) -> String {
    if event.cut_off.is_empty() {
        "every handover is ready".to_owned()
    } else {
        format!("{} handover(s) cut off", event.cut_off.len())
    }
}

impl Stages for ServerStages<'_> {
    fn handover(&self) -> Result<String, Stop> {
        let accounts = &self.run.accounts;
        let desk = accounts
            .handovers()
            .ok_or_else(|| Stop::Failed("the handover desk has not started".to_owned()))?;
        let mut events = desk.subscribe();
        let now_use = use_now(accounts, &self.run.from.key);
        let request = StartRequest {
            account: self.run.from.key.clone(),
            mode: CycleMode::Now,
            k: accounts.config().k,
            ask_at: None,
            use_now: now_use,
        };
        let started = desk
            .start(request, accounts)
            .map_err(|e| Stop::Failed(e.to_string()))?;
        // A Now cycle asks at start; asking again is harmless and covers a desk that did not.
        let asked = desk.ask_all(now_use).ok();
        let ready = started.event.or_else(|| asked.and_then(|a| a.event));
        match ready {
            Some(event) => Ok(ready_note(&event)),
            None => self.wait_ready(&desk, &mut events, &started.cycle.id, now_use),
        }
    }

    fn sync(&self) -> Result<String, String> {
        let summary = self.run.accounts.sync(false)?;
        Ok(format!("synced, {} left alone", summary.conflicts))
    }

    fn resume(&self) -> Result<String, String> {
        let (Some(account), Some(org)) = (&self.run.to.desktop_account, &self.run.to.desktop_org)
        else {
            return Err("the target has no Desktop scope".to_owned());
        };
        let scope = Scope {
            account: account.clone(),
            org: org.clone(),
        };
        let task = super::accounts_resume::schedule_for_cycle(&self.run.svc, &scope)?;
        Ok(task.map_or_else(|| "nothing to resume".to_owned(), |id| format!("task {id}")))
    }

    fn active_account(&self) -> Option<String> {
        last_known_account(&self.run.app)
    }
}

/// `<DASHBOARD_HOME>\accounts\logins`.
fn logins_root(svc: &Services) -> PathBuf {
    svc.paths.home.join("accounts").join("logins")
}

/// Saves the login Desktop is on as the login of its account (`key`, when given, must be that
/// account). Blocking. Answers when it was saved.
///
/// # Errors
/// No such active account, a `key` that is not the active account, or why the vault refused.
pub fn save_login(
    svc: &Services,
    accounts: &AccountsService,
    key: Option<&str>,
) -> Result<String, String> {
    let config = accounts.config();
    let app = find_app_data(config.app_data.as_deref())
        .ok_or("the Claude Desktop data folder was not found")?;
    let active = last_known_account(&app);
    let account = config
        .accounts
        .iter()
        .find(|a| a.desktop_account.is_some() && a.desktop_account == active)
        .ok_or("Claude Desktop is not on a known account")?;
    if key.is_some_and(|k| k != account.key) {
        return Err(format!(
            "Claude Desktop is on {}, not on that account",
            account.label
        ));
    }
    let vault = FsVault::new(&app, &logins_root(svc));
    vault.save(account)?;
    publish_logins(svc, accounts);
    vault
        .saved_at(account)
        .ok_or_else(|| "the login was not saved".to_owned())
}

/// Writes `desktopLogin {saved, savedAt}` into every `Accounts` doc from the vault. Blocking.
pub(crate) fn publish_logins(svc: &Services, accounts: &AccountsService) {
    let config = accounts.config();
    let app = find_app_data(config.app_data.as_deref()).unwrap_or_default();
    let vault = FsVault::new(&app, &logins_root(svc));
    for account in &config.accounts {
        let saved_at = vault.saved_at(account);
        if let Err(e) = put_login(svc, accounts, &account.key, saved_at.as_deref()) {
            svc.log
                .error("accounts login state not written", json!({ "error": e }));
        }
    }
}

fn put_login(
    svc: &Services,
    accounts: &AccountsService,
    key: &str,
    saved_at: Option<&str>,
) -> Result<(), String> {
    let row = svc
        .db
        .docs()
        .get(PORTFOLIO, Coll::Accounts, key)
        .map_err(|e| e.to_string())?;
    let Some(mut doc) = row.map(|r| r.data) else {
        return Ok(());
    };
    let Some(object) = doc.as_object_mut() else {
        return Ok(());
    };
    object.insert(
        "desktopLogin".to_owned(),
        json!({ "saved": saved_at.is_some(), "savedAt": saved_at }),
    );
    accounts.docs.put(Coll::Accounts, key, doc)
}

#[cfg(test)]
mod tests {
    use argus_accounts::desktop::{Done, Step, StepStatus};

    use super::{Control, Origin, outcome_of, switch_doc, with_error};

    #[test]
    fn cancel_works_until_the_last_call() {
        let control = Control::default();
        assert!(control.cancel());
        assert!(control.last_call());
        assert!(control.cancelled());
    }

    #[test]
    fn cancel_is_refused_once_claude_is_closing() {
        let control = Control::default();
        assert!(!control.last_call());
        assert!(!control.cancel());
        assert!(!control.cancelled());
    }

    #[test]
    fn the_doc_carries_the_contract_fields() {
        let steps = vec![Step {
            id: "close",
            label: "Close Claude".to_owned(),
            status: StepStatus::Running,
            detail: String::new(),
        }];
        let doc = switch_doc("SW-3", ("a", "b"), Origin::Rule, "closing", &steps);
        assert_eq!(doc["startedBy"], "rule");
        assert_eq!(doc["phase"], "closing");
        assert_eq!(doc["steps"][0]["status"], "running");
        assert!(doc["error"].is_null());
        assert_eq!(with_error(doc, Some("boom"))["error"], "boom");
    }

    #[test]
    fn the_outcome_says_whether_the_old_account_is_back() {
        let back = Done::Failed {
            reason: "sync broke".to_owned(),
            rollback: Ok(()),
        };
        assert!(outcome_of(&back).unwrap_err().contains("started again"));
        let lost = Done::Failed {
            reason: "sync broke".to_owned(),
            rollback: Err("no".to_owned()),
        };
        assert!(outcome_of(&lost).unwrap_err().contains("going back failed"));
        assert!(outcome_of(&Done::Switched).is_ok());
    }
}
