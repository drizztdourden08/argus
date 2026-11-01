//! The reaper's wakes: one per thread per batch, and no sooner than a window after the last one
//! to that thread. What a thread may not hear yet waits for the next batch past the window.

use std::collections::{BTreeMap, HashMap};

type Key = (String, String);

/// The lines waiting for each (project, thread) and when each was last woken.
#[derive(Debug, Default)]
pub(crate) struct WakeDebounce {
    last: HashMap<Key, i64>,
    pending: BTreeMap<Key, Vec<String>>,
}

impl WakeDebounce {
    /// Keeps a line for the thread's next wake.
    pub(crate) fn push(&mut self, project: &str, thread: &str, line: String) {
        let key = (project.to_string(), thread.to_string());
        self.pending.entry(key).or_default().push(line);
    }

    /// The wakes due now, as (project, thread, lines), each thread's lines in the order kept.
    pub(crate) fn due(&mut self, now: i64, window_ms: i64) -> Vec<(String, String, Vec<String>)> {
        let ready: Vec<Key> = self
            .pending
            .keys()
            .filter(|key| self.last.get(*key).is_none_or(|at| now - at >= window_ms))
            .cloned()
            .collect();
        ready
            .into_iter()
            .filter_map(|key| {
                let lines = self.pending.remove(&key)?;
                self.last.insert(key.clone(), now);
                Some((key.0, key.1, lines))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: i64 = 60_000;

    fn wake(project: &str, thread: &str, lines: &[&str]) -> (String, String, Vec<String>) {
        let lines = lines.iter().map(ToString::to_string).collect();
        (project.to_string(), thread.to_string(), lines)
    }

    #[test]
    fn one_wake_per_thread_per_batch_then_none_inside_the_window() {
        let mut d = WakeDebounce::default();
        d.push("alpha", "lead", "a".to_string());
        d.push("alpha", "lead", "b".to_string());
        d.push("beta", "boss", "c".to_string());
        let first = d.due(0, 10 * MIN);
        assert_eq!(
            first,
            vec![
                wake("alpha", "lead", &["a", "b"]),
                wake("beta", "boss", &["c"])
            ]
        );
        d.push("alpha", "lead", "d".to_string());
        assert_eq!(d.due(5 * MIN, 10 * MIN), Vec::new());
        assert_eq!(
            d.due(10 * MIN, 10 * MIN),
            vec![wake("alpha", "lead", &["d"])]
        );
        assert_eq!(d.due(30 * MIN, 10 * MIN), Vec::new());
    }
}
