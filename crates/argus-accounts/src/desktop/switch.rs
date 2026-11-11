//! The steps of a Desktop switch, run over the ports so tests fake the outside world.
//!
//! Order: `handover`, `close`, `save_login`, `sync`, `restore_login`, `resume_task`, `start`,
//! `confirm`. The runner reports the phase and the step list after every change. A failure after
//! Claude closed starts the previous account's Claude again: the saved login is put back when the
//! target's login may already be in place (from `restore_login` on), otherwise the login on disk
//! was never touched and Claude is only started. A failed resume task does not undo the switch.

use std::time::Duration;

use serde::Serialize;

use crate::desktop::app::wait_until;
use crate::ports::{DesktopApp, LoginVault};
use crate::types::Account;

/// How long `confirm` waits for the app to show the target account.
pub const CONFIRM_WAIT: Duration = Duration::from_secs(60);
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
    /// Syncs chats and folders between the accounts, with Claude closed.
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
}

/// The waits of the confirm step.
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    /// How long to wait for the account to show.
    pub confirm: Duration,
    /// How often to look.
    pub poll: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Timing {
            confirm: CONFIRM_WAIT,
            poll: CONFIRM_POLL,
        }
    }
}

/// Who leaves and who comes.
pub struct Plan<'a> {
    /// The account Desktop is on now.
    pub from: &'a Account,
    /// The account to switch to.
    pub to: &'a Account,
    /// Whether handovers come first.
    pub handover: bool,
}

/// How the switch ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    /// Desktop runs as the target.
    Switched,
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
            "Sync the sidebars".to_owned(),
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

/// Runs the switch. `cancel` is asked before Claude closes and nowhere later.
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
        Done::Switched => "done",
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
    if let Err(why) = close_step(app, steps) {
        return failed(why);
    }
    let mut restore = false;
    let Err(why) = after_close(plan, (app, vault, stages), timing, steps, &mut restore) else {
        return Done::Switched;
    };
    Done::Failed {
        reason: why,
        rollback: roll_back(plan.from, app, vault, restore),
    }
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
fn close_step(app: &dyn DesktopApp, steps: &mut Steps) -> Result<(), String> {
    steps.begin(1);
    let mid_turn = app.turn_in_progress();
    let result = app.close().map(|()| {
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

/// The steps from `save_login` on. `restore` turns true once the target's login may be in place.
fn after_close(
    plan: &Plan,
    (app, vault, stages): (&dyn DesktopApp, &dyn LoginVault, &dyn Stages),
    timing: Timing,
    steps: &mut Steps,
    restore: &mut bool,
) -> Result<(), String> {
    run_step(steps, 2, || {
        vault.save(plan.from).map(|()| "saved".to_owned())
    })?;
    run_step(steps, 3, || stages.sync())?;
    *restore = true;
    run_step(steps, 4, || {
        vault.restore(plan.to).map(|()| "restored".to_owned())
    })?;
    let resume = stages.resume();
    steps.begin(5);
    match &resume {
        Ok(note) => steps.set(5, StepStatus::Done, note),
        Err(why) => steps.set(5, StepStatus::Failed, &format!("not scheduled: {why}")),
    }
    run_step(steps, 6, || app.start().map(|()| "started".to_owned()))?;
    run_step(steps, 7, || confirm(plan.to, stages, timing))
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

fn confirm(to: &Account, stages: &dyn Stages, timing: Timing) -> Result<String, String> {
    let wanted = to.desktop_account.clone();
    let shown = wait_until(timing.confirm, timing.poll, || {
        wanted.is_some() && stages.active_account() == wanted
    });
    if shown {
        Ok(format!("Claude shows {}", to.label))
    } else {
        Err(format!("Claude did not show {} in time", to.label))
    }
}

/// Brings the previous account's Claude back: closes a Claude that runs, puts the saved login
/// back when `restore` says the login may have changed, and starts.
fn roll_back(
    from: &Account,
    app: &dyn DesktopApp,
    vault: &dyn LoginVault,
    restore: bool,
) -> Result<(), String> {
    if app.is_running() {
        app.close()?;
    }
    if restore {
        vault.restore(from)?;
    }
    app.start()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Duration;

    use super::{Done, Plan, Stages, Step, StepStatus, Stop, Timing, run};
    use crate::ports::{DesktopApp, LoginVault};
    use crate::types::{Account, Rules};

    type Log = Mutex<Vec<String>>;

    struct Fake {
        log: Log,
        running: Mutex<bool>,
        fail: &'static str,
        shows: Option<String>,
    }

    impl Fake {
        fn new(fail: &'static str, shows: Option<&str>) -> Self {
            Fake {
                log: Log::default(),
                running: Mutex::new(true),
                fail,
                shows: shows.map(str::to_owned),
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
        fn close(&self) -> Result<(), String> {
            self.note("close")?;
            *self.running.lock().unwrap() = false;
            Ok(())
        }
        fn start(&self) -> Result<(), String> {
            self.note("start")?;
            *self.running.lock().unwrap() = true;
            Ok(())
        }
    }

    impl LoginVault for Fake {
        fn save(&self, a: &Account) -> Result<(), String> {
            self.note(&format!("save {}", a.key))
        }
        fn restore(&self, a: &Account) -> Result<(), String> {
            self.note(&format!("restore {}", a.key))
        }
        fn saved_at(&self, _: &Account) -> Option<String> {
            None
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
    }

    fn account(key: &str) -> Account {
        Account {
            key: key.to_owned(),
            label: key.to_owned(),
            color: "#000000".to_owned(),
            desktop_account: Some(format!("uuid-{key}")),
            desktop_org: None,
            cli_dir: String::new(),
            can_receive: true,
            rules: Rules::default(),
        }
    }

    fn go(fake: &Fake, handover: bool, cancelled: bool) -> (Done, Vec<Step>, Vec<String>) {
        let (from, to) = (account("a"), account("b"));
        let plan = Plan {
            from: &from,
            to: &to,
            handover,
        };
        let timing = Timing {
            confirm: Duration::from_millis(20),
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
}
