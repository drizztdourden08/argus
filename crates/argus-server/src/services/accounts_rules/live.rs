//! The rules' world for real: the usage and the boards of the accounts service, the toasts of
//! the notifications service, the docs and the switch. Everything that touches the disk or
//! waits on a wake runs on the blocking pool.

use std::sync::Arc;

use argus_accounts::LoginVault;
use argus_accounts::desktop::FsVault;
use argus_accounts::handover::{Cycle, CycleMode, CyclePhase, DeskError, Outcome, StartRequest};
use argus_accounts::rules::{CycleView, Estimates, LeadKey, View};
use argus_accounts::{Account, AccountsConfig, PollResult, Severity, binding_limit, find_app_data};
use argus_core::clock::parse_ms;
use argus_notify::{By, Level, PingRequest};
use async_trait::async_trait;
use serde_json::{Value, json};

use super::{Snapshot, World};
use crate::Services;
use crate::services::AccountsService;
use crate::services::accounts::last_known_account;
use crate::services::accounts_estimates::{LeadRef, estimate_state};
use crate::services::accounts_handover::Handovers;
use crate::services::accounts_switch::{Origin, is_running, logins_root, shown_switch, switch_to};

/// How long the read of the usage at the end of a countdown may take before the rules go on.
const REFRESH_LIMIT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Clone)]
pub(super) struct Live {
    svc: Arc<Services>,
    accounts: Arc<AccountsService>,
}

fn view_of(account: &Account, polled: Option<&PollResult>, has_login: bool) -> View {
    let limit = polled.and_then(|p| binding_limit(&p.usage));
    View {
        key: account.key.clone(),
        label: account.label.clone(),
        // A rule may only switch to an account whose Desktop login is saved.
        can_receive: account.can_receive && has_login,
        rules: account.rules.clone(),
        percent: limit.map(|l| l.percent),
        critical: limit.is_some_and(|l| l.severity == Severity::Critical),
        resets_at_ms: limit
            .and_then(|l| l.resets_at.as_deref())
            .and_then(parse_ms),
    }
}

fn lead_key(lead: LeadRef) -> LeadKey {
    LeadKey {
        board: lead.board,
        thread: lead.thread,
    }
}

fn cycle_view(cycle: Option<&Cycle>, active: Option<&str>) -> CycleView {
    let Some(cycle) = cycle else {
        return CycleView::None;
    };
    let mine = active == Some(cycle.account.as_str());
    match (cycle.phase, mine) {
        (CyclePhase::Estimating, true) => CycleView::Estimating,
        (CyclePhase::Asked, true) => CycleView::Asked,
        (CyclePhase::Ready, true) => CycleView::Ready,
        (CyclePhase::Estimating | CyclePhase::Asked, false) => CycleView::Foreign,
        _ => CycleView::None,
    }
}

fn active_key(config: &AccountsConfig) -> Option<String> {
    let app = find_app_data(config.app_data.as_deref())?;
    let uuid = last_known_account(&app)?;
    config
        .accounts
        .iter()
        .find(|a| a.desktop_account.as_deref() == Some(uuid.as_str()))
        .map(|a| a.key.clone())
}

impl Live {
    pub(super) fn new(svc: &Arc<Services>, accounts: &Arc<AccountsService>) -> Self {
        Live {
            svc: Arc::clone(svc),
            accounts: Arc::clone(accounts),
        }
    }

    async fn blocking<T: Send + 'static>(
        &self,
        work: impl FnOnce(&Live) -> T + Send + 'static,
    ) -> Result<T, String> {
        let me = self.clone();
        tokio::task::spawn_blocking(move || work(&me))
            .await
            .map_err(|e| e.to_string())
    }

    async fn desk_call(
        &self,
        call: impl FnOnce(&Live) -> Result<Outcome, DeskError> + Send + 'static,
    ) -> Result<(), String> {
        self.blocking(move |me| {
            let outcome = call(me).map_err(|e| e.to_string())?;
            for failure in &outcome.failures {
                me.svc
                    .log
                    .warn("accounts handover wake failed", json!({ "error": failure }));
            }
            Ok(())
        })
        .await?
    }

    fn read(&self) -> Snapshot {
        let config = self.accounts.config();
        let latest = self.accounts.latest_usage();
        let app = find_app_data(config.app_data.as_deref()).unwrap_or_default();
        let vault = FsVault::new(&app, &logins_root(&self.svc));
        let accounts = config
            .accounts
            .iter()
            .map(|a| view_of(a, latest.get(&a.key), vault.saved_at(a).is_some()))
            .collect();
        let active = active_key(&config);
        let estimates = active
            .as_deref()
            .and_then(|key| estimate_state(&self.accounts, key))
            .map(|e| Estimates {
                ask_at: e.ask_at,
                has_leads: !e.leads.is_empty(),
                ask_estimates: e.should_ask_estimates.into_iter().map(lead_key).collect(),
                new_leads: e.new_leads.into_iter().map(lead_key).collect(),
            });
        let cycle = self.accounts.handovers().and_then(|h| h.desk().cycle());
        Snapshot {
            accounts,
            estimates,
            cycle: cycle_view(cycle.as_ref(), active.as_deref()),
            cycle_id: cycle.map(|c| c.id),
            active,
        }
    }

    fn desk_of(&self) -> Result<Arc<Handovers>, DeskError> {
        self.accounts.handovers().ok_or(DeskError::NoCycle)
    }
}

