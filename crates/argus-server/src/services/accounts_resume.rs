//! The resume after an account switch, server side: the one-off scheduled task that wakes the
//! leads of the handover cycle, and the watcher that ends it. The switch calls
//! [`schedule_for_cycle`] with Claude closed, just before starting it; the task fires a few
//! minutes later in the Overlord's session and reports each lead to
//! `POST /api/accounts/handover` (`resumed` or `not_resumed`). The watcher never touches the
//! scheduled-tasks file, which the app owns while it runs: once the task ran and the leads
//! reported, or after 30 minutes, it marks the task done in Argus ("done, remove at next close")
//! and marks the leads that never reported as not resumed so the cycle ends in its final state.
//! [`cleanup_pending`] removes a finished task, and the switch calls it while Claude is closed.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use argus_accounts::handover::{Cycle, Handover};
use argus_accounts::resume::{
    ResumeLead, ResumeTask, TASK_CWD, build_prompt, has_run, remove_task, write_task,
};
use argus_accounts::{Scope, find_app_data};
use argus_core::clock::now_ms;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::tasks::Timer;
use crate::Services;

const NAME: &str = "accounts";
const PENDING_KEY: &str = "accounts.resume.pending";
/// How long after the switch the task fires.
const FIRE_AFTER_MS: i64 = 3 * 60_000;
/// The longest the resume may take from the moment it fires.
const GIVE_UP_MS: i64 = 30 * 60_000;
/// How long the leads get to report after the task started.
const REPORT_MS: i64 = 10 * 60_000;
const WATCH_EVERY: Duration = Duration::from_secs(30);

/// The task waiting for its end, kept in the database so a restart of Argus does not lose it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pending {
    id: String,
    scope: Scope,
    fire_at: i64,
    #[serde(default)]
    ran_at: Option<i64>,
    /// The task is finished; its entry and folder go at the next close of Claude.
    #[serde(default)]
    done: bool,
}

/// What the watcher does with the pending task now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Wait,
    /// Remember when the app was seen to have run the task.
    Ran,
    Finish,
}

/// Decides from what the watcher saw: `ran` is the app's `lastRunAt`, `reported` whether every
/// lead of the cycle has an answer.
fn verdict(pending: &Pending, ran: bool, reported: bool, now: i64) -> Verdict {
    if reported || now >= pending.fire_at + GIVE_UP_MS {
        return Verdict::Finish;
    }
    match (pending.ran_at, ran) {
        (Some(at), _) if now >= at + REPORT_MS => Verdict::Finish,
        (None, true) => Verdict::Ran,
        _ => Verdict::Wait,
    }
}

fn skill_root() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    PathBuf::from(home).join(".claude").join("scheduled-tasks")
}

/// The Desktop chat id (`local_<uuid>`) of a thread: the tail of its link, else its session id.
fn chat_id(link: Option<&str>, session: Option<&str>) -> Option<String> {
    link.and_then(|l| l.rsplit('/').next())
        .filter(|t| t.starts_with("local_"))
        .or(session)
        .map(str::to_owned)
}

fn lead_of(h: &Handover) -> Option<ResumeLead> {
    Some(ResumeLead {
        board: h.board.clone(),
        thread: h.thread.clone(),
        session_id: chat_id(h.link.as_deref(), h.session_id.as_deref())?,
        file: h.file.clone(),
        state: h.state,
    })
}

/// The leads of `cycle` that wait for the resume and can be messaged.
fn resume_leads(cycle: &Cycle) -> Vec<ResumeLead> {
    cycle
        .leads
        .iter()
        .filter(|h| h.state.awaits_resume())
        .filter_map(lead_of)
        .collect()
}

/// The Desktop chat of the Overlord: the live lead of the portfolio board.
fn overlord_chat(svc: &Services) -> Option<String> {
    let boards = svc.boards();
    let thread = boards.overlord_thread()?;
    let board = boards.overlord_board()?;
    let record = &board["threads"][thread.as_str()];
    if record["host"].as_str() == Some("argus") {
        return None;
    }
    chat_id(record["link"].as_str(), record["sessionId"].as_str())
}

fn save_pending(svc: &Services, pending: Option<&Pending>) -> Result<(), String> {
    let text = match pending {
        Some(p) => serde_json::to_string(p).map_err(|e| e.to_string())?,
        None => String::new(),
    };
    svc.db.kv_set(PENDING_KEY, &text).map_err(|e| e.to_string())
}

