//! The boards' side of the scanner: a board that was exported again is judged again (a keep, a
//! request, an answer reach the rows), and changed rules ask for a scan in a moment.

use std::sync::{Arc, Weak};

use argus_core::{Coll, DocsEvent};
use tokio::sync::mpsc::UnboundedSender;

use super::publish::publish;
use crate::Services;

/// What a board export touches that a worktree row or a rule can come from.
fn about_boards(event: &DocsEvent) -> bool {
    event.changes.iter().any(|c| match c.change.coll {
        Coll::Log | Coll::Entries | Coll::Threads => true,
        Coll::Meta => c.change.id != "worktrees",
        _ => false,
    })
}

/// Listens to the doc applies for as long as the services live. A change of rules sends the
/// project to `scans`.
pub(super) fn watch(svc: &Arc<Services>, scans: UnboundedSender<String>) {
    let weak: Weak<Services> = Arc::downgrade(svc);
    let link = svc.bus.on_docs(move |event| {
        let Some(svc) = weak.upgrade().filter(|s| !s.stop.is_cancelled()) else {
            return;
        };
        let Some(service) = svc.worktrees.clone().filter(|_| about_boards(event)) else {
            return;
        };
        if service.rules_changed(&event.scope) {
            let _ = scans.send(event.scope.clone());
        }
        publish(&svc, &service, &event.scope);
    });
    svc.keep(link);
}
