//! The boot revival of the hosted Overlord (I-171). The desktop Overlord messaged each lead with
//! no listener after a machine boot; with the hosted one Argus does it: after the boot has
//! settled it writes the one-off Desktop resume task of the account switch (`argus-resume-*`,
//! through `argus_accounts::resume::write_task`) for the leads without a listener. Each lead is
//! recorded for the boot, so a second start of Argus in the same boot adds nothing. The rule is in
//! `boot_rule.rs`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use argus_accounts::handover::HandoverState;
use argus_accounts::resume::{ResumeLead, ResumeTask, TASK_CWD, build_prompt, write_task};
use argus_accounts::{Scope, find_app_data};
use argus_core::clock::now_ms;
use argus_wake::service::driver::lock;
use serde_json::{Value, json};

use super::boot_rule::{Lead, Mode, lead_key, needing_resume, same_boot};
use super::config;
use crate::Services;
use crate::boards::coordinator_in;
use crate::services::accounts::last_known_account;
use crate::services::accounts_resume::{chat_id, overlord_chat, skill_root};

const KEY: &str = "overlord.boot-resume";
/// How long Argus waits after its start, so the leads that are alive can connect their listeners.
const SETTLE: Duration = Duration::from_secs(120);
/// How long after the check the task fires.
const FIRE_AFTER_MS: i64 = 60_000;

fn mode(svc: &Services) -> Mode {
    if svc.shadow {
        Mode::Shadow
    } else if config::is_live(svc) {
        Mode::Live
    } else {
        Mode::Off
    }
}

/// The boot time of the machine, milliseconds since the epoch.
#[cfg(windows)]
fn boot_ms() -> Option<i64> {
    i64::try_from(argus_native::Boot::current().time_ms()).ok()
}

#[cfg(not(windows))]
fn boot_ms() -> Option<i64> {
    None
}

/// The leads already resumed in the boot `boot`.
fn recorded(svc: &Services, boot: i64) -> Vec<String> {
    let text = svc.db.kv_get(KEY).ok().flatten().unwrap_or_default();
    let Ok(doc) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    if !doc["boot"].as_i64().is_some_and(|at| same_boot(at, boot)) {
        return Vec::new();
    }
    doc["leads"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn record(svc: &Services, boot: i64, keys: &[String]) -> Result<(), String> {
    let text = json!({ "boot": boot, "leads": keys }).to_string();
    svc.db.kv_set(KEY, &text).map_err(|e| e.to_string())
}

fn listening(svc: &Services, board: &str, thread: &str) -> bool {
    let Some(wake) = &svc.wake else {
        return false;
    };
    lock(wake)
        .listeners(board)
        .iter()
        .any(|l| l.thread == thread || l.thread == "*")
}

/// The coordinator thread of a board as a lead, when it has a chat to message.
fn lead_of(svc: &Services, key: &str) -> Option<Lead> {
    let board = svc.boards().board(key)?;
    let thread = coordinator_in(&board)?;
    let record = &board["threads"][thread.as_str()];
    let session_id = chat_id(record["link"].as_str(), record["sessionId"].as_str())?;
    Some(Lead {
        listening: listening(svc, key, &thread),
        board: key.to_owned(),
        thread,
        session_id,
    })
}

/// Every project lead but the Overlord's own.
fn leads(svc: &Services) -> Vec<Lead> {
    let boards = svc.boards();
    boards
        .projects()
        .iter()
        .filter(|row| row.has_board && !boards.is_overlord(&row.key))
        .filter_map(|row| lead_of(svc, &row.key))
        .collect()
}

/// The Desktop data folder and the scope of the account that is logged in now.
fn active_scope(svc: &Services) -> Result<(PathBuf, Scope), String> {
    let accounts = svc
        .accounts
        .as_ref()
        .ok_or("the accounts service is not running")?;
    let config = accounts.config();
    let app = find_app_data(config.app_data.as_deref())
        .ok_or("the Claude Desktop data folder was not found")?;
    let active = last_known_account(&app).ok_or("no Desktop account is logged in")?;
    let (account, org) = config
        .accounts
        .iter()
        .find(|a| a.desktop_account.as_deref() == Some(active.as_str()))
        .and_then(|a| Some((a.desktop_account.clone()?, a.desktop_org.clone()?)))
        .ok_or("the active account has no Desktop scope")?;
    Ok((app, Scope { account, org }))
}

fn resume_lead(lead: &Lead) -> ResumeLead {
    ResumeLead {
        board: lead.board.clone(),
        thread: lead.thread.clone(),
        session_id: lead.session_id.clone(),
        file: None,
        state: HandoverState::CutOff,
    }
}

fn write(svc: &Services, boot: i64, picks: &[Lead]) -> Result<String, String> {
    let (app, scope) = active_scope(svc)?;
    let list: Vec<ResumeLead> = picks.iter().map(resume_lead).collect();
    let task = ResumeTask {
        id: format!("argus-resume-boot-{boot}"),
        fire_at_ms: now_ms() + FIRE_AFTER_MS,
        cwd: TASK_CWD.to_owned(),
        notify_session_id: overlord_chat(svc),
        prompt: build_prompt(&list, svc.port()),
    };
    write_task(&app, &scope, &task, &skill_root())?;
    Ok(task.id)
}

/// One check: writes the task for the leads that need it and records them for this boot.
fn run(svc: &Services) {
    let Some(boot) = boot_ms() else {
        return;
    };
    let mut done = recorded(svc, boot);
    let picks = needing_resume(mode(svc), &leads(svc), &done);
    if picks.is_empty() {
        return;
    }
    match write(svc, boot, &picks) {
        Ok(id) => {
            done.extend(picks.iter().map(lead_key));
            let saved = record(svc, boot, &done);
            svc.log.info(
                "boot resume written",
                json!({ "task": id, "leads": picks.len(), "recorded": saved.is_ok() }),
            );
        }
        Err(error) => svc
            .log
            .warn("boot resume not written", json!({ "error": error })),
    }
}

/// Checks once, after the settle time, when the mode is `live`.
pub(crate) fn spawn(svc: &Arc<Services>) {
    if mode(svc) != Mode::Live {
        return;
    }
    let for_task = Arc::clone(svc);
    let task = tokio::spawn(async move {
        tokio::time::sleep(SETTLE).await;
        let _ = tokio::task::spawn_blocking(move || run(&for_task)).await;
    });
    svc.tasks.keep("overlord-boot-resume", task);
}
