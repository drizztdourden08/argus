//! Which problems are due for a wake: a flag seen on two samples in a row, again after the repeat time.

use crate::evaluate::Evaluation;
use crate::rule::Rule;
use crate::types::{Flag, ProcDoc};
use std::collections::{HashMap, HashSet};

/// One thing that needs a look.
#[derive(Debug, Clone, PartialEq)]
pub enum Due {
    Proc {
        doc: Box<ProcDoc>,
        kind: Flag,
    },
    Group {
        name: String,
        mem_mb: i64,
        limit: f64,
    },
}

/// When the sample was taken and how long a sent alert waits before it is due again.
#[derive(Debug, Clone, Copy)]
pub struct Pace {
    pub now: i64,
    pub repeat_ms: i64,
}

/// The flags seen so far and when each was last sent.
#[derive(Debug, Default)]
pub struct AlertBook {
    alerted: HashMap<String, Option<i64>>,
}

fn counts_for(doc: &ProcDoc, kind: Flag) -> bool {
    Flag::KINDS.contains(&kind) && (!doc.protected || matches!(kind, Flag::Hog | Flag::Hung))
}

impl AlertBook {
    fn mark(&mut self, key: String, pace: Pace, item: impl FnOnce() -> Due) -> Option<Due> {
        let Some(entry) = self.alerted.get_mut(&key) else {
            self.alerted.insert(key, None);
            return None;
        };
        let due = entry.is_none_or(|at| pace.now - at >= pace.repeat_ms);
        if due {
            *entry = Some(pace.now);
        }
        due.then(item)
    }

    fn proc_due(&mut self, docs: &[ProcDoc], pace: Pace, keys: &mut HashSet<String>) -> Vec<Due> {
        let mut out = Vec::new();
        for doc in docs {
            for kind in doc.flags.iter().copied().filter(|k| counts_for(doc, *k)) {
                let key = format!("{}:{}", doc.id, kind.as_str());
                keys.insert(key.clone());
                let item = || Due::Proc {
                    doc: Box::new(doc.clone()),
                    kind,
                };
                out.extend(self.mark(key, pace, item));
            }
        }
        out
    }

    fn group_due(
        &mut self,
        ev: &Evaluation,
        rules: &[Rule],
        pace: Pace,
        keys: &mut HashSet<String>,
    ) -> Vec<Due> {
        let mut out = Vec::new();
        for name in &ev.over {
            let key = format!("group:{name}");
            keys.insert(key.clone());
            let limit = rules
                .iter()
                .find(|r| &r.name == name)
                .and_then(|r| r.max_group_mem_mb);
            let item = || Due::Group {
                name: name.clone(),
                mem_mb: ev.groups.get(name).map_or(0, |g| g.mem_mb),
                limit: limit.unwrap_or(0.0),
            };
            out.extend(self.mark(key, pace, item));
        }
        out
    }

    /// The items due now. A protected process is only worth a wake for its memory or a hung window.
    pub fn due(&mut self, ev: &Evaluation, rules: &[Rule], pace: Pace) -> Vec<Due> {
        let mut keys = HashSet::new();
        let mut out = self.proc_due(&ev.docs, pace, &mut keys);
        out.extend(self.group_due(ev, rules, pace, &mut keys));
        self.alerted.retain(|key, _| keys.contains(key));
        out
    }
}