fn load_pending(svc: &Services) -> Option<Pending> {
    let text = svc.db.kv_get(PENDING_KEY).ok().flatten()?;
    serde_json::from_str(&text).ok()
}

/// Writes the resume task for the leads of the handover cycle that wait for it, to fire three
/// minutes from now in the Desktop app of `target`. Called with Claude closed. Returns the task
/// id, or `None` when no lead waits for the resume.
///
/// # Errors
/// No accounts service, no handover desk, the Desktop data folder is not found, or a file cannot
/// be written.
pub fn schedule_for_cycle(svc: &Services, target: &Scope) -> Result<Option<String>, String> {
    let accounts = svc
        .accounts
        .clone()
        .ok_or("the accounts service is not running")?;
    let desk = accounts
        .handovers()
        .ok_or("the handover desk has not started")?;
    let Some(cycle) = desk.desk().cycle() else {
        return Ok(None);
    };
    let leads = resume_leads(&cycle);
    if leads.is_empty() {
        return Ok(None);
    }
    let app = find_app_data(accounts.config().app_data.as_deref())
        .ok_or("the Claude Desktop data folder was not found")?;
    if let Some(old) = load_pending(svc).filter(|p| p.done) {
        cleanup_pending(svc, &old.scope)?;
    }
    let fire_at = now_ms() + FIRE_AFTER_MS;
    let task = ResumeTask {
        id: format!("argus-resume-{}", cycle.id.to_ascii_lowercase()),
        fire_at_ms: fire_at,
        cwd: TASK_CWD.to_owned(),
        notify_session_id: overlord_chat(svc),
        prompt: build_prompt(&leads, svc.port()),
    };
    write_task(&app, target, &task, &skill_root())?;
    let pending = Pending {
        id: task.id.clone(),
        scope: target.clone(),
        fire_at,
        ran_at: None,
        done: false,
    };
    save_pending(svc, Some(&pending))?;
    Ok(Some(task.id))
}

/// Marks every handover that still waits as not resumed; the cycle ends in its final state.
fn close_cycle(svc: &Services) {
    let Some(desk) = svc.accounts.as_ref().and_then(|a| a.handovers()) else {
        return;
    };
    let Some(cycle) = desk.desk().cycle() else {
        return;
    };
    for h in cycle.leads.iter().filter(|h| h.state.awaits_resume()) {
        if let Err(e) = desk.desk().mark_resumed(&h.id, false) {
            svc.log.warn(
                "accounts resume: lead not closed",
                json!({ "handover": h.id, "error": e.to_string() }),
            );
        }
    }
}

fn all_reported(svc: &Services) -> bool {
    svc.accounts
        .as_ref()
        .and_then(|a| a.handovers())
        .and_then(|d| d.desk().cycle())
        .is_none_or(|c| !c.leads.iter().any(|h| h.state.awaits_resume()))
}

/// Ends the resume in Argus only: the leads are closed and the task is marked done. Its files
/// stay until [`cleanup_pending`] runs with Claude closed.
fn finish(svc: &Services, pending: &Pending) {
    close_cycle(svc);
    let done = Pending {
        done: true,
        ..pending.clone()
    };
    if let Err(e) = save_pending(svc, Some(&done)) {
        svc.log
            .error("accounts resume state not saved", json!({ "error": e }));
    }
}

/// Removes the finished resume task of `scope` (its `SKILL.md` folder and its entry in
/// `scheduled-tasks.json`) and forgets it. Nothing happens when no finished task of that scope
/// waits. The switch calls this at its close step, with Claude closed; while Claude runs it
/// refuses and the task stays pending.
///
/// # Errors
/// No accounts service, the Desktop data folder is not found, Claude is running, or a file
/// cannot be changed.
pub fn cleanup_pending(svc: &Services, scope: &Scope) -> Result<(), String> {
    let Some(pending) = load_pending(svc).filter(|p| p.done && p.scope == *scope) else {
        return Ok(());
    };
    let accounts = svc
        .accounts
        .as_ref()
        .ok_or("the accounts service is not running")?;
    let app = find_app_data(accounts.config().app_data.as_deref())
        .ok_or("the Claude Desktop data folder was not found")?;
    remove_task(&app, &pending.scope, &pending.id, &skill_root())?;
    save_pending(svc, None)
}

