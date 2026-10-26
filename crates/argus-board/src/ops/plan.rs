//! What a purge would remove, computed without touching the board.

use argus_core::clock::{now_ms, parse_ms};
use argus_core::js;
use serde_json::{Map, Value, json};

use super::announce::coordinator_ids;
use super::target::thread_arg;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{doc, ids, nullish, section, to_number, truthy};

/// The collections a purge removes records from.
pub(crate) const COLLS: [&str; 6] = ["threads", "items", "tests", "entries", "actions", "agents"];
const DAY_MS: f64 = 86_400_000.0;
const LEGACY_EPOCH_MS: i64 = 946_684_800_000;

/// The ids a purge removes, per collection, in the order they were selected.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    pub(crate) threads: Vec<String>,
    pub(crate) items: Vec<String>,
    pub(crate) tests: Vec<String>,
    pub(crate) entries: Vec<String>,
    pub(crate) actions: Vec<String>,
    pub(crate) agents: Vec<String>,
    /// Work-log lines to remove, per subject.
    pub(crate) log: Vec<(String, Vec<Value>)>,
    /// The threads whose inbox goes.
    pub(crate) inbox: Vec<String>,
}

impl Selection {
    /// The ids selected in a collection.
    pub(crate) fn list(&self, coll: &str) -> &[String] {
        match coll {
            "threads" => &self.threads,
            "items" => &self.items,
            "tests" => &self.tests,
            "entries" => &self.entries,
            "actions" => &self.actions,
            "agents" => &self.agents,
            _ => &[],
        }
    }

    fn list_mut(&mut self, coll: &str) -> Option<&mut Vec<String>> {
        match coll {
            "threads" => Some(&mut self.threads),
            "items" => Some(&mut self.items),
            "tests" => Some(&mut self.tests),
            "entries" => Some(&mut self.entries),
            "actions" => Some(&mut self.actions),
            "agents" => Some(&mut self.agents),
            _ => None,
        }
    }

    fn has(&self, coll: &str, id: &str) -> bool {
        self.list(coll).iter().any(|i| i == id)
    }

    fn add(&mut self, coll: &str, id: &str) {
        if let Some(list) = self.list_mut(coll) {
            if !list.iter().any(|i| i == id) {
                list.push(id.to_owned());
            }
        }
    }

    fn total(&self) -> usize {
        COLLS.iter().map(|c| self.list(c).len()).sum()
    }
}

/// A purge plan: what goes, the counts, and the criteria it was computed from. Valid for the
/// board it was computed on.
#[derive(Debug, Clone)]
pub struct PurgePlan {
    /// The ids that go.
    pub ids: Selection,
    /// How many of each kind go (`threads`, `items`, `ac`, `tests`, ...), in this order.
    pub counts: Map<String, Value>,
    /// The criteria the plan came from.
    pub criteria: Value,
}

fn named_threads(board: &Board, list: &Value) -> Result<Vec<String>, BoardError> {
    let Some(list) = list.as_array() else {
        return Err(refuse("keep and threads take a list of thread names."));
    };
    list.iter().map(|id| thread_arg(board, id)).collect()
}

fn age_ms(text: Option<String>) -> Option<i64> {
    match text {
        None => Some(LEGACY_EPOCH_MS),
        Some(t) => parse_ms(&t),
    }
}

fn select_keep(
    board: &Board,
    criteria: &Value,
    always: &[String],
    sel: &mut Selection,
) -> Result<(), BoardError> {
    if truthy(criteria, "threads") || truthy(criteria, "status") || !criteria["idleDays"].is_null()
    {
        return Err(refuse("keep cannot be combined with other purge criteria."));
    }
    let mut keep = named_threads(board, &criteria["keep"])?;
    keep.extend(always.iter().cloned());
    for id in ids(board, "threads") {
        if !keep.contains(&id) {
            sel.add("threads", &id);
        }
    }
    Ok(())
}

fn select_named(
    board: &Board,
    criteria: &Value,
    always: &[String],
    sel: &mut Selection,
) -> Result<(), BoardError> {
    for id in named_threads(board, &criteria["threads"])? {
        if always.contains(&id) {
            return Err(refuse(format!(
                "{id} is the coordinator or the calling thread and is never purged."
            )));
        }
        sel.add("threads", &id);
    }
    Ok(())
}

