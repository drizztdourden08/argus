//! The boot revival of the hosted Overlord (I-171). The desktop Overlord messaged each lead with
//! no listener after a machine boot; with the hosted one Argus does it: after the boot has
//! settled it raises a wake for each lead without a listener, and the wake relay (I-161) delivers it. Each lead is
//! recorded for the boot, so a second start of Argus in the same boot adds nothing. The rule is in
//! `boot_rule.rs`.

use std::sync::Arc;
use std::time::Duration;

use argus_wake::service::driver::lock;
use serde_json::{Value, json};

use super::boot_rule::{Lead, Mode, lead_key, needing_resume, same_boot};
use super::config;
use crate::boards::coordinator_in;
use crate::{Services, WakeRequest};

const KEY: &str = "overlord.boot-resume";
/// How long Argus waits after its start, so the leads that are alive can connect their listeners.
const SETTLE: Duration = Duration::from_secs(120);
const REASON: &str = "boot-revival";
const TEXT: &str = "Argus restarted after a reboot; you had no listener. Run coord inbox.";

fn mode(svc: &Services) -> Mode {
    if svc.shadow || config::is_shadow(svc) {
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

/// The coordinator thread of a board as a lead.
fn lead_of(svc: &Services, key: &str) -> Option<Lead> {
    let board = svc.boards().board(key)?;
    let thread = coordinator_in(&board)?;
    Some(Lead {
        listening: listening(svc, key, &thread),
        board: key.to_owned(),
        thread,
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

/// The wake of a lead: the deaf-wake path of the wake relay (`docs/wake-relay.md`) takes it from
/// here, asking a live listener thread to `send_message` once and resuming it headless when none is.
fn wake_of(lead: &Lead) -> WakeRequest {
    WakeRequest::about(&lead.board, &lead.thread, REASON, "boot", TEXT.to_owned())
}

fn raise(svc: &Services, picks: &[Lead]) -> Result<(), String> {
    let wakes: Vec<WakeRequest> = picks.iter().map(wake_of).collect();
    svc.wake_hook().submit(&wakes)
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
    match raise(svc, &picks) {
        Ok(()) => {
            done.extend(picks.iter().map(lead_key));
            let saved = record(svc, boot, &done);
            svc.log.info(
                "boot resume raised",
                json!({ "leads": picks.len(), "recorded": saved.is_ok() }),
            );
        }
        Err(error) => svc
            .log
            .warn("boot resume not raised", json!({ "error": error })),
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
