//! The steps of a Desktop switch, run over the ports so tests fake the outside world.
//!
//! Order: `handover`, `close`, `save_login`, `sync`, `restore_login`, `resume_task`, `start`,
//! `confirm`. The runner reports the phase and the step list after every change. A failure after
//! Claude closed starts the previous account's Claude again: the saved login is put back when the
//! target's login may already be in place (from `restore_login` on), otherwise the login on disk
//! was never touched and Claude is only started. A failed resume task does not undo the switch.
//!
//! A [`Journal`] is written before the target's login is put in place and removed when the switch
//! ended or was rolled back, so a cut-short switch can be repaired. A journal left over from an
//! earlier switch is repaired first (the saved login of its `from` account is put back) and the
//! login on disk is not saved over the vault.
//!
//! `confirm` does not trust the restored `config.json` (the restore wrote the target's account
//! into it): it needs the app to run and to have rewritten `config.json` after the start with the
//! target still named. A target with no saved login yet skips `restore_login`; Claude is started
//! as it is and `confirm` waits for the owner to sign in, and a switch where nobody did ends as
//! [`Done::Noted`], not as a failure.

use std::time::Duration;

use argus_core::clock::now_ms;
use serde::Serialize;

use crate::desktop::app::wait_until;
use crate::desktop::journal::{Entry, Journal};
use crate::ports::{DesktopApp, LoginVault, QUIT_CANCELLED, Quit};
use crate::types::Account;

/// How long `confirm` waits for the app to show the target account.
pub const CONFIRM_WAIT: Duration = Duration::from_secs(60);
/// How long `confirm` waits for the owner to sign in when the target has no saved login.
pub const SIGN_IN_WAIT: Duration = Duration::from_secs(600);
const CONFIRM_POLL: Duration = Duration::from_millis(500);

/// Whether the owner cancelled; asked before Claude closes.
pub type Cancel<'a> = &'a dyn Fn() -> bool;

/// How a step stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Not reached yet.
    Pending,
    /// Running now.
    Running,
    /// Finished.
    Done,
    /// Went wrong.
    Failed,
    /// Not needed, or not reached because of a failure.
    Skipped,
}

/// One step as the `switch` doc shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Step {
    /// The id, one of the eight.
    pub id: &'static str,
    /// What the step does, for the owner.
    pub label: String,
    /// How it stands.
    pub status: StepStatus,
    /// A short note on how it went.
    pub detail: String,
}

/// Why a stage stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// The owner cancelled.
    Cancelled,
    /// It went wrong, with the reason.
    Failed(String),
}

/// The steps that are not a port: the handover wait, the sync, the resume task and the look at
/// which account the app shows.
pub trait Stages: Send + Sync {
    /// Asks for the handovers and waits for them; the note says how many were ready.
    ///
    /// # Errors
    /// [`Stop::Cancelled`] when the owner cancelled, else the reason.
    fn handover(&self) -> Result<String, Stop>;
    /// Copies the chats and folders of the current account onto the next one, with Claude closed.
    ///
    /// # Errors
    /// The reason the sync failed.
    fn sync(&self) -> Result<String, String>;
    /// Schedules the resume task of the target, with Claude closed.
    ///
    /// # Errors
    /// The reason it was not scheduled.
    fn resume(&self) -> Result<String, String>;
    /// The Desktop account `config.json` names now.
    fn active_account(&self) -> Option<String>;
    /// Called right after Claude is confirmed closed, before anything else touches its data.
    fn closed(&self) {}
}

/// The waits of the confirm step.
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    /// How long to wait for the account to show.
    pub confirm: Duration,
    /// How long to wait for a sign-in when the target has no saved login.
    pub sign_in: Duration,
    /// How often to look.
    pub poll: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Timing {
            confirm: CONFIRM_WAIT,
            sign_in: SIGN_IN_WAIT,
            poll: CONFIRM_POLL,
        }
    }
}

/// Who leaves and who comes.
pub struct Plan<'a> {
    /// The switch id, `SW-n`, for the journal.
    pub id: &'a str,
    /// Where the switch is journaled.
    pub journal: &'a Journal,
    /// The account Desktop is on now.
    pub from: &'a Account,
    /// The account to switch to.
    pub to: &'a Account,
    /// Whether handovers come first.
    pub handover: bool,
    /// End the Claude processes by PID when the polite close fails.
    pub force_quit: bool,
}

