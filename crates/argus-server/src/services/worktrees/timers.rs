//! When the scanner looks: one full scan after the start; then, every `liveMs`, a no-git live check,
//! the cheap file-time look at every worktree (a scan reads with git only what changed, and everything
//! once every `scanMs`, 30 minutes by default) and a worktree hygiene step; and a pump that scans what
//! asked for it (changed rules after a debounce, a removal's rescan, leftovers a scan had no time to
//! measure) and republishes the rows while a deletion job moves. A scan the pump runs is always full.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use argus_worktrees::Service as WorktreeService;
use serde_json::json;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Instant;

use super::config::Timing;
use super::publish::{publish, publish_derived, publish_scratch};
use crate::Services;

const PUMP_EVERY: Duration = Duration::from_millis(200);
const TASK: &str = "worktrees";

/// The projects that have a board.
fn projects(service: &WorktreeService) -> Vec<String> {
    service.deps.store.projects()
}

/// Scans one project and writes its docs. Blocks: git and the disk.
fn scan(svc: &Services, service: &WorktreeService, key: &str) {
    if let Some(derived) = service.scan_now(key) {
        publish_derived(svc, key, &derived);
    }
}

/// Runs `work` on a thread that may block. A panic in a scan is the scanner's: one log line, and
/// the next tick looks again.
async fn blocking(svc: &Arc<Services>, work: impl FnOnce(&Services) + Send + 'static) {
    let held = Arc::clone(svc);
    if let Err(err) = tokio::task::spawn_blocking(move || work(&held)).await {
        svc.log.error(
            "worktree scan failed",
            json!({ "err": format!("the scanner stopped on a panic: {err}") }),
        );
    }
}

async fn scan_all(svc: &Arc<Services>, service: &Arc<WorktreeService>) {
    let service = Arc::clone(service);
    blocking(svc, move |svc| {
        for key in projects(&service) {
            scan(svc, &service, &key);
        }
    })
    .await;
}

async fn live_all(svc: &Arc<Services>, service: &Arc<WorktreeService>) {
    let service = Arc::clone(service);
    blocking(svc, move |svc| {
        if service.scratch_step() {
            publish_scratch(svc, &service);
        }
        for key in projects(&service) {
            let found = service.live_step(&key);
            let scanned = service.scan_if_changed(&key).is_some();
            let hygiene = service.hygiene_step(&key);
            if scanned || found.dropped > 0 || !found.added.is_empty() || hygiene.changed {
                publish(svc, &service, &key);
            }
        }
    })
    .await;
}

/// The first scan reads everything. Later scans follow the live tick's look at the disk.
async fn first_scan(svc: Arc<Services>, service: Arc<WorktreeService>, start_delay: Duration) {
    tokio::time::sleep(start_delay).await;
    scan_all(&svc, &service).await;
}

async fn live_loop(svc: Arc<Services>, service: Arc<WorktreeService>) {
    let every = Duration::from_millis(u64::try_from(service.opts.live_ms).unwrap_or(20_000));
    loop {
        tokio::time::sleep(every).await;
        live_all(&svc, &service).await;
    }
}

/// What the pump remembers between ticks.
struct Pump {
    timing: Timing,
    asked: UnboundedReceiver<String>,
    debounce: HashMap<String, Instant>,
    scanned: HashMap<String, Instant>,
    jobs: HashMap<String, String>,
}

impl Pump {
    fn due(&mut self, service: &WorktreeService) -> Vec<String> {
        let now = Instant::now();
        let wait = Duration::from_millis(self.timing.debounce);
        while let Ok(key) = self.asked.try_recv() {
            self.debounce.entry(key).or_insert(now + wait);
        }
        let mut due: Vec<String> = self
            .debounce
            .iter()
            .filter(|(_, at)| **at <= now)
            .map(|(key, _)| key.clone())
            .collect();
        for key in &due {
            self.debounce.remove(key);
        }
        due.extend(service.take_rescans());
        due.extend(self.unmeasured(service, now));
        due.sort();
        due.dedup();
        due
    }

    /// Projects with leftovers the last scan had no time to measure, once `more_ms` has passed.
    fn unmeasured(&self, service: &WorktreeService, now: Instant) -> Vec<String> {
        let more = Duration::from_millis(self.timing.more);
        projects(service)
            .into_iter()
            .filter(|key| service.pending_count(key) > 0)
            .filter(|key| self.scanned.get(key).is_none_or(|at| now - *at >= more))
            .collect()
    }

    /// The projects whose deletion jobs or removal requests changed since the last tick.
    fn moved(&mut self, service: &WorktreeService) -> Vec<String> {
        let mut moved = Vec::new();
        for key in projects(service) {
            let now = json!([service.jobs(&key), service.requests_recent(&key, 20)]).to_string();
            if self.jobs.get(&key).is_none_or(|before| *before != now) {
                self.jobs.insert(key.clone(), now);
                moved.push(key);
            }
        }
        moved
    }

    async fn tick(&mut self, svc: &Arc<Services>, service: &Arc<WorktreeService>) {
        for key in self.due(service) {
            self.scanned.insert(key.clone(), Instant::now());
            let service = Arc::clone(service);
            blocking(svc, move |svc| scan(svc, &service, &key)).await;
        }
        let moved = self.moved(service);
        if !moved.is_empty() {
            let service = Arc::clone(service);
            blocking(svc, move |svc| {
                for key in &moved {
                    publish(svc, &service, key);
                }
            })
            .await;
        }
    }
}

async fn pump_loop(svc: Arc<Services>, service: Arc<WorktreeService>, mut pump: Pump) {
    let mut ticker = tokio::time::interval(PUMP_EVERY);
    loop {
        ticker.tick().await;
        pump.tick(&svc, &service).await;
    }
}

/// Starts the timers; ended by [`stop`].
pub(super) fn start(
    svc: &Arc<Services>,
    service: &Arc<WorktreeService>,
    timing: Timing,
    asked: UnboundedReceiver<String>,
) {
    let delay = Duration::from_millis(timing.start_delay);
    let pump = Pump {
        timing,
        asked,
        debounce: HashMap::new(),
        scanned: HashMap::new(),
        jobs: HashMap::new(),
    };
    svc.tasks.keep(
        TASK,
        tokio::spawn(first_scan(Arc::clone(svc), Arc::clone(service), delay)),
    );
    svc.tasks.keep(
        TASK,
        tokio::spawn(live_loop(Arc::clone(svc), Arc::clone(service))),
    );
    svc.tasks.keep(
        TASK,
        tokio::spawn(pump_loop(Arc::clone(svc), Arc::clone(service), pump)),
    );
}

/// Ends the timers.
pub(super) fn stop(svc: &Services) {
    svc.tasks.abort(TASK);
}
