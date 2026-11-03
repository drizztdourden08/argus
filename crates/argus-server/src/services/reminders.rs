//! What the wake service cannot see by itself: the owner's notifications for a fault open an hour,
//! and the worktrees that wait for a decision. Both are plugged into the wake service once every
//! service is built, because the wake service is built before them.

use std::sync::Arc;
use std::thread;

use argus_notify::{AutoConfig, By, Level, Notify, PingRequest};
use argus_store::Db;
use argus_wake::reminders::{
    NoticeSource, OpenNotice, OwnerNotice, OwnerSink, TOPIC_ASSIGNED, TOPIC_RUN, TOPIC_VERIFY,
};
use argus_wake::service::driver::lock;
use argus_worktrees::{REASON, Service as WorktreeService};

use crate::Services;

/// Tells the owner of a fault, a stall or a change to an item once, for good: it is remembered by its topic,
/// its project, its thread and when it opened (or the event of the item), so a restart does not toast it
/// again and the kinds never share a key. A notice goes through `Notify::send`, so the limits of the
/// toasts hold: a thread gets three toasts a minute, and the rest are recorded with one summary.
struct OwnerPings {
    notify: Arc<Notify>,
    db: Arc<Db>,
    shadow: bool,
}

fn told_key(notice: &OwnerNotice) -> String {
    format!(
        "{}-owner:{}:{}:{}",
        notice.topic, notice.project, notice.thread, notice.since
    )
}

/// Is the automatic ping of this topic switched on (`notify.auto`)? The faults and the stalls always are.
fn switched_on(auto: AutoConfig, topic: &str) -> bool {
    match topic {
        TOPIC_ASSIGNED => auto.owner.assigned,
        TOPIC_VERIFY => auto.owner.verify,
        TOPIC_RUN => auto.owner.run,
        _ => true,
    }
}

fn ping_of(notice: OwnerNotice) -> PingRequest {
    PingRequest {
        title: Some(notice.title),
        text: notice.text,
        level: Level::Ping,
        project: Some(notice.project),
        thread: Some(notice.thread),
        by: By::Auto,
        url: None,
    }
}

impl OwnerSink for OwnerPings {
    fn notify(&self, notice: OwnerNotice) {
        let key = told_key(&notice);
        if self.db.kv_get(&key).ok().flatten().is_some() {
            return;
        }
        if !self.shadow {
            let _ = self.db.kv_set(&key, "1");
        }
        if !switched_on(self.notify.auto(), &notice.topic) {
            return;
        }
        let notify = Arc::clone(&self.notify);
        let ping = ping_of(notice);
        thread::spawn(move || drop(notify.send(ping)));
    }
}

/// The worktrees of every project whose status asks their thread for a decision.
struct StaleWorktrees {
    service: Arc<WorktreeService>,
}

impl NoticeSource for StaleWorktrees {
    fn open(&self, _now_ms: i64) -> Vec<OpenNotice> {
        let mut open = Vec::new();
        for project in self.service.deps.store.projects() {
            for doc in self.service.list(&project) {
                if doc.status.needs_decision() {
                    open.push(OpenNotice {
                        reason: REASON.to_owned(),
                        project: project.clone(),
                        subject: doc.id,
                    });
                }
            }
        }
        open
    }
}

pub(crate) fn wire(svc: &Services) {
    let Some(wake) = &svc.wake else {
        return;
    };
    let mut service = lock(wake);
    if let Some(notify) = &svc.notify {
        service.set_owner_sink(Arc::new(OwnerPings {
            notify: Arc::clone(notify),
            db: Arc::clone(&svc.db),
            shadow: svc.shadow,
        }));
    }
    if let Some(worktrees) = &svc.worktrees {
        service.add_notice_source(Arc::new(StaleWorktrees {
            service: Arc::clone(worktrees),
        }));
    }
}
