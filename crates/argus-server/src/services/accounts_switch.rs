//! The Desktop switch of the accounts service: one switch at a time, from the Switch now button
//! or an auto-switch rule. It asks the leads for handovers first when told to, then runs the
//! steps of [`argus_accounts::desktop`] and writes the `switch` state doc after every change.
//! `cancel` works until Claude is about to close. The saved Desktop logins live in the vault
//! under `<DASHBOARD_HOME>\accounts\logins`; their state is shown in the `Accounts` docs.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use argus_accounts::desktop::{
    Done, FsVault, Journal, Plan, Repair, Stages, Step, StepStatus, Stop, Timing, WAITING_PHASE,
    last_known_account, repair,
};
use argus_accounts::handover::{CycleEvent, CycleMode, StartRequest};
use argus_accounts::{Account, LoginVault, QUIT_BY_HAND, Scope, find_app_data};
use argus_core::Coll;
use argus_core::clock::now_ms;
use argus_core::ids::PORTFOLIO;
use argus_notify::{By, Level, PingRequest};
use serde_json::{Value, json};
use tokio::sync::broadcast::Receiver;

use super::AccountsService;
use super::accounts_handover::{Handovers, use_now};
use super::accounts_switch_check::check_sidebars;
use super::accounts_switch_log::{
    Line, LineState, Stage, stage_of, step_lines, sync_info, sync_note,
};
use crate::Services;

/// What a second switch answers while one runs; the route turns it into a 409.
pub const SWITCH_BUSY: &str = "a switch is already running";
/// Why an automatic switch is held while a cut-short switch is not repaired.
pub const INTERRUPTED: &str = "interrupted switch, will repair at the next switch";
const SWITCH_DOC: &str = "switch";

/// The Claude package version, read at start-up and on each switch.
static VERSION: Mutex<Option<String>> = Mutex::new(None);
/// The version the owner was last told about, so the notice goes out once per version.
static TOLD_VERSION: Mutex<Option<String>> = Mutex::new(None);

pub(super) fn cached_version() -> Option<String> {
    VERSION
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// Reads the installed Claude version and keeps it. Blocking (runs PowerShell).
fn refresh_version(accounts: &AccountsService, app: &Path) -> Option<String> {
    let found = accounts.desktop(app).version();
    VERSION
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone_from(&found);
    found
}

/// The Claude version, read once and kept; a switch and start-up read it again. Blocking.
pub fn app_version(accounts: &AccountsService) -> Option<String> {
    cached_version().or_else(|| {
        let app = find_app_data(accounts.config().app_data.as_deref())?;
        refresh_version(accounts, &app)
    })
}

fn journal_of(svc: &Services) -> Journal {
    Journal::new(&svc.paths.home.join("accounts").join("switch-journal.json"))
}
/// How long the switch waits for the handovers.
const HANDOVER_WAIT: Duration = Duration::from_secs(600);
const HANDOVER_POLL: Duration = Duration::from_millis(250);
/// How many polls of the handover wait pass between looks for leads that went idle (about 5 s).
const IDLE_SWEEP_POLLS: u32 = 20;

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
    /// Claude is in the tray and the close waits for the owner to quit it: cancel works again.
    waiting: bool,
}

/// The cancel, "anyway" and "skip the wait" switches of the running switch.
#[derive(Clone, Default)]
struct Control {
    gate: Arc<Mutex<Gate>>,
    anyway: Arc<AtomicBool>,
    skip_wait: Arc<AtomicBool>,
}

impl Control {
    /// The owner cancels; false once Claude is closing.
    fn cancel(&self) -> bool {
        let mut gate = self.gate.lock().unwrap_or_else(PoisonError::into_inner);
        if gate.closing && !gate.waiting {
            return false;
        }
        gate.cancelled = true;
        true
    }

