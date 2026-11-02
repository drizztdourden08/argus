//! The stalls open right now: the diff against the last tick says which are new, which stay and which cleared,
//! and the acks the Overlord recorded in the kv are read here.

use indexmap::IndexMap;
use serde_json::{Value, json};

use super::key::{ack_key, stall_key};
use super::sources::StallKv;
use crate::stall_rules::Found;

/// The Overlord's acknowledgement of a stall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Ack {
    /// When (ISO).
    pub(super) at: String,
    /// Who.
    pub(super) by: String,
    /// What was done.
    pub(super) note: String,
}

impl Ack {
    fn to_json(&self) -> String {
        json!({"at": self.at, "by": self.by, "note": self.note}).to_string()
    }

    fn from_json(text: &str) -> Option<Self> {
        let value: Value = serde_json::from_str(text).ok()?;
        let field = |name: &str| value.get(name).and_then(Value::as_str).map(str::to_owned);
        Some(Self {
            at: field("at")?,
            by: field("by").unwrap_or_default(),
            note: field("note").unwrap_or_default(),
        })
    }
}

/// One open stall.
#[derive(Debug, Clone)]
pub(super) struct Open {
    /// The stall key.
    pub(super) key: String,
    /// What the rules found at the last tick.
    pub(super) found: Found,
    /// When Argus first saw it (epoch ms).
    pub(super) raised_at: i64,
    /// The Overlord's ack, once there is one.
    pub(super) ack: Option<Ack>,
}

/// The stalls open as of the last tick, in the order they were raised.
#[derive(Default)]
pub(super) struct StallBook {
    open: IndexMap<String, Open>,
}

impl StallBook {
    /// Take the stalls found at `now` as the open set. Returns the ones that were open and are gone.
    pub(super) fn update(&mut self, now: i64, found: Vec<Found>, kv: &dyn StallKv) -> Vec<Open> {
        let mut next: IndexMap<String, Open> = IndexMap::new();
        for stall in found {
            let key = stall_key(&stall);
            let (raised_at, known) = match self.open.shift_remove(&key) {
                Some(open) => (open.raised_at, open.ack),
                None => (now, None),
            };
            let ack = known.or_else(|| read_ack(kv, &key));
            next.insert(
                key.clone(),
                Open {
                    key,
                    found: stall,
                    raised_at,
                    ack,
                },
            );
        }
        std::mem::replace(&mut self.open, next)
            .into_values()
            .collect()
    }

    /// Every open stall.
    pub(super) fn all(&self) -> impl Iterator<Item = &Open> {
        self.open.values()
    }

    /// The open stalls nobody acknowledged.
    pub(super) fn unacked(&self) -> impl Iterator<Item = &Open> {
        self.open.values().filter(|open| open.ack.is_none())
    }

    /// Acknowledge every unacknowledged stall of a project. Returns the stalls acknowledged.
    pub(super) fn ack_project(&mut self, project: &str, ack: &Ack, kv: &dyn StallKv) -> Vec<Open> {
        let mut acked = Vec::new();
        for open in self.open.values_mut() {
            if open.found.project == project && open.ack.is_none() {
                kv.set(&ack_key(&open.key), &ack.to_json());
                open.ack = Some(ack.clone());
                acked.push(open.clone());
            }
        }
        acked
    }
}

fn read_ack(kv: &dyn StallKv, key: &str) -> Option<Ack> {
    Ack::from_json(&kv.get(&ack_key(key))?)
}
