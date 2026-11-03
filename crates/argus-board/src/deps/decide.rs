//! The state machine, as a pure function of the ledger record and what the two boards show.

use argus_core::clock::parse_ms;

use super::mirror::entry_of;
use super::model::{Dep, DepState};
use super::sight::{Look, Sight};

/// An `opening` dependency this old that cannot be finished is cancelled.
const OPENING_PATIENCE_MS: i64 = 60_000;

/// What reconcile should do with a dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Verdict {
    Stay,
    Move(DepState, Option<String>),
}

fn move_to(to: DepState, why: impl Into<String>) -> Verdict {
    Verdict::Move(to, Some(why.into()))
}

/// The transition a dependency should make now, if any.
pub(crate) fn decide(dep: &Dep, s: &Sight) -> Verdict {
    match dep.state {
        DepState::Opening => opening(dep, s),
        DepState::Open | DepState::Blocked => running(dep, s),
        DepState::Done => reopened(dep, s),
        DepState::Dropped | DepState::Gone | DepState::Cancelled => Verdict::Stay,
    }
}

fn requester_ended(s: &Sight) -> Option<String> {
    if s.retired.0 {
        return Some("requester project retired".to_owned());
    }
    match &s.requester {
        Look::NoBoard | Look::NoItem => Some("requester item gone".to_owned()),
        Look::Item(i) if i.removed() => Some(format!("requester item {}", i.shown_status())),
        Look::Item(i) if !i.open() => Some(format!("requester item {}", i.status)),
        Look::Item(_) => None,
    }
}

fn assignee_ended(s: &Sight) -> Option<String> {
    if s.retired.1 {
        return Some("project retired".to_owned());
    }
    match &s.assignee {
        Look::NoBoard => Some("board gone".to_owned()),
        Look::NoItem => Some("item removed".to_owned()),
        Look::Item(i) if i.removed() => Some(format!("item {}", i.shown_status())),
        Look::Item(_) => None,
    }
}

fn age_ms(dep: &Dep, now: i64) -> i64 {
    parse_ms(&dep.created_at).map_or(0, |at| now - at)
}

fn opening(dep: &Dep, s: &Sight) -> Verdict {
    if let Some(why) = requester_ended(s) {
        return move_to(DepState::Cancelled, why);
    }
    match &s.assignee {
        Look::Item(i) if !i.removed() => Verdict::Move(DepState::Open, None),
        _ if age_ms(dep, s.now) > OPENING_PATIENCE_MS => {
            move_to(DepState::Cancelled, "the request never opened")
        }
        _ => Verdict::Stay,
    }
}

fn running(dep: &Dep, s: &Sight) -> Verdict {
    if let Some(why) = requester_ended(s) {
        return move_to(DepState::Cancelled, why);
    }
    if let Some(why) = assignee_ended(s) {
        return move_to(DepState::Gone, why);
    }
    let Some(item) = s.assignee.item() else {
        return Verdict::Stay;
    };
    match (item.status.as_str(), dep.state) {
        ("done", _) => Verdict::Move(DepState::Done, None),
        ("dropped", _) => Verdict::Move(DepState::Dropped, None),
        ("doing", DepState::Blocked) => Verdict::Move(DepState::Open, None),
        _ => Verdict::Stay,
    }
}

/// A done dependency whose assignee item is open again and whose link is still open comes back
/// to `open`, unless that would close a loop.
fn reopened(dep: &Dep, s: &Sight) -> Verdict {
    let link_open = matches!(
        &s.requester,
        Look::Item(r) if r.open() && entry_of(&r.waits_on, &dep.id).is_some()
    );
    let item_open = matches!(&s.assignee, Look::Item(a) if a.open());
    if link_open && item_open && !s.loops {
        Verdict::Move(DepState::Open, None)
    } else {
        Verdict::Stay
    }
}