    fn set_waiting(&self, waiting: bool) {
        self.gate
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .waiting = waiting;
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

/// Stops waiting for the account check or the sign-in: the switch ends as noted. False when no
/// switch runs.
pub fn request_skip_wait() -> bool {
    let held = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
    held.as_ref()
        .map(|control| control.skip_wait.store(true, Ordering::SeqCst))
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
    /// The warning of a switch on a Claude version no switch confirmed yet.
    warning: Option<String>,
    control: Control,
    _slot: Slot,
}

/// The account Desktop is on: the one a half-done switch left, else the one that is signed in.
fn source_account(
    accounts: &[Account],
    leftover: Option<&String>,
    active: Option<&String>,
) -> Option<Account> {
    accounts
        .iter()
        .find(|a| match leftover {
            Some(key) => &a.key == key,
            None => a.desktop_account.is_some() && a.desktop_account.as_ref() == active,
        })
        .cloned()
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
    let vault = FsVault::new(&app, &logins_root(svc));
    if origin == Origin::Rule && vault.saved_at(&target).is_none() {
        return Err(format!("{} has no saved login yet", target.label));
    }
    let warning = version_warning(svc, &config, origin);
    let leftover = match repair(
        &journal_of(svc),
        &*accounts.desktop(&app),
        &vault,
        &config.accounts,
    )? {
        Repair::Blocked { .. } if origin == Origin::Rule => return Err(INTERRUPTED.to_owned()),
        Repair::Blocked { from } => Some(from),
        _ => None,
    };
    let active = last_known_account(&app);
    let from = source_account(&config.accounts, leftover.as_ref(), active.as_ref())
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
        warning,
        control,
        _slot: slot,
    })
}

/// The warning step of a switch on a Claude version that no switch confirmed yet;
/// `None` when the version is the confirmed one.
fn unverified_text(current: Option<&str>, verified: Option<&str>) -> Option<String> {
    match current {
        Some(now) if verified == Some(now) => None,
        Some(now) => Some(format!(
            "Warning: no switch confirmed Claude {now} yet; the switch goes on and checks the account and the sidebars after it"
        )),
        None => Some(
            "Warning: the Claude version could not be read; the switch goes on and checks the account and the sidebars after it"
                .to_owned(),
        ),
    }
}

/// No switch stops on a new Claude version: it goes on with a warning step. An automatic switch
/// also tells the owner, once per version; the owner who switches by hand sees the step.
fn version_warning(
    svc: &Services,
    config: &argus_accounts::AccountsConfig,
    origin: Origin,
) -> Option<String> {
    let current = cached_version();
    let text = unverified_text(current.as_deref(), config.verified_app_version.as_deref())?;
    if origin == Origin::Rule {
        tell_once(svc, current.unwrap_or_default(), &text);
    }
    Some(text)
}

/// Pings the owner about the Claude version `seen`, unless it was told about that one already.
fn tell_once(svc: &Services, seen: String, text: &str) {
    let mut told = TOLD_VERSION.lock().unwrap_or_else(PoisonError::into_inner);
    if told.as_deref() != Some(seen.as_str()) {
        *told = Some(seen);
        if let Some(notify) = &svc.notify {
            notify.send(PingRequest {
                title: Some("Claude was updated".to_owned()),
                text: text.to_owned(),
                level: Level::Ping,
                project: None,
                thread: None,
                by: By::Auto,
                url: Some(format!("http://127.0.0.1:{}/?tab=accounts", svc.port())),
            });
        }
    }
}

/// The `switch` doc as it is shown now.
pub(crate) fn shown_switch(svc: &Services) -> Option<Value> {
    svc.db
        .docs()
        .get(PORTFOLIO, Coll::AccountsState, SWITCH_DOC)
        .ok()
        .flatten()
        .map(|row| row.data)
}