/// How the switch ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    /// Desktop runs as the target.
    Switched,
    /// Claude was started, but the target could not be confirmed: it has no saved login and nobody
    /// signed in in time, or the journal could not be removed. Nothing is rolled back.
    Noted(String),
    /// The owner cancelled before Claude closed; nothing changed.
    Cancelled,
    /// A step failed.
    Failed {
        /// What failed.
        reason: String,
        /// Whether the previous account's Claude is back: `Err` says why not.
        rollback: Result<(), String>,
    },
}

type Report<'a> = &'a mut dyn FnMut(&'static str, &[Step]);

/// The steps with their report.
struct Steps<'a> {
    list: Vec<Step>,
    report: Report<'a>,
}

const IDS: [&str; 8] = [
    "handover",
    "close",
    "save_login",
    "sync",
    "restore_login",
    "resume_task",
    "start",
    "confirm",
];
const PHASES: [&str; 8] = [
    "handover",
    "closing",
    "saving_login",
    "syncing",
    "restoring_login",
    "scheduling_resume",
    "starting",
    "confirming",
];

impl<'a> Steps<'a> {
    fn new(plan: &Plan, report: Report<'a>) -> Self {
        let labels = [
            "Ask the leads for handovers".to_owned(),
            "Close Claude".to_owned(),
            format!("Save {}'s login", plan.from.label),
            format!("Copy {}'s sidebar onto {}", plan.from.label, plan.to.label),
            format!("Restore {}'s login", plan.to.label),
            "Schedule the resume".to_owned(),
            "Start Claude".to_owned(),
            "Confirm the account".to_owned(),
        ];
        let list = IDS
            .iter()
            .zip(labels)
            .map(|(id, label)| Step {
                id,
                label,
                status: StepStatus::Pending,
                detail: String::new(),
            })
            .collect();
        Steps { list, report }
    }

    fn show(&mut self, phase: &'static str) {
        (self.report)(phase, &self.list);
    }

    fn set(&mut self, at: usize, status: StepStatus, detail: &str) {
        self.list[at].status = status;
        detail.clone_into(&mut self.list[at].detail);
        self.show(PHASES[at]);
    }

    fn begin(&mut self, at: usize) {
        self.set(at, StepStatus::Running, "");
    }

    /// Ends a step from a result; true when it went well.
    fn end(&mut self, at: usize, result: &Result<String, String>) -> bool {
        match result {
            Ok(note) => self.set(at, StepStatus::Done, note),
            Err(why) => self.set(at, StepStatus::Failed, why),
        }
        result.is_ok()
    }

    /// Marks what was never reached as skipped and reports the final phase.
    fn finish(&mut self, phase: &'static str) {
        for step in &mut self.list {
            if matches!(step.status, StepStatus::Pending | StepStatus::Running) {
                step.status = StepStatus::Skipped;
            }
        }
        self.show(phase);
    }
}

/// The phase while Claude is kept in the tray and the owner has to quit it.
pub const WAITING_PHASE: &str = "waiting_for_quit";

/// Runs the switch. `cancel` is asked before Claude closes and while the close waits for the
/// owner to quit Claude from its tray; nowhere later.
pub fn run(
    plan: &Plan,
    app: &dyn DesktopApp,
    vault: &dyn LoginVault,
    stages: &dyn Stages,
    cancel: Cancel,
    timing: Timing,
    report: Report,
) -> Done {
    let mut steps = Steps::new(plan, report);
    let done = drive(plan, (app, vault, stages), cancel, timing, &mut steps);
    let phase = match &done {
        Done::Switched | Done::Noted(_) => "done",
        Done::Cancelled => "cancelled",
        Done::Failed { .. } => "failed",
    };
    steps.finish(phase);
    done
}

fn failed(reason: String) -> Done {
    Done::Failed {
        reason,
        rollback: Ok(()),
    }
}