fn select_status(board: &Board, criteria: &Value, always: &[String], sel: &mut Selection) {
    let wanted: Vec<&str> = criteria["status"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let placeholder = wanted.contains(&"placeholder");
    for (id, t) in section(board, "threads") {
        let by_status = t["status"].as_str().is_some_and(|s| wanted.contains(&s));
        if !always.contains(id) && (by_status || (placeholder && truthy(t, "placeholder"))) {
            sel.add("threads", id);
        }
    }
}

fn select_idle(
    board: &Board,
    criteria: &Value,
    always: &[String],
    sel: &mut Selection,
) -> Result<(), BoardError> {
    let days = to_number(criteria.get("idleDays"));
    if !days.is_finite() || days < 0.0 {
        return Err(refuse("idleDays must be a number of days."));
    }
    let cutoff = cutoff_ms(days);
    for (id, t) in section(board, "threads") {
        let seen = nullish(t, "lastSeen").or_else(|| nullish(t, "registeredAt"));
        let idle = age_ms(seen).is_some_and(|ms| ms < cutoff);
        if !always.contains(id) && idle {
            sel.add("threads", id);
        }
    }
    Ok(())
}

fn cutoff_ms(days: f64) -> i64 {
    let span = js::number((days * DAY_MS).floor())
        .as_i64()
        .unwrap_or(i64::MAX);
    now_ms().saturating_sub(span)
}

fn select_trash(board: &Board, criteria: &Value, sel: &mut Selection) {
    let pick = |v: &Value| (!v.is_null()).then(|| to_number(Some(v)));
    let older = pick(&criteria["olderDays"])
        .or_else(|| pick(&criteria["older"]))
        .or_else(|| pick(&criteria["trash"]["olderDays"]))
        .unwrap_or(0.0);
    let cutoff = cutoff_ms(older);
    for coll in ["items", "tests", "entries", "actions", "agents"] {
        for (id, d) in section(board, coll) {
            let old_enough = older == 0.0
                || older.is_nan()
                || nullish(d, "deletedAt")
                    .and_then(|t| parse_ms(&t))
                    .is_some_and(|ms| ms < cutoff);
            if truthy(d, "deletedAt") && old_enough {
                sel.add(coll, id);
            }
        }
    }
}

fn referenced(sel: &Selection, e: &Value) -> bool {
    let reply = e["replyTo"].as_str();
    let by_reply = reply.is_some_and(|r| COLLS.iter().any(|c| sel.has(c, r)));
    let target = &e["target"];
    let by_target = truthy(e, "target")
        && target["c"]
            .as_str()
            .zip(target["id"].as_str())
            .is_some_and(|(c, id)| sel.has(c, id));
    by_reply || by_target
}

fn thread_in(sel: &Selection, d: &Value) -> bool {
    d["thread"].as_str().is_some_and(|t| sel.has("threads", t))
}

fn item_in(sel: &Selection, d: &Value) -> bool {
    d["item"].as_str().is_some_and(|i| sel.has("items", i))
}

fn merged_into_selected(sel: &Selection, t: &Value) -> bool {
    t["mergedInto"]
        .as_str()
        .is_some_and(|m| sel.has("threads", m))
}

fn owned(sel: &Selection, d: &Value) -> bool {
    thread_in(sel, d) || item_in(sel, d)
}

fn entry_goes(sel: &Selection, e: &Value) -> bool {
    owned(sel, e) || referenced(sel, e)
}

type Rule = fn(&Selection, &Value) -> bool;

/// What takes a record along with a purge, per collection.
const RULES: [(&str, Rule); 6] = [
    ("threads", merged_into_selected),
    ("items", thread_in),
    ("tests", owned),
    ("entries", entry_goes),
    ("actions", owned),
    ("agents", thread_in),
];

fn closure_pass(board: &Board, sel: &mut Selection) {
    for (coll, rule) in RULES {
        for (id, d) in section(board, coll) {
            if rule(sel, d) {
                sel.add(coll, id);
            }
        }
    }
}

fn closure(board: &Board, sel: &mut Selection) {
    loop {
        let before = sel.total();
        closure_pass(board, sel);
        if sel.total() == before {
            return;
        }
    }
}

fn selected_log(board: &Board, sel: &Selection) -> Vec<(String, Vec<Value>)> {
    let mut out = Vec::new();
    for (subject, d) in section(board, "log") {
        if subject == "audit" {
            continue;
        }
        let lines: Vec<Value> = d["entries"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|l| {
                l["thread"].as_str().is_some_and(|t| sel.has("threads", t))
                    || l["item"].as_str().is_some_and(|i| sel.has("items", i))
            })
            .cloned()
            .collect();
        if !lines.is_empty() {
            out.push((subject.clone(), lines));
        }
    }
    out
}

fn removed<'a>(board: &'a Board, sel: &Selection, coll: &str) -> Vec<&'a Value> {
    sel.list(coll)
        .iter()
        .filter_map(|id| doc(board, coll, id))
        .collect()
}

