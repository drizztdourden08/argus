//! Scans and removals take turns, first come first served.
//!
//! A plain mutex is not fair: the scanner releases it between two projects and takes it again at once, so a removal
//! that waits can lose to every scan of a round, round after round (a request stayed `running` for an hour that way).
//! A ticket lock hands the turn to whoever asked first, so a removal waits for the scan that runs and no more.

use crate::sync::{locked, waited};
use std::sync::{Condvar, Mutex};

#[derive(Default)]
struct Line {
    next: u64,
    serving: u64,
}

/// The one-at-a-time door of scans and removals, in the order they asked.
#[derive(Default)]
pub struct Turns {
    line: Mutex<Line>,
    moved: Condvar,
}

/// A turn: held while a scan or a removal runs, given to the next one when dropped (a panic included).
pub struct Turn<'a>(&'a Turns);

impl Turns {
    /// Wait for every turn asked before this one, then take it.
    pub fn take(&self) -> Turn<'_> {
        let mut line = locked(&self.line);
        let mine = line.next;
        line.next += 1;
        while line.serving != mine {
            line = waited(&self.moved, line);
        }
        Turn(self)
    }
}

impl Drop for Turn<'_> {
    fn drop(&mut self) {
        locked(&self.0.line).serving += 1;
        self.0.moved.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn a_waiting_turn_comes_before_one_asked_after_it_however_fast_that_one_comes_back() {
        let turns = Arc::new(Turns::default());
        let done = Arc::new(AtomicBool::new(false));
        let first = turns.take();
        let (t, d) = (turns.clone(), done.clone());
        let waiter = thread::spawn(move || {
            let _turn = t.take();
            d.store(true, Ordering::SeqCst);
        });
        thread::sleep(Duration::from_millis(50));
        drop(first);
        let again = turns.take();
        assert!(
            done.load(Ordering::SeqCst),
            "the waiter had its turn before the one that asked again"
        );
        drop(again);
        waiter.join().unwrap();
    }
}