fn drive(
    plan: &Plan,
    (app, vault, stages): (&dyn DesktopApp, &dyn LoginVault, &dyn Stages),
    cancel: Cancel,
    timing: Timing,
    steps: &mut Steps,
) -> Done {
    if let Some(stop) = handover_step(plan, stages, steps) {
        return stop;
    }
    if cancel() {
        return Done::Cancelled;
    }
    if let Err(why) = close_step(app, plan.force_quit, cancel, steps) {
        return if why == QUIT_CANCELLED {
            Done::Cancelled
        } else {
            failed(why)
        };
    }
    stages.closed();
    let mut flow = Flow {
        restore: false,
        repaired: false,
        first_time: vault.saved_at(plan.to).is_none(),
    };
    let result = repair_leftover(plan, vault, &mut flow)
        .and_then(|()| after_close(plan, (app, vault, stages), timing, steps, &mut flow));
    match result {
        Ok(note) => finished(plan, note),
        Err(why) => {
            let rollback = roll_back(plan.from, app, vault, (flow.restore, plan.force_quit));
            if rollback.is_ok() {
                let _ = plan.journal.clear();
            }
            Done::Failed {
                reason: why,
                rollback,
            }
        }
    }
}

/// What the steps after the close know about each other.
struct Flow {
    /// The target's login may be in place: a rollback has to put the old one back.
    restore: bool,
    /// A leftover journal was repaired, so the login on disk is the saved one, not to be saved.
    repaired: bool,
    /// The target has no saved login yet.
    first_time: bool,
}

/// The end of a switch whose steps all went through: the journal goes, a note is kept.
fn finished(plan: &Plan, note: Option<String>) -> Done {
    match (plan.journal.clear(), note) {
        (Err(why), _) => Done::Noted(why),
        (Ok(()), Some(note)) => Done::Noted(note),
        (Ok(()), None) => Done::Switched,
    }
}

/// With Claude closed, puts back the login of an interrupted switch (see the module notes).
fn repair_leftover(plan: &Plan, vault: &dyn LoginVault, flow: &mut Flow) -> Result<(), String> {
    if !plan.journal.exists() {
        return Ok(());
    }
    vault.restore(plan.from)?;
    plan.journal.clear()?;
    flow.repaired = true;
    Ok(())
}

fn handover_step(plan: &Plan, stages: &dyn Stages, steps: &mut Steps) -> Option<Done> {
    if !plan.handover {
        steps.set(0, StepStatus::Skipped, "handovers were not asked for");
        return None;
    }
    steps.begin(0);
    match stages.handover() {
        Ok(note) => {
            steps.set(0, StepStatus::Done, &note);
            None
        }
        Err(Stop::Cancelled) => Some(Done::Cancelled),
        Err(Stop::Failed(why)) => {
            steps.set(0, StepStatus::Failed, &why);
            Some(failed(why))
        }
    }
}

/// Closes Claude. A close that failed leaves the app as it was: started again if it went away.
fn close_step(
    app: &dyn DesktopApp,
    force: bool,
    cancel: Cancel,
    steps: &mut Steps,
) -> Result<(), String> {
    steps.begin(1);
    let mid_turn = app.turn_in_progress();
    let mut waiting = |detail: &str| {
        detail.clone_into(&mut steps.list[1].detail);
        steps.show(WAITING_PHASE);
    };
    let mut quit = Quit {
        force,
        cancelled: cancel,
        waiting: &mut waiting,
    };
    let result = app.close(&mut quit).map(|()| {
        if mid_turn {
            "closed; a chat looked mid-turn".to_owned()
        } else {
            "closed".to_owned()
        }
    });
    if !steps.end(1, &result) {
        if !app.is_running() {
            let _ = app.start();
        }
        return result.map(|_| ());
    }
    Ok(())
}

