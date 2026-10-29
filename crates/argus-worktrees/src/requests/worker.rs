//! The workers: one per project, oldest request first, one at a time, resumed after a restart.

use super::record::RequestStatus;
use crate::queue::Queue;
use crate::service::Service;
use crate::sync::locked;
use std::panic::{AssertUnwindSafe, catch_unwind};

impl Service {
    pub(crate) fn requests_halted(&self) -> bool {
        locked(&self.requests.state).halted
    }

    /// The worker of a project, made on first use.
    fn worker_of(&self, project: &str) -> Queue {
        locked(&self.requests.queues)
            .entry(project.to_string())
            .or_default()
            .clone()
    }

    /// Every worker made so far.
    pub(crate) fn workers(&self) -> Vec<Queue> {
        locked(&self.requests.queues).values().cloned().collect()
    }

    /// The project's oldest request still to run or running.
    fn next_request(&self, project: &str) -> Option<String> {
        self.with_requests(project, |store| {
            store
                .list
                .iter()
                .filter(|r| r.status.active())
                .min_by(|a, b| a.at.cmp(&b.at))
                .map(|r| r.id.clone())
        })
    }

    fn drain_requests(&self, project: &str) {
        let ran = catch_unwind(AssertUnwindSafe(|| {
            while !self.requests_halted() {
                let Some(id) = self.next_request(project) else {
                    return;
                };
                self.process_request(project, &id);
            }
        }));
        locked(&self.requests.state).looping.remove(project);
        if ran.is_err() {
            self.deps.notifier.problem(&format!(
                "{project}: the worktree request worker stopped on a panic; it starts again on the next request"
            ));
            return;
        }
        if !self.requests_halted() && self.next_request(project).is_some() {
            self.kick_requests(project);
        }
    }

    /// Start the project's worker unless it runs, or the service is stopped.
    pub(crate) fn kick_requests(&self, project: &str) {
        {
            let mut state = locked(&self.requests.state);
            if state.halted || !state.looping.insert(project.to_string()) {
                return;
            }
        }
        let (service, key) = (self.clone(), project.to_string());
        self.worker_of(project)
            .push(Box::new(move || service.drain_requests(&key)));
    }

    /// A restart: what was queued or running is queued again; the finished ones that were never told are told.
    pub(crate) fn start_requests(&self) {
        {
            let mut state = locked(&self.requests.state);
            if !state.halted {
                return;
            }
            state.halted = false;
        }
        for project in self.deps.store.projects() {
            self.resume_requests(&project);
        }
    }

    /// One project's part of a restart.
    fn resume_requests(&self, project: &str) {
        let mut any = false;
        let mut untold: Vec<String> = Vec::new();
        self.with_requests(project, |store| {
            for rec in &mut store.list {
                if rec.status == RequestStatus::Running {
                    rec.status = RequestStatus::Queued;
                }
                if rec.status.active() {
                    any = true;
                } else if !rec.notified && rec.finished_at.is_some() {
                    untold.push(rec.id.clone());
                }
            }
        });
        if !self.opts.run_requests {
            return;
        }
        if any {
            self.kick_requests(project);
        }
        for id in untold {
            let (service, key) = (self.clone(), project.to_string());
            self.worker_of(project)
                .push(Box::new(move || service.deliver_request(&key, &id)));
        }
    }

    /// The worker stops after the target it is on; a request it leaves stays running until the next start.
    pub(crate) fn stop_requests(&self) {
        locked(&self.requests.state).halted = true;
    }
}
