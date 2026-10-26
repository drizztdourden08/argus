//! The launcher wakes (no master launcher, a launcher unused past its window), ticked every
//! ten minutes. They read the boards, so they need the sync service; their wakes go to the wake
//! engine through the [`WakeBridge`](crate::adapters::link::WakeBridge).

use std::sync::Arc;

use argus_launchers::SystemClock;
use argus_launchers::launcher::{LauncherStale, StaleConfig};
use serde_json::Value;

use crate::adapters::activity::ActivityAdapter;
use crate::adapters::stale::StaleHost;
use crate::{ServeOptions, Services};

const TASK: &str = "launcher-stale";

fn number(setting: &Value, name: &str, default: i64) -> i64 {
    setting[name]
        .as_i64()
        .filter(|n| *n > 0)
        .unwrap_or(default)
}

/// `config.launchers`: `tickMs`, `masterWaitMs`, `masterRepeatMs`, `staleRepeatMs`.
fn config_of(svc: &Services) -> StaleConfig {
    let setting = svc
        .config()
        .extra
        .get("launchers")
        .cloned()
        .unwrap_or(Value::Null);
    let base = StaleConfig::default();
    StaleConfig {
        tick_ms: number(&setting, "tickMs", base.tick_ms),
        master_wait_ms: number(&setting, "masterWaitMs", base.master_wait_ms),
        master_repeat_ms: number(&setting, "masterRepeatMs", base.master_repeat_ms),
        stale_repeat_ms: number(&setting, "staleRepeatMs", base.stale_repeat_ms),
    }
}

pub(crate) fn build(svc: &Services, _opts: &ServeOptions) -> Option<Arc<LauncherStale>> {
    svc.sync.as_ref()?;
    let env = StaleHost::new(
        svc.boards(),
        svc.runs.clone(),
        Arc::clone(&svc.db),
        Arc::clone(&svc.wake_link),
        ActivityAdapter::of(svc),
        svc.shadow,
    );
    Some(Arc::new(LauncherStale::new(
        Arc::new(env),
        Arc::new(SystemClock),
        config_of(svc),
    )))
}

pub(crate) fn start(svc: &Arc<Services>) {
    if let Some(stale) = svc.launcher_stale.as_ref().filter(|_| svc.tasks.stale_timer) {
        svc.tasks.keep(TASK, stale.start());
    }
}

pub(crate) fn stop(svc: &Services) {
    svc.tasks.abort(TASK);
}