/// The steps from `save_login` on. `flow.restore` turns true once the target's login may be in
/// place. The answer is a note when the switch ended without a confirmed account.
fn after_close(
    plan: &Plan,
    (app, vault, stages): (&dyn DesktopApp, &dyn LoginVault, &dyn Stages),
    timing: Timing,
    steps: &mut Steps,
    flow: &mut Flow,
) -> Result<Option<String>, String> {
    if flow.repaired {
        let kept = "an interrupted switch was repaired; the saved login stays as it is";
        steps.set(2, StepStatus::Skipped, kept);
    } else {
        run_step(steps, 2, || {
            vault.save(plan.from).map(|()| "saved".to_owned())
        })?;
    }
    run_step(steps, 3, || stages.sync())?;
    restore_step(plan, vault, steps, flow)?;
    steps.begin(5);
    let _ = mark(plan, flow, "resume_task");
    match stages.resume() {
        Ok(note) => steps.set(5, StepStatus::Done, &note),
        Err(why) => steps.set(5, StepStatus::Failed, &format!("not scheduled: {why}")),
    }
    let since = now_ms();
    run_step(steps, 6, || {
        mark(plan, flow, "start")?;
        app.start().map(|()| "started".to_owned())
    })?;
    steps.begin(7);
    mark(plan, flow, "confirm").map_err(|why| fail_confirm(steps, &why))?;
    let outcome = if flow.first_time {
        Ok(wait_for_sign_in(plan, (app, stages), timing))
    } else {
        confirm(plan, (app, stages), timing, since).map(|note| (note, false))
    };
    match outcome {
        Ok((note, false)) => {
            steps.set(7, StepStatus::Done, &note);
            Ok(None)
        }
        Ok((note, true)) => {
            steps.set(7, StepStatus::Done, &note);
            Ok(Some(note))
        }
        Err(why) => Err(fail_confirm(steps, &why)),
    }
}

fn fail_confirm(steps: &mut Steps, why: &str) -> String {
    steps.set(7, StepStatus::Failed, why);
    why.to_owned()
}

/// Restores the target's login, journaled; with no saved login the step is skipped.
fn restore_step(
    plan: &Plan,
    vault: &dyn LoginVault,
    steps: &mut Steps,
    flow: &mut Flow,
) -> Result<(), String> {
    if flow.first_time {
        let label = &plan.to.label;
        let note = format!(
            "No saved login for {label} yet: sign in to it in Claude once. \
             Argus saves it the next time you switch away."
        );
        steps.set(4, StepStatus::Skipped, &note);
        return Ok(());
    }
    flow.restore = true;
    run_step(steps, 4, || {
        mark(plan, flow, "restore_login")?;
        vault.restore(plan.to).map(|()| "restored".to_owned())
    })
}

/// Writes the journal for `step`; nothing to write when the login on disk is not replaced.
fn mark(plan: &Plan, flow: &Flow, step: &str) -> Result<(), String> {
    if flow.first_time {
        return Ok(());
    }
    plan.journal
        .write(&Entry::now(plan.id, &plan.from.key, &plan.to.key, step))
}

/// Runs one step: begin, call, end; the failure text goes up.
fn run_step(
    steps: &mut Steps,
    at: usize,
    work: impl FnOnce() -> Result<String, String>,
) -> Result<(), String> {
    steps.begin(at);
    let result = work();
    steps.end(at, &result);
    result.map(|_| ())
}

/// Confirms the account after a restore. `config.json` already names the target (the restore
/// wrote it), so that alone proves nothing: the app must run and must have rewritten
/// `config.json` after `since_ms`, still naming the target.
fn confirm(
    plan: &Plan,
    (app, stages): (&dyn DesktopApp, &dyn Stages),
    timing: Timing,
    since_ms: i64,
) -> Result<String, String> {
    let to = plan.to;
    let wanted = to.desktop_account.clone();
    let shown = || {
        app.is_running()
            && app.config_touched_since(since_ms)
            && wanted.is_some()
            && stages.active_account() == wanted
    };
    if wait_until(timing.confirm, timing.poll, shown) {
        return Ok(format!("Claude shows {}", to.label));
    }
    Err(if !app.is_running() {
        "Claude is not running after the start".to_owned()
    } else if !app.config_touched_since(since_ms) {
        format!(
            "Claude did not rewrite its settings after the start, so {} is not confirmed",
            to.label
        )
    } else {
        format!("Claude did not show {} in time", to.label)
    })
}

/// A target with no saved login: waits for the owner to sign in to it. The note says how it went
/// and whether it is a note to keep (`true`: nobody signed in).
fn wait_for_sign_in(
    plan: &Plan,
    (app, stages): (&dyn DesktopApp, &dyn Stages),
    timing: Timing,
) -> (String, bool) {
    let wanted = plan.to.desktop_account.clone();
    let label = &plan.to.label;
    let signed = wait_until(timing.sign_in, timing.poll, || {
        wanted.is_some() && stages.active_account() == wanted
    });
    if signed {
        (format!("Claude shows {label}"), false)
    } else if app.is_running() {
        let note = format!(
            "Claude is open but not signed in to {label}; it stays on the account you choose"
        );
        (note, true)
    } else {
        (format!("Claude was closed before {label} signed in"), true)
    }
}

