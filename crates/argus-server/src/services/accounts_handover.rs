//! The handover desk of the accounts service and its event stream. The desk is attached when
//! the accounts service starts, once the wake engine and the boards are up; every call that
//! makes the cycle ready sends a [`CycleEvent`] to the subscribers of
//! [`AccountsService::cycle_events`] (the rules start the countdown on it).

use std::sync::Arc;

use argus_accounts::handover::{
    CycleEvent, Desk, DeskError, Lead, LeadAction, LeadReply, Outcome, StartRequest,
};
use argus_accounts::{Threads, binding_limit};
use serde_json::json;
use tokio::sync::broadcast;

use super::AccountsService;
use super::accounts_switch_log::cycle_lines;
use crate::Services;
use crate::adapters::accounts_threads::{BoardThreads, HandoverDocs, SystemNow};

/// How many cycle events a slow subscriber may fall behind.
const EVENTS: usize = 16;

/// The desk and the senders of its events.
pub struct Handovers {
    desk: Desk,
    threads: Arc<dyn Threads>,
    events: broadcast::Sender<CycleEvent>,
}

impl Handovers {
    /// The desk over its ports.
    #[must_use]
    pub fn new(desk: Desk, threads: Arc<dyn Threads>) -> Self {
        Handovers {
            desk,
            threads,
            events: broadcast::channel(EVENTS).0,
        }
    }

    /// The desk, for reads and for the calls that never make the cycle ready.
    #[must_use]
    pub fn desk(&self) -> &Desk {
        &self.desk
    }

    /// A receiver of every cycle that becomes ready.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<CycleEvent> {
        self.events.subscribe()
    }

    fn publish(&self, event: Option<&CycleEvent>) {
        if let Some(event) = event {
            // No subscriber yet is not an error: the cycle doc still says it is ready.
            let _ = self.events.send(event.clone());
        }
    }

    fn published(&self, outcome: Result<Outcome, DeskError>) -> Result<Outcome, DeskError> {
        if let Ok(done) = &outcome {
            self.publish(done.event.as_ref());
        }
        outcome
    }

    /// Runs a desk call and notes in the switch step log what it changed in the cycle: the
    /// cycle started, each lead asked, each handover received, the resumes.
    fn logged<T>(
        &self,
        accounts: &AccountsService,
        call: impl FnOnce(&Desk) -> Result<T, DeskError>,
    ) -> Result<T, DeskError> {
        let before = self.desk.cycle();
        let out = call(&self.desk);
        let after = self.desk.cycle();
        accounts.note_cycle(
            &cycle_lines(before.as_ref(), after.as_ref()),
            after.as_ref(),
            &|board| self.desk.project_name(board),
        );
        out
    }

    /// The use of the cycle's account now, when the cycle and the poll are known.
    fn cycle_use(&self, accounts: &AccountsService) -> Option<f64> {
        self.desk
            .cycle()
            .and_then(|c| use_now(accounts, &c.account))
    }

    /// The handover file of a lead that is idle with a current handover, when it has one.
    #[must_use]
    pub fn handed_over(&self, lead: (&str, &str)) -> Option<String> {
        self.threads.handed_over(lead.0, lead.1)
    }

    /// Starts a cycle. Blocking: wakes go through the wake engine.
    ///
    /// # Errors
    /// See [`Desk::start`].
    pub fn start(
        &self,
        request: StartRequest,
        accounts: &AccountsService,
    ) -> Result<Outcome, DeskError> {
        let defaults = accounts.config().estimate_defaults;
        self.published(self.logged(accounts, |d| d.start(request, &defaults)))
    }

    /// The ask point. Blocking.
    ///
    /// # Errors
    /// See [`Desk::ask_all`].
    pub fn ask_all(
        &self,
        use_now: Option<f64>,
        accounts: &AccountsService,
    ) -> Result<Outcome, DeskError> {
        self.published(self.logged(accounts, |d| d.ask_all(use_now)))
    }

    /// The switch point came first. Blocking.
    ///
    /// # Errors
    /// See [`Desk::cut_off_rest`].
    pub fn cut_off_rest(
        &self,
        use_now: Option<f64>,
        accounts: &AccountsService,
    ) -> Result<Outcome, DeskError> {
        self.published(self.logged(accounts, |d| d.cut_off_rest(use_now)))
    }

    /// Asks a lead of the estimating cycle for a fresh estimate. Blocking.
    ///
    /// # Errors
    /// See [`Desk::re_ask_estimate`].
    pub fn re_ask_estimate(
        &self,
        lead: (&str, &str),
        accounts: &AccountsService,
    ) -> Result<Outcome, DeskError> {
        let defaults = accounts.config().estimate_defaults;
        self.logged(accounts, |d| d.re_ask_estimate(lead, &defaults))
    }

    /// Adds a lead that started working after the cycle did. Blocking.
    ///
    /// # Errors
    /// See [`Desk::add_lead`].
    pub fn add_lead(&self, lead: &Lead, accounts: &AccountsService) -> Result<Outcome, DeskError> {
        let defaults = accounts.config().estimate_defaults;
        self.logged(accounts, |d| d.add_lead(lead, &defaults))
    }

    /// The owner skips one lead's wait: ready with `file`, or cut off without one. Blocking.
    ///
    /// # Errors
    /// See [`Desk::skip_lead`].
    pub fn skip_lead(
        &self,
        lead: (&str, &str),
        file: Option<&str>,
        accounts: &AccountsService,
    ) -> Result<Outcome, DeskError> {
        let use_now = self.cycle_use(accounts);
        self.published(self.logged(accounts, |d| d.skip_lead(lead, file, use_now)))
    }

    /// Counts every pending lead that went idle with a current handover as ready, and returns
    /// how many it settled. The cycle may close with the last one. Blocking.
    pub fn confirm_idle_sweep(&self, accounts: &AccountsService) -> usize {
        let Some(cycle) = self.desk.cycle().filter(|c| c.phase.is_open()) else {
            return 0;
        };
        let use_now = self.cycle_use(accounts);
        let mut settled = 0;
        for h in cycle.leads.iter().filter(|h| h.state.is_pending()) {
            let lead = (h.board.as_str(), h.thread.as_str());
            let Some(file) = self.handed_over(lead) else {
                continue;
            };
            let done = self.logged(accounts, |d| d.confirm_idle(lead, &file, use_now));
            settled += usize::from(self.published(done).is_ok());
        }
        settled
    }

    /// A lead's request, with the use of the cycle's account now. Blocking.
    ///
    /// # Errors
    /// See [`Desk::lead`].
    pub fn lead(
        &self,
        lead: (&str, &str),
        action: LeadAction,
        accounts: &AccountsService,
    ) -> Result<LeadReply, DeskError> {
        let use_now = self.cycle_use(accounts);
        let defaults = accounts.config().estimate_defaults;
        let reply = self.logged(accounts, |d| d.lead(lead, action, use_now, &defaults))?;
        self.publish(reply.event.as_ref());
        Ok(reply)
    }
}

