//! How an op went, and the revision check that decides between applying and a conflict.

use serde_json::{Map, Value, json};

use crate::json::{display, nullish, same, strict_eq, truthy};

/// The outcome class of an op.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Done.
    Applied,
    /// Nothing applied: the record changed since the writer looked.
    Conflict,
    /// Nothing applied: the op was refused.
    Refused,
}

impl Status {
    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Applied => "applied",
            Status::Conflict => "conflict",
            Status::Refused => "refused",
        }
    }
}

/// A line already queued for a thread's inbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// The thread told.
    pub to: String,
    /// What it was told.
    pub text: String,
}

/// What `apply_op` returns.
#[derive(Debug, Clone)]
pub struct OpResult {
    /// How it went.
    pub status: Status,
    /// A sentence saying what happened or why not (`gone`, `in trash`, a refusal).
    pub outcome: String,
    /// On a conflict, the fields that differ, with their current values.
    pub current: Option<Value>,
    /// On a conflict, who changed the record last.
    pub by: Option<String>,
    /// The inbox lines already queued for the owning threads (do not queue them again).
    pub notify: Vec<Notice>,
    /// A bulk's result per sub-op.
    pub results: Option<Vec<Value>>,
    /// Kind specific extras: `created` for a create, `file` and `counts` for a purge.
    pub extra: Map<String, Value>,
}

impl OpResult {
    fn new(status: Status, outcome: String) -> Self {
        OpResult {
            status,
            outcome,
            current: None,
            by: None,
            notify: Vec::new(),
            results: None,
            extra: Map::new(),
        }
    }

    /// An applied op.
    pub(crate) fn applied(outcome: impl Into<String>) -> Self {
        Self::new(Status::Applied, outcome.into())
    }

    /// A refused op.
    pub(crate) fn refused(outcome: impl Into<String>) -> Self {
        Self::new(Status::Refused, outcome.into())
    }

    /// A conflict on a doc, with the fields that differ.
    pub(crate) fn conflict(doc: &Value, current: Map<String, Value>) -> Self {
        let by = nullish(doc, "updatedBy");
        let who = by.clone().unwrap_or_else(|| "someone".to_owned());
        let mut result = Self::new(
            Status::Conflict,
            format!(
                "{} changed since you looked (last by {who}).",
                display(&doc["id"])
            ),
        );
        result.current = Some(Value::Object(current));
        result.by = by;
        result
    }

    /// The same result with these notices.
    pub(crate) fn with_notify(mut self, notify: Vec<Notice>) -> Self {
        self.notify = notify;
        self
    }

    /// The same result with one more extra.
    pub(crate) fn with_extra(mut self, key: &str, value: Value) -> Self {
        self.extra.insert(key.to_owned(), value);
        self
    }

    /// This result as a bulk lists it.
    pub(crate) fn summary(&self) -> Value {
        let mut out = Map::new();
        out.insert("status".into(), json!(self.status.as_str()));
        out.insert("outcome".into(), json!(self.outcome));
        if let Some(current) = &self.current {
            out.insert("current".into(), current.clone());
        }
        if let Some(by) = self.by.as_ref().filter(|b| !b.is_empty()) {
            out.insert("by".into(), json!(by));
        }
        if let Some(created) = self.extra.get("created") {
            out.insert("created".into(), created.clone());
        }
        Value::Object(out)
    }
}

fn all_fields(doc: &Value, fields: &[String]) -> Map<String, Value> {
    let all: Map<String, Value> = fields
        .iter()
        .map(|f| (f.clone(), doc[f.as_str()].clone()))
        .collect();
    if all.is_empty() {
        Map::from_iter([("rev".to_owned(), doc["rev"].clone())])
    } else {
        all
    }
}

/// Whether the op asks to skip the revision check: a `force` flag, not a `force` reason.
fn skips_check(op: &Value) -> bool {
    op["force"].as_str().is_none() && truthy(op, "force")
}

/// The revision check. The writer saw `op.baseRev` (with `op.prev` holding the values it looked
/// at). Same revision, no revision given (last writer wins) or `force: true`: go. A `force` that is
/// a reason text overrides a rule only, never this check. A newer revision still
/// passes field by field: a field is fine while it holds what the writer saw or already holds
/// the new value. Anything else is the writer's edit colliding with someone else's: those fields
/// come back as `{ field: current value }`.
pub(crate) fn version_check(
    doc: &Value,
    op: &Value,
    fields: &[String],
    next: &Map<String, Value>,
) -> Option<Map<String, Value>> {
    let base = &op["baseRev"];
    if skips_check(op) || base.is_null() || strict_eq(base, &doc["rev"]) {
        return None;
    }
    let prev = &op["prev"];
    if prev.is_null() {
        return Some(all_fields(doc, fields));
    }
    let current: Map<String, Value> = fields
        .iter()
        .filter(|f| !same(doc.get(f.as_str()), next.get(f.as_str())))
        .filter(|f| {
            !(prev.get(f.as_str()).is_some() && same(doc.get(f.as_str()), prev.get(f.as_str())))
        })
        .map(|f| (f.clone(), doc[f.as_str()].clone()))
        .collect();
    (!current.is_empty()).then_some(current)
}
