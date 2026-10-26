//! Step 3: one id, one record. Two open records with one Claude agent id are one agent (the
//! lead's `agent start` and the hook's record): the hook-bound record stays and takes the lead's
//! fields, the other is closed `stopped` with `duplicateOf`.

use super::pass::{IDLE_KEYS, ORPHAN_KEYS, Pass};
use super::report::Class;
use crate::iso::format_iso;
use crate::record::{Patch, Record, claude_id_of, filled, id_of, is_set, text};
use crate::text::same_model;
use serde_json::Value;

const LEAD_FIELDS: [&str; 9] = [
    "name",
    "task",
    "model",
    "modelWhy",
    "kind",
    "item",
    "branch",
    "worktree",
    "checkinMin",
];

fn rank(agent: &Record) -> u8 {
    let hook = if text(agent, "source") == Some("hook") {
        0
    } else {
        4
    };
    let path = if is_set(agent, "transcriptFile") {
        0
    } else {
        2
    };
    let bound = u8::from(!is_set(agent, "leadBound"));
    hook + path + bound
}

fn number_of(id: &str) -> i64 {
    id.get(2..).and_then(|n| n.parse().ok()).unwrap_or(0)
}

fn carries(value: Option<&Value>) -> bool {
    value.is_some_and(|v| !v.is_null() && v.as_str() != Some(""))
}

/// The fields the lead wrote that the hook-bound record does not have yet.
fn lead_fields(keeper: &Record, dup: &Record) -> Patch {
    let mut merge = Patch::new().with("leadBound", true);
    for field in LEAD_FIELDS {
        let value = dup.get(field);
        if carries(value) && keeper.get(field) != value {
            merge = merge.with(field, value.cloned().unwrap_or(Value::Null));
        }
    }
    let asked = merge
        .entries()
        .iter()
        .find(|(k, _)| k == "model")
        .and_then(|(_, v)| v.as_ref()?.as_str().map(str::to_owned));
    match (asked, filled(keeper, "modelUsed")) {
        (Some(asked), Some(used)) => merge.with("modelMismatch", !same_model(&asked, used)),
        _ => merge,
    }
}

impl<'a> Pass<'a> {
    fn groups_by_claude_id(&self) -> Vec<(String, Vec<&'a Record>)> {
        let mut groups: Vec<(String, Vec<&'a Record>)> = Vec::new();
        for agent in self.all.iter().copied() {
            let Some(id) = claude_id_of(&self.cur(agent)) else {
                continue;
            };
            match groups.iter_mut().find(|(g, _)| *g == id) {
                Some((_, list)) => list.push(agent),
                None => groups.push((id, vec![agent])),
            }
        }
        groups
    }

    pub(super) fn dedupe(&mut self) {
        for (id, mut records) in self.groups_by_claude_id() {
            if records.len() < 2 || !records.iter().any(|a| Self::is_open(a)) {
                continue;
            }
            records.sort_by_key(|a| (rank(a), number_of(id_of(a))));
            let Some((keeper, others)) = records.split_first() else {
                continue;
            };
            for dup in others.iter().filter(|d| Self::is_open(d)) {
                if text(dup, "source") != Some("hook") {
                    self.patch(keeper, lead_fields(keeper, dup));
                }
                self.close_duplicate(keeper, dup, &id);
            }
        }
    }

    fn close_duplicate(&mut self, keeper: &Record, dup: &Record, claude_id: &str) {
        let keeper_id = id_of(keeper);
        let closed = Patch::new()
            .with("status", "stopped")
            .with("finishedAt", format_iso(self.at))
            .with(
                "note",
                format!("duplicate of {keeper_id} (same Claude agent id), closed automatically"),
            )
            .with("duplicateOf", keeper_id)
            .with("closedBy", "reconciler")
            .without_all(&ORPHAN_KEYS)
            .without_all(&IDLE_KEYS);
        self.patch(dup, closed);
        self.closed_here.insert(id_of(dup).to_owned());
        self.counts.duplicates += 1;
        self.counts.closed += 1;
        let detail = format!(
            "same Claude agent id {claude_id} as {keeper_id}: closed, {keeper_id} keeps the record"
        );
        self.finding(dup, Class::Duplicate, detail);
    }
}
