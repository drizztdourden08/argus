//! Start, stop and restart of the hosted Overlord for the owner's page. One action at a time.

use std::sync::Arc;
use std::time::Duration;

use super::live::{StartFault, try_start};
use crate::Services;

/// How long a stop waits for the supervisor to end.
const STOP_WAIT: Duration = Duration::from_secs(15);

/// Why an action was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refused {
    Start(StartFault),
    NotRunning,
    StopTimeout,
}

impl Refused {
    pub(crate) fn sentence(self) -> &'static str {
        match self {
            Refused::Start(fault) => fault.sentence(),
            Refused::NotRunning => "the hosted Overlord is not running",
            Refused::StopTimeout => "the hosted Overlord did not stop in time",
        }
    }
}

/// Starts the child.
pub(crate) async fn start(svc: &Arc<Services>) -> Result<(), Refused> {
    let _one = svc.overlord.control.lock().await;
    try_start(svc).map_err(Refused::Start)
}

/// Stops the child and waits for its supervisor to end; the caller holds the control lock.
pub(super) async fn end(svc: &Services) -> Result<(), Refused> {
    let mut task = svc.overlord.begin_stop().ok_or(Refused::NotRunning)?;
    if tokio::time::timeout(STOP_WAIT, &mut task).await.is_err() {
        svc.overlord.return_task(task);
        return Err(Refused::StopTimeout);
    }
    svc.overlord.clear();
    Ok(())
}

/// Stops the child and waits for its supervisor to end.
pub(crate) async fn stop(svc: &Services) -> Result<(), Refused> {
    let _one = svc.overlord.control.lock().await;
    end(svc).await
}

/// Stops the child if one runs, waits for the end, then starts a new one. Never two at once.
pub(crate) async fn restart(svc: &Arc<Services>) -> Result<(), Refused> {
    let _one = svc.overlord.control.lock().await;
    match end(svc).await {
        Ok(()) | Err(Refused::NotRunning) => {}
        Err(other) => return Err(other),
    }
    try_start(svc).map_err(Refused::Start)
}
