//! The boards' side of the scanner: a board that was exported again is judged again (a keep, a
//! request, an answer reach the rows), and changed rules or an agent that ended ask for a scan in a
//! moment (the hygiene step follows every scan).

use std::sync::{Arc, Weak};

use tokio::sync::mpsc::UnboundedSender;

use super::publish::publish;
use crate::Services;

/// Listens to the board exports for as long as the services live. A change of rules or an agent that ended sends the
/// project to `scans`.
pub(super) fn watch(svc: &Arc<Services>, scans: UnboundedSender<String>) {
    let Some(sync) = svc.sync.clone() else {
        return;
    };
    let weak: Weak<Services> = Arc::downgrade(svc);
    sync.on_board(move |key| {
        let Some(svc) = weak.upgrade().filter(|s| !s.stop.is_cancelled()) else {
            return;
        };
        let Some(service) = svc.worktrees.clone() else {
            return;
        };
        let rules = service.rules_changed(key);
        let ended = service.agents_ended(key);
        if rules || ended {
            let _ = scans.send(key.to_owned());
        }
        publish(&svc, &service, key);
    });
}