/// The percent of an account's binding limit at the last poll.
#[must_use]
pub fn use_now(accounts: &AccountsService, account: &str) -> Option<f64> {
    let latest = accounts.latest_usage();
    binding_limit(&latest.get(account)?.usage).map(|l| l.percent)
}

impl AccountsService {
    /// The handover desk, once the service started.
    #[must_use]
    pub fn handovers(&self) -> Option<Arc<Handovers>> {
        self.handovers.get().cloned()
    }

    /// A receiver of every handover cycle that becomes ready (all asked leads ready, or the rest
    /// cut off); `None` before the service started.
    #[must_use]
    pub fn cycle_events(&self) -> Option<broadcast::Receiver<CycleEvent>> {
        self.handovers.get().map(|h| h.subscribe())
    }
}

/// Builds the desk over the boards, the wake engine and the database, and attaches it.
pub(crate) fn attach(svc: &Services, accounts: &AccountsService) {
    let clock = Arc::new(SystemNow);
    let threads = Arc::new(BoardThreads::new(
        svc.boards(),
        svc.wake_hook(),
        clock.clone(),
        svc.paths.coord_home.clone(),
    ));
    let store = Arc::new(HandoverDocs::new(Arc::clone(&svc.db)));
    match Desk::new(Arc::clone(&threads) as Arc<dyn Threads>, store, clock) {
        Ok(desk) => {
            let _ = accounts
                .handovers
                .set(Arc::new(Handovers::new(desk, threads)));
        }
        Err(e) => svc
            .log
            .error("accounts handover desk not started", json!({ "error": e })),
    }
}
