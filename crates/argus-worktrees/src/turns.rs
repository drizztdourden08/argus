//! Scans and removals of one project take turns, first come first served. Another project never waits for them.
//!
//! A plain mutex is not fair: the scanner releases it between two projects and takes it again at once, so a removal
//! that waits can lose to every scan of a round, round after round (a request stayed `running` for an hour that way).
//! A ticket lock hands the turn to whoever asked first, so a removal waits for the scan that runs and no more.
//!
//! The door is the project's own. One door for every project made a removal of an empty folder in Argus wait behind
//! the `node_modules` clear of a worktree in Brock, then one in Tessera, then one in Archipelia (the live bug of
//! 2026-11-01: a request stayed on "waiting for its turn" for minutes while nothing of its own project ran).
//! The turn also says who holds it and since when, so a long wait names the holder.

use crate::sync::{locked, waited};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

/// Who has the turn: what runs, the removal's worktree key when it is one, since when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holder {
    pub what: String,
    pub key: Option<String>,
    pub since_ms: i64,
}

#[derive(Default)]
struct Line {
    next: u64,
    serving: u64,
    holder: Option<Holder>,
}

/// One project's one-at-a-time door, in the order they asked.
#[derive(Default)]
struct Door {
    line: Mutex<Line>,
    moved: Condvar,
}

/// The doors of every project, made on first use.
#[derive(Default)]
pub struct Turns {
    doors: Mutex<HashMap<String, Arc<Door>>>,
}

/// What asks for a turn: a sentence for the ones that wait, and the removal's worktree key.
pub struct Asker<'a> {
    pub what: &'a str,
    pub key: Option<&'a str>,
}

/// A turn: held while a scan or a removal runs, given to the next one of the project when dropped (a panic included).
pub struct Turn(Arc<Door>);

impl Turns {
    fn door(&self, project: &str) -> Arc<Door> {
        locked(&self.doors)
            .entry(project.to_string())
            .or_default()
            .clone()
    }

    /// Wait for every turn of the project asked before this one, then take it. `now` reads the time it was given.
    pub fn take(&self, project: &str, asker: &Asker<'_>, now: &dyn Fn() -> i64) -> Turn {
        let door = self.door(project);
        {
            let mut line = locked(&door.line);
            let mine = line.next;
            line.next += 1;
            while line.serving != mine {
                line = waited(&door.moved, line);
            }
            line.holder = Some(Holder {
                what: asker.what.to_string(),
                key: asker.key.map(ToString::to_string),
                since_ms: now(),
            });
        }
        Turn(door)
    }

    /// Who holds the project's turn now, if anyone.
    pub fn holder(&self, project: &str) -> Option<Holder> {
        let door = locked(&self.doors).get(project).cloned()?;
        locked(&door.line).holder.clone()
    }
}

impl Drop for Turn {
    fn drop(&mut self) {
        {
            let mut line = locked(&self.0.line);
            line.holder = None;
            line.serving += 1;
        }
        self.0.moved.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::Duration;

    const TEST: Asker<'static> = Asker {
        what: "a test",
        key: None,
    };

    fn at_zero() -> i64 {
        0
    }

    #[test]
    fn a_waiting_turn_comes_before_one_asked_after_it_however_fast_that_one_comes_back() {
        let turns = Arc::new(Turns::default());
        let done = Arc::new(AtomicBool::new(false));
        let first = turns.take("p", &TEST, &at_zero);
        let (t, d) = (turns.clone(), done.clone());
        let waiter = thread::spawn(move || {
            let _turn = t.take("p", &TEST, &at_zero);
            d.store(true, Ordering::SeqCst);
        });
        thread::sleep(Duration::from_millis(50));
        drop(first);
        let again = turns.take("p", &TEST, &at_zero);
        assert!(
            done.load(Ordering::SeqCst),
            "the waiter had its turn before the one that asked again"
        );
        drop(again);
        waiter.join().unwrap();
    }

    #[test]
    fn another_projects_turn_is_given_at_once_and_the_holder_is_named_until_it_lets_go() {
        let turns = Turns::default();
        let asker = Asker {
            what: "the removal of w1",
            key: Some("w1"),
        };
        let held = turns.take("brock", &asker, &|| 42);
        let other = turns.take("argus", &TEST, &at_zero);
        drop(other);
        assert_eq!(
            turns.holder("brock"),
            Some(Holder {
                what: "the removal of w1".to_string(),
                key: Some("w1".to_string()),
                since_ms: 42,
            })
        );
        assert_eq!(turns.holder("argus"), None);
        drop(held);
        assert_eq!(turns.holder("brock"), None);
    }
}
