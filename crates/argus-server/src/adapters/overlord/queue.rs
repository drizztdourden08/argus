//! The owner's Queue view of the hosted Overlord: what waits for it. The wakes of its lead thread,
//! the messages held for the child, the approvals nobody answered and the alerts (open notices of its board
//! and the stall lines). Each part is read from its own source; a missing source gives an empty part.

use argus_board::store::read_board;
use argus_overlord::approvals::Status;
use argus_overlord::view::{held_rows, notice_rows, stall_row, wake_row};
use argus_wake::service::driver::lock;
use serde_json::{Value, json};

use super::approvals::THREAD;
use crate::Services;
use argus_board::roles::overlord_key;

/// How many approvals the gate is asked for; the pending ones are filtered out of them.
const APPROVALS_READ: usize = 100;

fn wakes(svc: &Services) -> Vec<Value> {
    let Some(wake) = &svc.wake else {
        return Vec::new();
    };
    let board = overlord_key(&svc.paths.coord_home);
    let pending = lock(wake).registry().pending();
    pending
        .iter()
        .filter(|(scope, thread, _)| *scope == board && thread == THREAD)
        .map(|(_, _, p)| wake_row(p.reason.as_deref(), p.text.as_deref(), &p.at))
        .collect()
}

fn held(svc: &Services) -> Vec<Value> {
    svc.overlord
        .link()
        .map(|link| held_rows(&link.held()))
        .unwrap_or_default()
}

fn approvals(svc: &Services) -> Vec<Value> {
    svc.overlord_approvals
        .recent(APPROVALS_READ)
        .into_iter()
        .filter(|a| a.status == Status::Pending)
        .map(|a| json!(a))
        .collect()
}

fn stalls(svc: &Services) -> Vec<Value> {
    let Some(wake) = &svc.wake else {
        return Vec::new();
    };
    let open = lock(wake).stalls(None);
    open.iter()
        .filter(|s| s.acked_at.is_none())
        .map(|s| stall_row(&s.project, &s.text, &s.since))
        .collect()
}

fn notices(svc: &Services) -> Vec<Value> {
    let id = overlord_key(&svc.paths.coord_home);
    let board = svc
        .boards()
        .board(&id)
        .or_else(|| read_board(&svc.paths.coord_home, &id).ok().flatten());
    board.map(|b| notice_rows(&b)).unwrap_or_default()
}

/// `{wakes, held, approvals, alerts}`. It reads the wake registry and the board, so a caller on the runtime
/// goes through `spawn_blocking`.
#[must_use]
pub fn body(svc: &Services) -> Value {
    let alerts: Vec<Value> = stalls(svc).into_iter().chain(notices(svc)).collect();
    json!({
        "wakes": wakes(svc),
        "held": held(svc),
        "approvals": approvals(svc),
        "alerts": alerts,
    })
}
