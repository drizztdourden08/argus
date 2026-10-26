//! The activity log service. It is the one service built before [`Services`] exists, since
//! every other service writes to it. `start` writes the docs and keeps them whole after a hard
//! resync; `started` and `stopped` are the dashboard's own two lines.

use std::sync::Arc;

use argus_core::clock::SystemClock;
use argus_core::log::Logger;
use argus_store::{ACTIVITY_SCOPE, ActivityLevel, ActivityLog, Db, NewActivity};

use crate::Services;

const KIND: &str = "dashboard";

pub(crate) fn build(db: &Arc<Db>, log: &Arc<Logger>) -> Arc<ActivityLog> {
    Arc::new(ActivityLog::new(
        Arc::clone(db),
        Arc::new(SystemClock),
        Arc::clone(log),
    ))
}

pub(crate) fn start(svc: &Arc<Services>) {
    svc.activity.mirror();
    let log = Arc::clone(&svc.activity);
    svc.keep(svc.bus.on_reset(move |e| {
        if e.scope == ACTIVITY_SCOPE {
            log.mirror();
        }
    }));
}

fn announce(svc: &Services, what: &str) {
    let text = format!("dashboard {what} (pid {})", std::process::id());
    svc.activity
        .add(NewActivity::new(KIND, ActivityLevel::Info, text));
}

pub(crate) fn started(svc: &Services) {
    announce(svc, "started");
}

pub(crate) fn stopped(svc: &Services) {
    announce(svc, "stopped");
}
