//! Who hears of a break, and the findings grouped for one told line.

use super::{Ctx, Finding, Kind, OWNER};
use crate::board::{BoardView, ItemView};
use crate::item_wakes::target_for;
use crate::policy::coordinator_in;

fn assignee_target(board: &BoardView, item: &ItemView) -> Option<String> {
    let assignee = item.assignee.as_str();
    match assignee {
        "" => target_for(board, item.thread.as_str()),
        OWNER => Some(OWNER.to_owned()),
        _ => match assignee.strip_prefix("agent:") {
            Some(agent) => {
                let owner = board.agents.get(agent).map_or("", |a| a.thread.as_str());
                target_for(board, owner)
            }
            None => target_for(board, assignee),
        },
    }
}

/// The thread an alert of a column goes to: the lead for `item-thread` and `assignee`, which a column has
/// none of.
pub(super) fn column_recipient(board: &BoardView, to: &str) -> Option<String> {
    match to {
        "item-thread" | "assignee" | "lead" => coordinator_in(Some(board)),
        OWNER => Some(OWNER.to_owned()),
        thread => target_for(board, thread),
    }
}

/// The thread (or `owner`) an alert of an item goes to; `None` when nobody can hear it.
pub(super) fn recipient(ctx: &Ctx<'_>, item: &ItemView, to: &str) -> Option<String> {
    match to {
        "item-thread" => target_for(ctx.board, item.thread.as_str()),
        "assignee" => assignee_target(ctx.board, item),
        _ => column_recipient(ctx.board, to),
    }
}

/// Is the thread paused, so that it hears nothing until it resumes?
pub(super) fn paused(board: &BoardView, thread: &str) -> bool {
    board
        .threads
        .get(thread)
        .is_some_and(|t| t.status.is("paused"))
}

/// The findings of one project, one recipient and one kind that are told in one line, at one pace.
#[derive(Debug, Clone)]
pub(in crate::item_wakes) struct Group {
    pub(in crate::item_wakes) scope: String,
    pub(in crate::item_wakes) thread: String,
    pub(in crate::item_wakes) kind: Kind,
    pub(in crate::item_wakes) wake: bool,
    pub(in crate::item_wakes) repeat_ms: i64,
    pub(in crate::item_wakes) list: Vec<Finding>,
}

impl Group {
    fn takes(&self, f: &Finding) -> bool {
        self.scope == f.stale.scope
            && self.thread == f.stale.thread
            && self.kind == f.kind
            && self.wake == f.wake
            && self.repeat_ms == f.repeat_ms
    }

    /// The reason of the wake row: a dwell keeps `kanban-stale`, the other kinds are `kanban-alert`; an
    /// alert that does not wake is a `kanban-notice`, which lands in the inbox only.
    pub(in crate::item_wakes) fn reason(&self) -> &'static str {
        match (self.wake, self.kind) {
            (false, _) => "kanban-notice",
            (true, Kind::Dwell) => "kanban-stale",
            (true, _) => "kanban-alert",
        }
    }

    /// What the wakes table remembers the line by: the thread for a dwell (as before), else the kind.
    pub(in crate::item_wakes) fn subject(&self) -> String {
        match self.kind {
            Kind::Dwell => self.thread.clone(),
            other => other.key().to_owned(),
        }
    }

    /// The key of the repeat state in memory.
    pub(in crate::item_wakes) fn memory_key(&self) -> String {
        let base = format!("{}/{}", self.scope, self.thread);
        match self.kind {
            Kind::Dwell => base,
            other => format!("{base}/{}", other.key()),
        }
    }

    /// Is the line for the owner, who hears it as a notice and not as a wake?
    pub(in crate::item_wakes) fn for_owner(&self) -> bool {
        self.thread == OWNER
    }
}

/// The findings grouped, in order of first appearance.
pub(in crate::item_wakes) fn groups(findings: Vec<Finding>) -> Vec<Group> {
    let mut out: Vec<Group> = Vec::new();
    for f in findings {
        if let Some(group) = out.iter_mut().find(|g| g.takes(&f)) {
            group.list.push(f);
            continue;
        }
        out.push(Group {
            scope: f.stale.scope.clone(),
            thread: f.stale.thread.clone(),
            kind: f.kind,
            wake: f.wake,
            repeat_ms: f.repeat_ms,
            list: vec![f],
        });
    }
    out
}
