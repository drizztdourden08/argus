//! A board document is a JSON object with fields the crate does not own, so records stay JSON
//! objects. These are the accessors, the edit lists and the stamp every service uses.

use crate::iso::{format_iso, parse_iso};
use serde_json::{Map, Value};

/// One document of the `agents` collection (or a thread), as the board file keeps it.
pub type Record = Map<String, Value>;

/// The statuses that count as running.
pub const LIVE_STATUSES: [&str; 3] = ["working", "waiting", "stop-requested"];

/// The text of a string field.
#[must_use]
pub fn text<'a>(record: &'a Record, key: &str) -> Option<&'a str> {
    record.get(key)?.as_str()
}

/// The text of a string field that is not empty.
#[must_use]
pub fn filled<'a>(record: &'a Record, key: &str) -> Option<&'a str> {
    text(record, key).filter(|s| !s.is_empty())
}

/// Whether a value is truthy the way the Node code reads it.
#[must_use]
pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// Whether a field is present and truthy.
#[must_use]
pub fn is_set(record: &Record, key: &str) -> bool {
    record.get(key).is_some_and(truthy)
}

/// The moment a timestamp field holds, in epoch milliseconds.
#[must_use]
pub fn ms_of(record: &Record, key: &str) -> Option<i64> {
    text(record, key).and_then(parse_iso)
}

/// The id of a document, empty when it has none.
#[must_use]
pub fn id_of(record: &Record) -> &str {
    text(record, "id").unwrap_or("")
}

/// The id Claude gave the sub-agent: `claudeId`, else its older name `hookId`.
#[must_use]
pub fn claude_id_of(record: &Record) -> Option<String> {
    filled(record, "claudeId")
        .or_else(|| filled(record, "hookId"))
        .map(str::to_owned)
}

/// Whether the agent counts as running: a live status that is not archived or deleted.
#[must_use]
pub fn is_live(record: &Record) -> bool {
    text(record, "status").is_some_and(|s| LIVE_STATUSES.contains(&s))
        && !is_set(record, "archivedAt")
        && !is_set(record, "deletedAt")
}

/// Records one change the way the board does: the revision, who and when.
pub fn stamp(record: &mut Record, actor: &str, now_ms: i64) {
    let rev = record.get("rev").and_then(Value::as_i64).unwrap_or(0) + 1;
    record.insert("updatedAt".into(), Value::String(format_iso(now_ms)));
    record.insert("updatedBy".into(), Value::String(actor.into()));
    record.insert("rev".into(), Value::from(rev));
}

/// A list of field changes for one record; `None` removes the field.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Patch(Vec<(String, Option<Value>)>);

impl Patch {
    /// An empty list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a field; a later change to the same field replaces this one.
    #[must_use]
    pub fn with(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.put(key, Some(value.into()));
        self
    }

    /// Removes a field.
    #[must_use]
    pub fn without(mut self, key: &str) -> Self {
        self.put(key, None);
        self
    }

    /// Removes every one of the fields.
    #[must_use]
    pub fn without_all(mut self, keys: &[&str]) -> Self {
        for key in keys {
            self.put(key, None);
        }
        self
    }

    fn put(&mut self, key: &str, value: Option<Value>) {
        match self.0.iter_mut().find(|(k, _)| k == key) {
            Some(entry) => entry.1 = value,
            None => self.0.push((key.to_owned(), value)),
        }
    }

    /// Lets the changes of `other` win over this list.
    pub fn merge(&mut self, other: Patch) {
        for (key, value) in other.0 {
            self.put(&key, value);
        }
    }

    /// The changes in the order they were made.
    #[must_use]
    pub fn entries(&self) -> &[(String, Option<Value>)] {
        &self.0
    }

    /// Writes the changes onto a record.
    pub fn apply_to(&self, record: &mut Record) {
        for (key, value) in &self.0 {
            match value {
                Some(v) => {
                    record.insert(key.clone(), v.clone());
                }
                None => {
                    record.shift_remove(key);
                }
            }
        }
    }
}

/// The changes decided so far for the records of one board, by record id.
#[derive(Debug, Default)]
pub struct Patches(Vec<(String, Patch)>);

impl Patches {
    /// Adds changes for a record on top of what is there.
    pub fn add(&mut self, id: &str, patch: Patch) {
        match self.0.iter_mut().find(|(i, _)| i == id) {
            Some(entry) => entry.1.merge(patch),
            None => self.0.push((id.to_owned(), patch)),
        }
    }

    /// The record as it will be once the changes are written.
    #[must_use]
    pub fn view(&self, record: &Record) -> Record {
        let mut copy = record.clone();
        if let Some((_, patch)) = self.0.iter().find(|(i, _)| i == id_of(record)) {
            patch.apply_to(&mut copy);
        }
        copy
    }

    /// Whether nothing is to be written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The records to change and their changes.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Patch)> {
        self.0.iter().map(|(id, patch)| (id.as_str(), patch))
    }
}
