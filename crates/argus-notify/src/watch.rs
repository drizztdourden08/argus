//! The automatic pings: the service reads each board export and tells the owner once.

use crate::auto::{Context, Draft, plan};
use crate::board::BoardFacts;
use crate::request::Outcome;
use crate::seen::{parse, render};
use crate::service::{Notify, locked};

impl Notify {
    /// A project board was exported again. A build handed to the owner and a new open question by
    /// a thread are pinged once each, as `ping` from `auto`. The markers are saved before a ping
    /// is sent, so a restart, a resync or a second export never tells again. Nothing happens while
    /// the service is stopped or for a retired project.
    pub fn on_board(&self, key: &str, facts: &BoardFacts) -> Vec<Outcome> {
        if !self.is_running() || facts.retired {
            return Vec::new();
        }
        self.advance(key, facts)
            .into_iter()
            .map(|draft| self.send(draft.into_request()))
            .collect()
    }

    fn advance(&self, key: &str, facts: &BoardFacts) -> Vec<Draft> {
        let config = self.ports.config.load();
        let name = self.name_of(key);
        let mut guard = locked(&self.seen);
        let seen = guard.get_or_insert_with(|| parse(self.ports.store.seen_load().as_deref()));
        let first = !seen.contains_key(key);
        let marker = seen.entry(key.to_owned()).or_default();
        let before = marker.clone();
        let context = Context {
            key,
            name: &name,
            first,
            auto: config.auto,
            now: self.ports.clock.now_ms(),
        };
        let drafts = plan(facts, marker, &context);
        let changed = first || *marker != before;
        if changed && self.ports.store.seen_save(&render(seen)).is_err() {
            self.warn("automatic ping failed");
            return Vec::new();
        }
        drafts
    }
}
