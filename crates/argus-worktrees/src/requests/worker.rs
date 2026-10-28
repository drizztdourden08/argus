//! The one worker: oldest request first, one at a time, resumed after a restart.

use super::record::RequestStatus;
use crate::service::Service;
use crate::sync::locked;
use std::panic::{AssertUnwindSafe, catch_unwind};

impl Service {
    pub(crate) fn requests_halted(&self) -> bool {
        locked(&self.requests.state).halted
    }

    /// The projects that may hold requests: the ones with a board, and the ones already loaded.
    fn request_projects(&self) -> Vec<String> {
        let mut projects = self.deps.store.projects();
        let loaded: Vec<String> = locked(&self.requests.state)
            .stores
            .keys()
            .cloned()
            .collect();
        for project in loaded {
            if !projects.contains(&project) {
                projects.push(project);
            }
        }
        projects
    }

    /// The oldest request still to run or running: its project and id.
    fn next_request(&self) -> Option<(String, String)> {
        let mut best: Option<(String, String, String)> = None;
        for project in self.request_projects() {
            let waiting = self.with_requests(&project, |store| {
                store
                    .list
                    .iter()
                    .filter(|r| r.status.active())
                    .map(|r| (r.at.clone(), r.id.clone()))
                    .collect::<Vec<_>>()
            });
            for (at, id) in waiting {
                if best.as_ref().is_none_or(|(b, _, _)| at < *b) {
                    best = Some((at, project.clone(), id));
                }
            }
        }
        best.map(|(_, project, id)| (project, id))
    }

    fn drain_requests(&self) {
        let ran = catch_unwind(AssertUnwindSafe(|| {
            while !self.requests_halted() {
                let Some((project, id)) = self.next_request() else {
                    return;
                };
                self.process_request(&project, &id);
            }
        }));
        locked(&self.requests.state).looping = false;
        if ran.is_ok() && !self.requests_halted() && self.next_request().is_some() {
            self.kick_requests();
        }
    }

    /// Start the worker unless it runs, or the service is stopped.
    pub(crate) fn kick_requests(&self) {
        {
            let mut state = locked(&self.requests.state);
            if state.halted || state.looping {
                return;
            }
            state.looping = true;
        }
        let service = self.clone();
        self.requests
            .queue
            .push(Box::new(move || service.drain_requests()));
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
        let mut any = false;
        let mut untold: Vec<(String, String)> = Vec::new();
        for project in self.deps.store.projects() {
            self.with_requests(&project, |store| {
                for rec in &mut store.list {
                    if rec.status == RequestStatus::Running {
                        rec.status = RequestStatus::Queued;
                    }
                    if rec.status.active() {
                        any = true;
                    } else if !rec.notified && rec.finished_at.is_some() {
                        untold.push((project.clone(), rec.id.clone()));
                    }
                }
            });
        }
        if !self.opts.run_requests {
            return;
        }
        if any {
            self.kick_requests();
        }
        for (project, id) in untold {
            let service = self.clone();
            self.requests
                .queue
                .push(Box::new(move || service.deliver_request(&project, &id)));
        }
    }

    /// The worker stops after the target it is on; a request it leaves stays running until the next start.
    pub(crate) fn stop_requests(&self) {
        locked(&self.requests.state).halted = true;
    }
}
