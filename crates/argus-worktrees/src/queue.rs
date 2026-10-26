//! One background worker that runs deletion jobs one at a time.

use crate::sync::{locked, waited};
use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

type Task = Box<dyn FnOnce() + Send + 'static>;

#[derive(Default)]
struct Lane {
    pending: VecDeque<Task>,
    running: bool,
}

#[derive(Default)]
struct Shared {
    lane: Mutex<Lane>,
    idle: Condvar,
}

impl Shared {
    fn drain(&self) {
        loop {
            let next = {
                let mut lane = locked(&self.lane);
                let next = lane.pending.pop_front();
                if next.is_none() {
                    lane.running = false;
                    self.idle.notify_all();
                }
                next
            };
            match next {
                Some(task) => {
                    let _ = catch_unwind(AssertUnwindSafe(task));
                }
                None => return,
            }
        }
    }
}

/// Tasks run in the order they were pushed, one at a time, on a worker thread that lives while there is work.
#[derive(Clone, Default)]
pub struct Queue(Arc<Shared>);

impl Queue {
    /// Queue a task; start the worker when none is running.
    pub fn push(&self, task: Task) {
        let start = {
            let mut lane = locked(&self.0.lane);
            lane.pending.push_back(task);
            !std::mem::replace(&mut lane.running, true)
        };
        if start {
            let shared = self.0.clone();
            thread::spawn(move || shared.drain());
        }
    }

    /// Block until every queued task has run.
    pub fn wait_idle(&self) {
        let mut lane = locked(&self.0.lane);
        while lane.running || !lane.pending.is_empty() {
            lane = waited(&self.0.idle, lane);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn a_task_that_panics_does_not_stop_the_ones_after_it() {
        let queue = Queue::default();
        let ran = Arc::new(AtomicUsize::new(0));
        queue.push(Box::new(|| panic!("a job went wrong")));
        let after = ran.clone();
        queue.push(Box::new(move || {
            after.fetch_add(1, Ordering::SeqCst);
        }));
        queue.wait_idle();
        assert_eq!(ran.load(Ordering::SeqCst), 1);
        let again = ran.clone();
        queue.push(Box::new(move || {
            again.fetch_add(1, Ordering::SeqCst);
        }));
        queue.wait_idle();
        assert_eq!(ran.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn tasks_run_in_order_and_wait_idle_sees_them_all() {
        let queue = Queue::default();
        let log = Arc::new(Mutex::new(Vec::new()));
        let ran = Arc::new(AtomicUsize::new(0));
        for i in 0..5 {
            let (log, ran) = (log.clone(), ran.clone());
            queue.push(Box::new(move || {
                thread::sleep(std::time::Duration::from_millis(5));
                log.lock().unwrap().push(i);
                ran.fetch_add(1, Ordering::SeqCst);
            }));
        }
        queue.wait_idle();
        assert_eq!(ran.load(Ordering::SeqCst), 5);
        assert_eq!(*log.lock().unwrap(), [0, 1, 2, 3, 4]);
        queue.wait_idle();
    }
}