/// The id of the switch: the one a countdown already shows, else a new one.
fn switch_id(svc: &Services) -> String {
    shown_switch(svc)
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
    if origin == Origin::Rule {
        // The gate reads the cached version; make it current first (PowerShell, off the runtime).
        if let Some(accounts) = svc.accounts.clone() {
            let _ = tokio::task::spawn_blocking(move || {
                find_app_data(accounts.config().app_data.as_deref())
                    .and_then(|app| refresh_version(&accounts, &app))
            })
            .await;
        }
    }
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
        let ends = (self.from.clone(), self.to.clone());
        let id = self.id.clone();
        let result = tokio::task::spawn_blocking(move || self.run_blocking())
            .await
            .map_err(|e| e.to_string())?;
        // The active account and the logins changed; show them now rather than at the next poll.
        let _ = accounts.poll_now().await;
        let (svc2, accounts2) = (Arc::clone(&svc), Arc::clone(&accounts));
        let _ = tokio::task::spawn_blocking(move || publish_logins(&svc2, &accounts2)).await;
        let done = result?;
        if done == Done::Switched {
            let _ =
                tokio::task::spawn_blocking(move || check_sidebars(&svc, &accounts, &ends, &id))
                    .await;
        }
        Ok(())
    }

    fn put(&self, phase: &str, steps: &[Step], error: Option<&str>, lines: &[Line]) {
        let doc = switch_doc(
            &self.id,
            (&self.from.key, &self.to.key),
            self.origin,
            phase,
            steps,
        );
        let doc = with_error(doc, error);
        if let Err(e) = self.accounts.put_switch(doc, lines) {
            self.svc
                .log
                .error("accounts switch doc not written", json!({ "error": e }));
        }
    }

    /// The first lines of the step log: who started the switch, and the version warning.
    fn opening_lines(&self) -> Vec<Line> {
        let by = match self.origin {
            Origin::Owner => "the owner",
            Origin::Rule => "the auto-switch rule",
        };
        let text = format!(
            "Switch started by {by}: from {} to {}.",
            self.from.label, self.to.label
        );
        let mut lines = vec![Line::new(Stage::Detect, LineState::Done, text)];
        lines.extend(
            self.warning
                .iter()
                .map(|w| Line::new(Stage::Detect, LineState::Done, format!("{w}."))),
        );
        lines
    }

    fn run_blocking(self) -> Result<Done, String> {
        let desktop = self.accounts.desktop(&self.app);
        let vault = FsVault::new(&self.app, &logins_root(&self.svc));
        let stages = ServerStages::new(&self);
        refresh_version(&self.accounts, &self.app);
        self.accounts.note_switch(&self.opening_lines());
        let journal = journal_of(&self.svc);
        let plan = Plan {
            id: &self.id,
            journal: &journal,
            from: &self.from,
            to: &self.to,
            handover: self.handover,
            force_quit: self.accounts.config().force_quit,
        };
        let mut last = (String::from("idle"), Vec::new());
        let mut told = false;
        let done = argus_accounts::desktop::run(
            &plan,
            &*desktop,
            &vault,
            &stages,
            &|| self.control.last_call(),
            Timing::default(),
            &mut |phase, steps| {
                let waiting = phase == WAITING_PHASE;
                self.control.set_waiting(waiting);
                if waiting && self.origin == Origin::Rule && !told {
                    told = true;
                    self.tell_to_quit();
                }
                let mut lines = step_lines(&last.1, steps);
                // A finished switch shows as verifying until the sidebar check.
                let shown = if phase == "done" {
                    let text = format!("Done: Claude runs as {}.", self.to.label);
                    lines.push(Line::new(Stage::Switch, LineState::Done, text));
                    "verifying"
                } else {
                    phase
                };
                self.put(shown, steps, None, &lines);
                last = (shown.to_owned(), steps.to_vec());
            },
        );
        let outcome = outcome_of(&done);
        if let Err(why) = &outcome {
            self.put(
                &last.0,
                &last.1,
                Some(why),
                &[failure_line(&done, &last.1, why)],
            );
        }
        if let Done::Noted(note) = &done {
            let text = format!("The account was not confirmed: {note}.");
            let line = Line::new(Stage::Verify, LineState::Skipped, text);
            self.accounts
                .finish_switch(&self.id, ("done", None), &[line]);
        }
        outcome.map(|()| done)
    }

    /// An automatic switch cannot finish until the owner quits Desktop from its tray.
    fn tell_to_quit(&self) {
        let Some(notify) = &self.svc.notify else {
            return;
        };
        notify.send(PingRequest {
            title: Some("Quit Desktop from its tray to finish the switch".to_owned()),
            text: QUIT_BY_HAND.to_owned(),
            level: Level::Urgent,
            project: None,
            thread: None,
            by: By::Auto,
            url: Some(format!(
                "http://127.0.0.1:{}/?tab=accounts",
                self.svc.port()
            )),
        });
    }
}

