//! The wake relay: a paused thread that holds a queued wake is woken through another live thread, then
//! by a headless resume, and the owner hears of it when nothing worked. Pure logic over what the registry
//! sees; the server turns the actions into inbox notes, a child process and a toast. See `docs/wake-relay.md`.

mod pick;
mod track;
mod types;

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap};

pub use types::{
    Action, AttemptKind, LastAttempt, RelayLive, RelayPort, RelaySettings, RelayTarget, ThreadKey,
    Ward,
};

use track::{Attempt, Step};

/// A wake waits this long before the first relay note.
pub const FIRST_AFTER_MS: i64 = 30_000;
/// A target still due this long after a note gets the next step.
pub const AGAIN_AFTER_MS: i64 = 120_000;
/// A target still due this long after the headless try reaches the owner.
pub const NOTICE_AFTER_MS: i64 = 600_000;
/// After the owner notice, nothing more for a target for this long.
pub const QUIET_MS: i64 = 3_600_000;
/// Relay notes sent for one target before the headless try.
pub const MAX_NOTES: u32 = 2;

/// The attempts made for every target that is due.
#[derive(Debug, Default)]
pub struct Relay {
    settings: RelaySettings,
    attempts: HashMap<ThreadKey, Attempt>,
}

impl Relay {
    /// A relay with its settings and no attempts.
    #[must_use]
    pub fn new(settings: RelaySettings) -> Self {
        Self {
            settings,
            attempts: HashMap::new(),
        }
    }

    /// The last attempt made for a target, if it is still tracked.
    #[must_use]
    pub fn last(&self, project: &str, thread: &str) -> Option<LastAttempt> {
        let key = ThreadKey::new(project, thread);
        self.attempts.get(&key)?.last.clone()
    }

    /// What to do now for the targets that are due, given the threads that can relay. A target that is no
    /// longer due is forgotten. The notes that go to one relay are folded into one action.
    pub fn step(&mut self, now: i64, targets: &[RelayTarget], live: &[RelayLive]) -> Vec<Action> {
        self.attempts
            .retain(|key, _| targets.iter().any(|t| &t.ward.key == key));
        if !self.settings.enabled {
            return Vec::new();
        }
        let mut notes: BTreeMap<ThreadKey, Vec<Ward>> = BTreeMap::new();
        let mut actions = Vec::new();
        for target in targets {
            match self.advance(now, target, live) {
                Some(Step::Note(via)) => notes.entry(via).or_default().push(target.ward.clone()),
                Some(Step::Do(action)) => actions.push(action),
                None => {}
            }
        }
        actions.extend(
            notes
                .into_iter()
                .map(|(via, targets)| Action::Note { via, targets }),
        );
        actions
    }

    fn advance(&mut self, now: i64, target: &RelayTarget, live: &[RelayLive]) -> Option<Step> {
        let headless = self.settings.headless;
        let attempt = self.attempts.entry(target.ward.key.clone()).or_default();
        attempt.step(now, target, live, headless)
    }
}