/// One look at the pending task.
fn watch_once(svc: &Services, now: i64) {
    let Some(pending) = load_pending(svc).filter(|p| !p.done) else {
        return;
    };
    let Some(app) = svc
        .accounts
        .as_ref()
        .and_then(|a| find_app_data(a.config().app_data.as_deref()))
    else {
        return;
    };
    let ran = has_run(&app, &pending.scope, &pending.id).unwrap_or(false);
    match verdict(&pending, ran, all_reported(svc), now) {
        Verdict::Wait => {}
        Verdict::Ran => {
            let seen = Pending {
                ran_at: Some(now),
                ..pending
            };
            if let Err(e) = save_pending(svc, Some(&seen)) {
                svc.log
                    .error("accounts resume state not saved", json!({ "error": e }));
            }
        }
        Verdict::Finish => finish(svc, &pending),
    }
}

/// Starts the watcher on the accounts timer.
pub(crate) fn start(svc: &Arc<Services>) {
    if !svc.tasks.runs(Timer::Accounts) {
        return;
    }
    let svc_for_task = Arc::clone(svc);
    let task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(WATCH_EVERY).await;
            let svc = Arc::clone(&svc_for_task);
            let _ = tokio::task::spawn_blocking(move || watch_once(&svc, now_ms())).await;
        }
    });
    svc.tasks.keep(NAME, task);
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use argus_accounts::Scope;
    use argus_accounts::handover::{Handover, HandoverState};

    use super::{GIVE_UP_MS, Pending, REPORT_MS, Verdict, chat_id, lead_of, verdict};

    fn pending(ran_at: Option<i64>) -> Pending {
        Pending {
            id: "argus-resume-sw-1".into(),
            scope: Scope {
                account: "a".into(),
                org: "o".into(),
            },
            fire_at: 1_000,
            ran_at,
            done: false,
        }
    }

    fn handover(state: HandoverState, link: Option<&str>) -> Handover {
        Handover {
            id: "H-1".into(),
            cycle: "SW-1".into(),
            board: "rotp".into(),
            thread: "lead".into(),
            session_id: Some("cli-1".into()),
            link: link.map(str::to_owned),
            state,
            estimate: None,
            file: Some("h.md".into()),
            agents: Vec::new(),
            estimate_asked_at: None,
            asked_at: None,
            ready_at: None,
            via: None,
        }
    }

    #[test]
    fn the_chat_id_comes_from_the_link_else_the_session() {
        let link = Some("claude://claude.ai/epitaxy/local_abc");
        assert_eq!(chat_id(link, Some("cli")).as_deref(), Some("local_abc"));
        assert_eq!(chat_id(None, Some("cli")).as_deref(), Some("cli"));
        assert_eq!(chat_id(None, None), None);
    }

    #[test]
    fn a_lead_without_any_session_cannot_be_messaged() {
        let mut h = handover(HandoverState::Ready, None);
        h.session_id = None;
        assert!(lead_of(&h).is_none());
        let ok = lead_of(&handover(HandoverState::CutOff, None)).unwrap();
        assert_eq!(ok.session_id, "cli-1");
        assert_eq!(ok.state, HandoverState::CutOff);
    }

    #[test]
    fn the_watcher_waits_until_the_task_ran_and_the_leads_had_time() {
        let p = pending(None);
        assert_eq!(verdict(&p, false, false, 2_000), Verdict::Wait);
        assert_eq!(verdict(&p, true, false, 2_000), Verdict::Ran);
        let seen = pending(Some(2_000));
        assert_eq!(
            verdict(&seen, true, false, 2_000 + REPORT_MS - 1),
            Verdict::Wait
        );
        assert_eq!(
            verdict(&seen, true, false, 2_000 + REPORT_MS),
            Verdict::Finish
        );
    }

    #[test]
    fn the_watcher_finishes_when_all_reported_or_after_thirty_minutes() {
        let p = pending(None);
        assert_eq!(verdict(&p, false, true, 1_500), Verdict::Finish);
        assert_eq!(
            verdict(&p, false, false, 1_000 + GIVE_UP_MS),
            Verdict::Finish
        );
    }

    #[test]
    fn a_pending_task_is_not_done_until_marked_and_the_mark_survives_the_database() {
        let old = r#"{"id":"argus-resume-sw-1","scope":{"account":"a","org":"o"},"fireAt":1}"#;
        let p: Pending = serde_json::from_str(old).unwrap();
        assert!(!p.done && p.ran_at.is_none());
        let done = Pending { done: true, ..p };
        let back: Pending = serde_json::from_str(&serde_json::to_string(&done).unwrap()).unwrap();
        assert!(back.done);
    }
}
