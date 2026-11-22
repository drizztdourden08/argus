//! The timer of the rules (I-190): every 30 seconds the rules that are due go to the hosted child
//! through the owner's chat link, held while a turn runs. With no live child the run is skipped, and
//! either way the rule moves to its next time and one activity line says what happened.

use std::sync::Arc;
use std::time::Duration;

use argus_core::clock::now_ms;
use argus_overlord::rules::due;
use serde_json::json;

use super::rules::{deliver, record};
use crate::Services;
use crate::adapters::overlord::rules::SqlRules;
use crate::services::tasks::Timer;

const TASK: &str = "overlord-rules";
const EVERY: Duration = Duration::from_secs(30);

/// One pass: delivers or skips every due rule.
fn tick(svc: &Services) -> Result<(), String> {
    let rules = SqlRules::open(svc.db.clone(), now_ms()).map_err(|e| e.to_string())?;
    let all = rules.list().map_err(|e| e.to_string())?;
    let ids = due(&all, now_ms());
    for rule in all.iter().filter(|rule| ids.contains(&rule.id)) {
        let ran = deliver(svc, rule);
        record(svc, &rules, rule, ran);
    }
    Ok(())
}

async fn tick_loop(svc: Arc<Services>) {
    loop {
        tokio::time::sleep(EVERY).await;
        let held = Arc::clone(&svc);
        let pass = tokio::task::spawn_blocking(move || tick(&held)).await;
        if !matches!(pass, Ok(Ok(_))) {
            let why = format!("{pass:?}");
            svc.log
                .warn("overlord rules tick failed", json!({ "pass": why }));
        }
    }
}

/// Starts the timer with the server; a shadow run and a run with timers off keep it down.
pub(crate) fn start(svc: &Arc<Services>) {
    if svc.shadow || !svc.tasks.runs(Timer::Wake) {
        return;
    }
    svc.tasks
        .keep(TASK, tokio::spawn(tick_loop(Arc::clone(svc))));
}

pub(crate) fn stop(svc: &Services) {
    svc.tasks.abort(TASK);
}