/// The step-log line of a switch that did not finish: the stage of the step that failed.
fn failure_line(done: &Done, steps: &[Step], why: &str) -> Line {
    if *done == Done::Cancelled {
        return Line::new(
            Stage::Switch,
            LineState::Skipped,
            "The switch was cancelled.",
        );
    }
    let stage = steps
        .iter()
        .find(|s| s.status == StepStatus::Failed)
        .map_or(Stage::Switch, |s| stage_of(s.id));
    Line::new(
        stage,
        LineState::Failed,
        format!("The switch failed: {why}."),
    )
}

/// The result a [`Done`] makes, with the reason in plain words.
fn outcome_of(done: &Done) -> Result<(), String> {
    match done {
        Done::Switched | Done::Noted(_) => Ok(()),
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
    backup: Backup,
}

/// The backup folder the sync of this switch wrote, kept so a rollback can put it back.
#[derive(Default)]
struct Backup(Mutex<Option<String>>);

impl Backup {
    fn keep(&self, dir: Option<&String>) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = dir.cloned();
    }

    /// The folder, once: a second undo has nothing to put back.
    fn take(&self) -> Option<String> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

impl<'a> ServerStages<'a> {
    fn new(run: &'a Prepared) -> Self {
        ServerStages {
            run,
            backup: Backup::default(),
        }
    }

    fn wait_ready(
        &self,
        desk: &Handovers,
        events: &mut Receiver<CycleEvent>,
        cycle: &str,
        now_use: Option<f64>,
    ) -> Result<String, Stop> {
        let end = Instant::now() + HANDOVER_WAIT;
        let mut polls = 0_u32;
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
                    .cut_off_rest(now_use, &self.run.accounts)
                    .map_err(|e| Stop::Failed(e.to_string()))?;
                return Ok(out
                    .event
                    .as_ref()
                    .map_or_else(|| "cut off the rest".to_owned(), ready_note));
            }
            polls += 1;
            if polls % IDLE_SWEEP_POLLS == 0 {
                desk.confirm_idle_sweep(&self.run.accounts);
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
        if let Some(open) = desk
            .desk()
            .cycle()
            .filter(|c| c.phase.is_open() && c.account == self.run.from.key)
        {
            // The rule opened this cycle at its switch point: wait for it instead of a new one.
            let asked = desk.ask_all(now_use, accounts).ok();
            return match asked.and_then(|a| a.event) {
                Some(event) => Ok(ready_note(&event)),
                None => self.wait_ready(&desk, &mut events, &open.id, now_use),
            };
        }
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
        let asked = desk.ask_all(now_use, accounts).ok();
        let ready = started.event.or_else(|| asked.and_then(|a| a.event));
        match ready {
            Some(event) => Ok(ready_note(&event)),
            None => self.wait_ready(&desk, &mut events, &started.cycle.id, now_use),
        }
    }

    fn sync(&self) -> Result<String, String> {
        let accounts = &self.run.accounts;
        let summary = match accounts.copy_onto(&self.run.from, &self.run.to, false) {
            Ok(summary) => summary,
            Err(why) => {
                let error = format!("the copy of the chats and folders failed: {why}");
                accounts.note_sync(sync_info(None, Some(error.as_str())));
                return Err(error);
            }
        };
        self.backup.keep(summary.backup_dir.as_ref());
        if !summary.verified {
            let error = format!(
                "the copy wrote but its check found changes left: {}",
                sync_note(&summary)
            );
            accounts.note_sync(sync_info(Some(&summary), Some(error.as_str())));
            return Err(error);
        }
        accounts.note_sync(sync_info(Some(&summary), None));
        Ok(sync_note(&summary))
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

    fn skip_wait(&self) -> bool {
        self.run.control.skip_wait.load(Ordering::SeqCst)
    }

    fn undo(&self) {
        let Some(dir) = self.backup.take() else {
            return;
        };
        if let Err(e) = self.run.accounts.undo_sync(Some(&dir), false) {
            self.run.svc.log.warn(
                "accounts sync not undone by the rollback",
                json!({ "error": e, "backup": dir }),
            );
        }
    }

    fn closed(&self) {
        let (Some(account), Some(org)) =
            (&self.run.from.desktop_account, &self.run.from.desktop_org)
        else {
            return;
        };
        let scope = Scope {
            account: account.clone(),
            org: org.clone(),
        };
        if let Err(e) = super::accounts_resume::cleanup_pending(&self.run.svc, &scope) {
            self.run
                .svc
                .log
                .warn("accounts resume task not cleaned up", json!({ "error": e }));
        }
    }
}

/// Looks for a cut-short switch at start-up and repairs it when Claude is closed; when Claude
/// runs, the `switch` doc says it failed and the repair waits for the next switch. Blocking.
pub(crate) fn repair_interrupted(svc: &Services, accounts: &AccountsService) {
    let config = accounts.config();
    let Some(app) = find_app_data(config.app_data.as_deref()) else {
        return;
    };
    refresh_version(accounts, &app);
    let journal = journal_of(svc);
    let vault = FsVault::new(&app, &logins_root(svc));
    match repair(&journal, &*accounts.desktop(&app), &vault, &config.accounts) {
        Ok(Repair::Clean) => {}
        Ok(Repair::Repaired { from }) => svc.log.warn(
            "accounts interrupted switch repaired",
            json!({ "from": from }),
        ),
        Ok(Repair::Blocked { from }) => {
            let entry = journal.read();
            let id = entry.map_or_else(|| "SW-0".to_owned(), |e| e.id);
            let doc = switch_doc(&id, (&from, ""), Origin::Owner, "failed", &[]);
            let doc = with_error(doc, Some(INTERRUPTED));
            let text = format!("The switch was cut short: {INTERRUPTED}.");
            let line = Line::new(Stage::Switch, LineState::Failed, text);
            if let Err(e) = accounts.put_switch(doc, &[line]) {
                svc.log
                    .error("accounts switch doc not written", json!({ "error": e }));
            }
        }
        Err(e) => svc.log.error(
            "accounts interrupted switch not repaired",
            json!({ "error": e }),
        ),
    }
}

/// `<DASHBOARD_HOME>\accounts\logins`.
pub(crate) fn logins_root(svc: &Services) -> PathBuf {
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
    if argus_accounts::sync::lock_running(&app) {
        return Err(
            "Claude is running, so its login cannot be saved safely. Close Claude and save again; \
             a switch saves the login for you"
                .to_owned(),
        );
    }
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

// test-tag: critical
#[cfg(test)]
mod tests {
    use argus_accounts::desktop::{Done, Step, StepStatus};

    use super::{Backup, Control, Origin, outcome_of, switch_doc, unverified_text, with_error};

    #[test]
    fn the_confirmed_version_has_no_warning() {
        assert_eq!(unverified_text(Some("1.2"), Some("1.2")), None);
    }

    #[test]
    fn a_new_or_unreadable_version_has_a_warning() {
        let new = unverified_text(Some("1.3"), Some("1.2")).unwrap_or_default();
        assert!(new.contains("Claude 1.3"), "{new}");
        let none = unverified_text(None, Some("1.2")).unwrap_or_default();
        assert!(none.contains("could not be read"), "{none}");
    }

    #[test]
    fn a_rollback_gets_the_backup_of_the_sync_once() {
        let backup = Backup::default();
        assert_eq!(backup.take(), None);
        backup.keep(Some(&"b-1".to_owned()));
        assert_eq!(backup.take(), Some("b-1".to_owned()));
        assert_eq!(backup.take(), None);
    }

    #[test]
    fn a_sync_that_wrote_nothing_leaves_nothing_to_undo() {
        let backup = Backup::default();
        backup.keep(Some(&"b-1".to_owned()));
        backup.keep(None);
        assert_eq!(backup.take(), None);
    }

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
    fn a_control_copy_sees_the_skip_wait_request() {
        use std::sync::atomic::Ordering;
        let control = Control::default();
        let copy = control.clone();
        assert!(!copy.skip_wait.load(Ordering::SeqCst));
        control.skip_wait.store(true, Ordering::SeqCst);
        assert!(copy.skip_wait.load(Ordering::SeqCst));
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
    fn a_noted_switch_is_not_an_error() {
        assert!(outcome_of(&Done::Noted("not signed in".to_owned())).is_ok());
    }

    #[test]
    fn the_interrupted_text_names_the_repair() {
        assert!(super::INTERRUPTED.contains("repair at the next switch"));
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