#[async_trait]
impl World for Live {
    async fn snapshot(&self) -> Snapshot {
        self.blocking(Live::read).await.unwrap_or(Snapshot {
            accounts: Vec::new(),
            active: None,
            estimates: None,
            cycle: CycleView::None,
            cycle_id: None,
        })
    }

    async fn start_cycle(
        &self,
        account: &str,
        mode: CycleMode,
        ask_at: Option<f64>,
        use_now: Option<f64>,
    ) -> Result<(), String> {
        let account = account.to_owned();
        self.desk_call(move |me| {
            let request = StartRequest {
                account,
                mode,
                k: me.accounts.config().k,
                ask_at,
                use_now,
            };
            me.desk_of()?.start(request, &me.accounts)
        })
        .await
    }

    async fn ask_all(&self, use_now: Option<f64>) -> Result<(), String> {
        self.desk_call(move |me| me.desk_of()?.ask_all(use_now, &me.accounts))
            .await
    }

    async fn cut_off_rest(&self, use_now: Option<f64>) -> Result<(), String> {
        self.desk_call(move |me| me.desk_of()?.cut_off_rest(use_now, &me.accounts))
            .await
    }

    async fn re_ask(&self, lead: &LeadKey) -> Result<(), String> {
        let lead = lead.clone();
        self.desk_call(move |me| {
            me.desk_of()?
                .re_ask_estimate((&lead.board, &lead.thread), &me.accounts)
        })
        .await
    }

    async fn add_lead(&self, lead: &LeadKey) -> Result<(), String> {
        let lead = lead.clone();
        self.desk_call(move |me| {
            let desk = me.desk_of()?;
            let working = desk.desk().working_leads();
            let found = working
                .iter()
                .find(|l| l.board == lead.board && l.thread == lead.thread)
                .ok_or(DeskError::NoCycle)?;
            desk.add_lead(found, &me.accounts)
        })
        .await
    }

    async fn cancel_cycle(&self) {
        let _ = self
            .blocking(|me| {
                if let Some(desk) = me.accounts.handovers() {
                    if let Err(e) = desk.desk().cancel() {
                        me.warn("handover cycle not cancelled", &e.to_string());
                    }
                }
            })
            .await;
    }

    async fn publish(&self, doc: Value, lines: Vec<String>) {
        let _ = self
            .blocking(move |me| {
                if let Err(e) = me.accounts.put_switch(doc, &lines) {
                    me.warn("switch doc not written", &e);
                }
            })
            .await;
    }

    async fn note(&self, lines: Vec<String>) {
        let _ = self
            .blocking(move |me| me.accounts.note_switch(&lines))
            .await;
    }

    async fn shown(&self) -> Option<Value> {
        self.blocking(|me| shown_switch(&me.svc)).await.ok().flatten()
    }

    async fn toast(&self, level: Level, title: &str, text: &str) {
        let Some(notify) = &self.svc.notify else {
            return;
        };
        notify.send(PingRequest {
            title: Some(title.to_owned()),
            text: text.to_owned(),
            level,
            project: None,
            thread: None,
            by: By::Auto,
            url: Some(format!(
                "http://127.0.0.1:{}/?tab=accounts",
                self.svc.port()
            )),
        });
    }

    async fn refresh(&self) {
        match tokio::time::timeout(REFRESH_LIMIT, self.accounts.poll_now()).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => self.warn("usage not refreshed", &e),
            Err(_) => self.warn("usage not refreshed", "the usage read took too long"),
        }
    }

    async fn learn(&self, cycle_id: &str) {
        let id = cycle_id.to_owned();
        let _ = self
            .blocking(move |me| {
                let cycle = me
                    .accounts
                    .handovers()
                    .and_then(|h| h.desk().cycle())
                    .filter(|c| c.id == id);
                if let Some(cycle) = cycle {
                    crate::services::learn_from_cycle(&me.svc, &me.accounts, &cycle);
                }
            })
            .await;
    }

    async fn switch(&self, to: &str, handover: bool) -> Result<(), String> {
        switch_to(&self.svc, to, Origin::Rule, handover).await
    }

    fn switching(&self) -> bool {
        is_running()
    }

    fn warn(&self, what: &str, detail: &str) {
        self.svc
            .log
            .warn(what, json!({ "error": detail, "by": "accounts rules" }));
    }
}