/// Brings the previous account's Claude back: closes a Claude that runs, puts the saved login
/// back when `restore` says the login may have changed, and starts.
fn roll_back(
    from: &Account,
    app: &dyn DesktopApp,
    vault: &dyn LoginVault,
    (restore, force): (bool, bool),
) -> Result<(), String> {
    if app.is_running() {
        let mut quit = Quit {
            force,
            cancelled: &|| false,
            waiting: &mut |_| {},
        };
        app.close(&mut quit)?;
    }
    if restore {
        vault.restore(from)?;
    }
    app.start()
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Duration;

    use tempfile::TempDir;

    use super::{Done, Plan, Stages, Step, StepStatus, Stop, Timing, run};
    use crate::desktop::journal::{Entry, Journal};
    use crate::fixtures::account;
    use crate::ports::{DesktopApp, LoginVault};
    use crate::types::Account;

    type Log = Mutex<Vec<String>>;

    struct Fake {
        log: Log,
        running: Mutex<bool>,
        fail: &'static str,
        shows: Option<String>,
        saved: bool,
        touched: bool,
        closed_calls: Mutex<u32>,
        /// Whether the journal existed at each restore.
        journal_at_restore: Mutex<Vec<bool>>,
        watch: Option<Journal>,
    }

    impl Fake {
        fn new(fail: &'static str, shows: Option<&str>) -> Self {
            Fake {
                log: Log::default(),
                running: Mutex::new(true),
                fail,
                shows: shows.map(str::to_owned),
                saved: true,
                touched: true,
                closed_calls: Mutex::new(0),
                journal_at_restore: Mutex::default(),
                watch: None,
            }
        }

        fn note(&self, what: &str) -> Result<(), String> {
            self.log.lock().unwrap().push(what.to_owned());
            if self.fail == what {
                Err(format!("{what} broke"))
            } else {
                Ok(())
            }
        }

        fn calls(&self) -> Vec<String> {
            self.log.lock().unwrap().clone()
        }
    }

    impl DesktopApp for Fake {
        fn is_running(&self) -> bool {
            *self.running.lock().unwrap()
        }
        fn turn_in_progress(&self) -> bool {
            false
        }
        fn close(&self, _: &mut crate::ports::Quit) -> Result<(), String> {
            self.note("close")?;
            *self.running.lock().unwrap() = false;
            Ok(())
        }
        fn start(&self) -> Result<(), String> {
            self.note("start")?;
            *self.running.lock().unwrap() = true;
            Ok(())
        }
        fn config_touched_since(&self, _: i64) -> bool {
            self.touched
        }
        fn version(&self) -> Option<String> {
            None
        }
    }

    impl LoginVault for Fake {
        fn save(&self, a: &Account) -> Result<(), String> {
            self.note(&format!("save {}", a.key))
        }
        fn restore(&self, a: &Account) -> Result<(), String> {
            let seen = self.watch.as_ref().is_some_and(Journal::exists);
            self.journal_at_restore.lock().unwrap().push(seen);
            self.note(&format!("restore {}", a.key))
        }
        fn saved_at(&self, _: &Account) -> Option<String> {
            self.saved.then(|| "t".to_owned())
        }
    }

    impl Stages for Fake {
        fn handover(&self) -> Result<String, Stop> {
            self.note("handover").map_err(Stop::Failed)?;
            Ok("2 of 2 ready".to_owned())
        }
        fn sync(&self) -> Result<String, String> {
            self.note("sync").map(|()| "synced".to_owned())
        }
        fn resume(&self) -> Result<String, String> {
            self.note("resume").map(|()| "task R-1".to_owned())
        }
        fn active_account(&self) -> Option<String> {
            self.shows.clone()
        }
        fn closed(&self) {
            *self.closed_calls.lock().unwrap() += 1;
        }
    }

    fn go(fake: &Fake, handover: bool, cancelled: bool) -> (Done, Vec<Step>, Vec<String>) {
        let dir = TempDir::new().unwrap();
        go_with(
            fake,
            handover,
            cancelled,
            &Journal::new(&dir.path().join("j.json")),
        )
    }

    fn go_with(
        fake: &Fake,
        handover: bool,
        cancelled: bool,
        journal: &Journal,
    ) -> (Done, Vec<Step>, Vec<String>) {
        let (from, to) = (account("a"), account("b"));
        let plan = Plan {
            id: "SW-1",
            journal,
            from: &from,
            to: &to,
            handover,
            force_quit: false,
        };
        let timing = Timing {
            confirm: Duration::from_millis(20),
            sign_in: Duration::from_millis(20),
            poll: Duration::from_millis(2),
        };
        let mut last = Vec::new();
        let mut phases = Vec::new();
        let done = run(
            &plan,
            fake,
            fake,
            fake,
            &|| cancelled,
            timing,
            &mut |phase, steps| {
                phases.push(phase.to_owned());
                last = steps.to_vec();
            },
        );
        (done, last, phases)
    }

    fn status(steps: &[Step], id: &str) -> StepStatus {
        steps.iter().find(|s| s.id == id).unwrap().status
    }

    #[test]
    fn the_happy_path_runs_in_order() {
        let fake = Fake::new("", Some("uuid-b"));
        let (done, steps, phases) = go(&fake, true, false);
        assert_eq!(done, Done::Switched);
        assert_eq!(
            fake.calls(),
            [
                "handover",
                "close",
                "save a",
                "sync",
                "restore b",
                "resume",
                "start"
            ]
        );
        assert!(steps.iter().all(|s| s.status == StepStatus::Done));
        assert_eq!(phases.last().unwrap(), "done");
        assert!(phases.contains(&"scheduling_resume".to_owned()));
    }

    #[test]
    fn without_handover_the_first_step_is_skipped() {
        let fake = Fake::new("", Some("uuid-b"));
        let (done, steps, _) = go(&fake, false, false);
        assert_eq!(done, Done::Switched);
        assert_eq!(status(&steps, "handover"), StepStatus::Skipped);
        assert_eq!(fake.calls()[0], "close");
    }

    #[test]
    fn cancel_before_close_changes_nothing() {
        let fake = Fake::new("", None);
        let (done, steps, phases) = go(&fake, false, true);
        assert_eq!(done, Done::Cancelled);
        assert!(fake.calls().is_empty());
        assert_eq!(status(&steps, "close"), StepStatus::Skipped);
        assert_eq!(phases.last().unwrap(), "cancelled");
    }

    #[test]
    fn a_failed_close_starts_claude_again_if_it_went_away() {
        let fake = Fake::new("close", None);
        *fake.running.lock().unwrap() = false;
        let (done, _, _) = go(&fake, false, false);
        assert!(matches!(done, Done::Failed { .. }));
        assert_eq!(fake.calls(), ["close", "start"]);
    }

    #[test]
    fn a_failed_sync_starts_claude_without_touching_the_login() {
        let fake = Fake::new("sync", None);
        let (done, steps, _) = go(&fake, false, false);
        let Done::Failed { reason, rollback } = done else {
            panic!("expected a failure");
        };
        assert_eq!(reason, "sync broke");
        assert!(rollback.is_ok());
        assert_eq!(fake.calls(), ["close", "save a", "sync", "start"]);
        assert_eq!(status(&steps, "restore_login"), StepStatus::Skipped);
    }

    #[test]
    fn an_unconfirmed_account_closes_restores_the_old_login_and_starts() {
        let fake = Fake::new("", Some("uuid-a"));
        let (done, steps, _) = go(&fake, false, false);
        let Done::Failed { rollback, .. } = done else {
            panic!("expected a failure");
        };
        assert!(rollback.is_ok());
        assert_eq!(
            fake.calls(),
            [
                "close",
                "save a",
                "sync",
                "restore b",
                "resume",
                "start",
                "close",
                "restore a",
                "start"
            ]
        );
        assert_eq!(status(&steps, "confirm"), StepStatus::Failed);
    }

    #[test]
    fn a_failed_resume_does_not_undo_the_switch() {
        let fake = Fake::new("resume", Some("uuid-b"));
        let (done, steps, _) = go(&fake, false, false);
        assert_eq!(done, Done::Switched);
        assert_eq!(status(&steps, "resume_task"), StepStatus::Failed);
    }

    #[test]
    fn a_rollback_that_cannot_start_says_so() {
        let fake = Fake::new("restore b", None);
        let (done, _, _) = go(&fake, false, false);
        let Done::Failed { rollback, .. } = done else {
            panic!("expected a failure");
        };
        assert!(rollback.is_ok());
        let fake = Fake::new("restore a", Some("uuid-a"));
        let (done, _, _) = go(&fake, false, false);
        let Done::Failed { rollback, .. } = done else {
            panic!("expected a failure");
        };
        assert_eq!(rollback, Err("restore a broke".to_owned()));
    }

    #[test]
    fn claude_is_told_when_it_has_closed() {
        let fake = Fake::new("", Some("uuid-b"));
        go(&fake, false, false);
        assert_eq!(*fake.closed_calls.lock().unwrap(), 1);
    }

    #[test]
    fn a_config_the_app_did_not_rewrite_fails_the_confirm_and_rolls_back() {
        let mut fake = Fake::new("", Some("uuid-b"));
        fake.touched = false;
        let (done, steps, _) = go(&fake, false, false);
        let Done::Failed { reason, rollback } = done else {
            panic!("expected a failure");
        };
        assert!(reason.contains("did not rewrite"));
        assert!(rollback.is_ok());
        assert_eq!(status(&steps, "confirm"), StepStatus::Failed);
        assert_eq!(fake.calls().last().unwrap(), "start");
    }

    #[test]
    fn the_journal_is_there_while_the_login_is_replaced_and_gone_after() {
        let dir = TempDir::new().unwrap();
        let journal = Journal::new(&dir.path().join("j.json"));
        let mut fake = Fake::new("", Some("uuid-b"));
        fake.watch = Some(journal.clone());
        let (done, _, _) = go_with(&fake, false, false, &journal);
        assert_eq!(done, Done::Switched);
        assert_eq!(*fake.journal_at_restore.lock().unwrap(), [true]);
        assert!(!journal.exists());
    }

    #[test]
    fn a_failure_before_the_restore_never_writes_a_journal() {
        let dir = TempDir::new().unwrap();
        let journal = Journal::new(&dir.path().join("j.json"));
        let fake = Fake::new("sync", None);
        go_with(&fake, false, false, &journal);
        assert!(!journal.exists());
    }

    #[test]
    fn a_rollback_that_fails_leaves_the_journal() {
        let dir = TempDir::new().unwrap();
        let journal = Journal::new(&dir.path().join("j.json"));
        let fake = Fake::new("restore a", Some("uuid-a"));
        go_with(&fake, false, false, &journal);
        assert_eq!(journal.read().unwrap().to, "b");
    }

    #[test]
    fn a_leftover_journal_is_repaired_and_the_login_is_not_saved() {
        let dir = TempDir::new().unwrap();
        let journal = Journal::new(&dir.path().join("j.json"));
        journal
            .write(&Entry::now("SW-0", "a", "b", "start"))
            .unwrap();
        let fake = Fake::new("", Some("uuid-b"));
        let (done, steps, _) = go_with(&fake, false, false, &journal);
        assert_eq!(done, Done::Switched);
        assert!(!fake.calls().contains(&"save a".to_owned()));
        assert_eq!(fake.calls()[..2], ["close", "restore a"]);
        assert_eq!(status(&steps, "save_login"), StepStatus::Skipped);
    }

    #[test]
    fn a_target_without_a_saved_login_skips_the_restore_and_waits_for_the_sign_in() {
        let mut fake = Fake::new("", Some("uuid-b"));
        fake.saved = false;
        let (done, steps, _) = go(&fake, false, false);
        assert_eq!(done, Done::Switched);
        assert_eq!(fake.calls(), ["close", "save a", "sync", "resume", "start"]);
        assert_eq!(status(&steps, "restore_login"), StepStatus::Skipped);
        let detail = &steps
            .iter()
            .find(|s| s.id == "restore_login")
            .unwrap()
            .detail;
        assert!(detail.starts_with("No saved login for b yet: sign in to it in Claude once."));
    }

    #[test]
    fn nobody_signing_in_ends_with_a_note_and_no_rollback() {
        let mut fake = Fake::new("", Some("uuid-a"));
        fake.saved = false;
        let (done, steps, _) = go(&fake, false, false);
        assert!(matches!(done, Done::Noted(_)));
        assert_eq!(fake.calls(), ["close", "save a", "sync", "resume", "start"]);
        assert_eq!(status(&steps, "confirm"), StepStatus::Done);
    }
}