fn counts_of(board: &Board, sel: &Selection) -> Map<String, Value> {
    let entries = removed(board, sel, "entries");
    let kinds = |kind: &str| entries.iter().filter(|e| e["kind"] == kind).count();
    let ac: usize = removed(board, sel, "items")
        .iter()
        .map(|i| i["ac"].as_array().map_or(0, Vec::len))
        .sum();
    let proofs: usize = removed(board, sel, "tests")
        .iter()
        .map(|t| {
            t["proofs"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|p| truthy(p, "file"))
                .count()
        })
        .sum();
    let inbox: usize = sel
        .inbox
        .iter()
        .map(|id| board["inbox"][id.as_str()].as_array().map_or(0, Vec::len))
        .sum();
    let log_lines: usize = sel.log.iter().map(|(_, lines)| lines.len()).sum();
    let rows = [
        ("threads", sel.threads.len()),
        ("items", sel.items.len()),
        ("ac", ac),
        ("tests", sel.tests.len()),
        ("entries", sel.entries.len()),
        ("questions", kinds("question")),
        ("decisions", kinds("decision")),
        ("followups", kinds("followup")),
        ("notices", kinds("notice")),
        ("messages", kinds("message")),
        ("actions", sel.actions.len()),
        ("agents", sel.agents.len()),
        ("logLines", log_lines),
        ("inbox", inbox),
        ("proofs", proofs),
    ];
    rows.into_iter()
        .map(|(k, n)| (k.to_owned(), json!(n)))
        .collect()
}

/// What a purge would remove. `criteria` is one or more of: `threads` (ids), `keep` (ids) (every
/// other thread), `status` (a list) (a thread status or `placeholder`), `idleDays` n, `trash` (true,
/// with `olderDays` n). Coordinators and `protect` threads are never purged. A placeholder merged
/// into a purged thread goes with it. Everything the threads own goes too: items (their AC),
/// tests, entries (and every entry replying to or targeting a removed record), actions, the
/// thread's agents, work-log lines, inbox. The audit log is kept.
///
/// # Errors
/// Refused when no criterion is named, a named thread does not exist or is protected, or the
/// criteria contradict each other.
pub fn purge_plan(board: &Board, criteria: &Value) -> Result<PurgePlan, BoardError> {
    let mut always = coordinator_ids(board, false);
    always.extend(
        criteria["protect"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned),
    );
    let mut sel = Selection::default();
    let mut selecting = false;
    if truthy(criteria, "keep") {
        selecting = true;
        select_keep(board, criteria, &always, &mut sel)?;
    }
    if truthy(criteria, "threads") {
        selecting = true;
        select_named(board, criteria, &always, &mut sel)?;
    }
    if criteria["status"].as_array().is_some_and(|s| !s.is_empty()) {
        selecting = true;
        select_status(board, criteria, &always, &mut sel);
    }
    if !criteria["idleDays"].is_null() {
        selecting = true;
        select_idle(board, criteria, &always, &mut sel)?;
    }
    if truthy(criteria, "trash") {
        selecting = true;
        select_trash(board, criteria, &mut sel);
    }
    if !selecting {
        return Err(refuse(
            "name what to purge: threads, keep, status, idleDays or trash.",
        ));
    }
    sel.threads.retain(|id| !always.contains(id));
    closure(board, &mut sel);
    sel.log = selected_log(board, &sel);
    sel.inbox = sel
        .threads
        .iter()
        .filter(|id| {
            board["inbox"][id.as_str()]
                .as_array()
                .is_some_and(|l| !l.is_empty())
        })
        .cloned()
        .collect();
    let counts = counts_of(board, &sel);
    Ok(PurgePlan {
        ids: sel,
        counts,
        criteria: criteria.clone(),
    })
}
