//! The switch-over and the rollback of the hosted Overlord (I-173). `POST /api/overlord/mode` runs
//! one step of `argus_overlord::mode` and its effect here, under the control lock, so two children
//! never exist at once. What the steps write outside the child is in `switch_board.rs`.

use std::sync::Arc;

use argus_core::ErrorCode;
use argus_overlord::mode::{Mode, ModeFault, Step, step};
use argus_overlord::seat::SEAT;
use argus_overlord::shadow::THREAD as SHADOW_THREAD;
use serde_json::json;

use super::config;
use super::control::{self, Refused};
use super::live::{Origin, try_start, try_start_from};
use super::setting::save_field;
use super::shadow::desktop_session;
use super::switch_board::{ask_handover, end_threads, hand_back, set_aside};
use crate::Services;

/// What a change of mode did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Switched {
    pub(crate) from: Mode,
    pub(crate) to: Mode,
    /// The mode was already the one asked.
    pub(crate) already: bool,
}

/// Why a change of mode did not happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SwitchFault {
    /// Off to live or live to shadow.
    Jump(ModeFault),
    /// Going live with no shadow session and no desktop session to start from.
    NoSession,
    /// The child did not start or stop.
    Control(Refused),
    /// `config.json` could not be written.
    Save,
}

impl SwitchFault {
    pub(crate) fn sentence(self) -> String {
        match self {
            SwitchFault::Jump(fault) => fault.to_string(),
            SwitchFault::NoSession => {
                "no session to go live from: the shadow has none and the desktop Overlord has none"
                    .to_owned()
            }
            SwitchFault::Control(refused) => refused.sentence().to_owned(),
            SwitchFault::Save => "the Overlord mode could not be saved".to_owned(),
        }
    }

    pub(crate) fn code(self) -> ErrorCode {
        match self {
            SwitchFault::Save => ErrorCode::Internal,
            _ => ErrorCode::Conflict,
        }
    }
}

fn save(svc: &Services, mode: Mode) -> Result<(), SwitchFault> {
    save_field(svc, "mode", json!(mode.as_str())).map_err(|_| SwitchFault::Save)
}

/// Runs a blocking effect (board lock, files, the wake store) off the async workers.
async fn blocking(svc: &Arc<Services>, effect: impl FnOnce(&Services) + Send + 'static) {
    let held = Arc::clone(svc);
    let _ = tokio::task::spawn_blocking(move || effect(&held)).await;
}

/// Stops the child, if one runs, and waits for its supervisor to end.
async fn stop_child(svc: &Services) -> Result<(), SwitchFault> {
    match control::end(svc).await {
        Ok(()) | Err(Refused::NotRunning) => Ok(()),
        Err(other) => Err(SwitchFault::Control(other)),
    }
}

/// Off to shadow: saves the mode and starts the shadow child; a start that fails puts `off` back.
fn shadow_on(svc: &Arc<Services>) -> Result<(), SwitchFault> {
    save(svc, Mode::Shadow)?;
    try_start(svc).map_err(|fault| {
        let _ = save(svc, Mode::Off);
        SwitchFault::Control(Refused::Start(fault))
    })
}

/// Shadow to off: stops the shadow child, then saves the mode.
async fn shadow_off(svc: &Services) -> Result<(), SwitchFault> {
    stop_child(svc).await?;
    save(svc, Mode::Off)
}

/// The live child's first session: the shadow's own when it has one, else a fork of the desktop
/// session. Read before anything stops or ends.
fn live_origin(svc: &Services) -> Option<Origin> {
    let shadow = svc.overlord.state().session_id;
    shadow
        .map(Origin::Resume)
        .or_else(|| desktop_session(&svc.boards()).map(Origin::Fork))
}

/// Shadow to live. The desktop Overlord is asked for its handover first, so the wake reaches its
/// listener before the live sink is attached. A live start that fails puts the shadow back.
async fn go_live(svc: &Arc<Services>) -> Result<(), SwitchFault> {
    let origin = live_origin(svc).ok_or(SwitchFault::NoSession)?;
    blocking(svc, |svc| {
        set_aside(svc);
        ask_handover(svc);
    })
    .await;
    stop_child(svc).await?;
    save(svc, Mode::Live)?;
    if let Err(fault) = try_start_from(svc, origin) {
        let _ = save(svc, Mode::Shadow);
        let _ = try_start(svc);
        return Err(SwitchFault::Control(Refused::Start(fault)));
    }
    blocking(svc, |svc| end_threads(svc, &[SEAT, SHADOW_THREAD])).await;
    Ok(())
}

/// Live to off: stops the child by pid, saves the mode, marks the hosted seat ended and writes the
/// handover file for the desktop Overlord.
async fn roll_back(svc: &Arc<Services>) -> Result<(), SwitchFault> {
    let session_id = svc.overlord.state().session_id;
    stop_child(svc).await?;
    save(svc, Mode::Off)?;
    blocking(svc, move |svc| hand_back(svc, session_id)).await;
    Ok(())
}

/// Changes the mode to `to`, running the effect of the step.
///
/// # Errors
/// A [`SwitchFault`]: a refused jump, no session to go live from, a child that did not start or
/// stop, or a config that could not be saved.
pub(crate) async fn switch(svc: &Arc<Services>, to: Mode) -> Result<Switched, SwitchFault> {
    let _one = svc.overlord.control.lock().await;
    let from = config::mode(svc);
    let taken = step(from, to).map_err(SwitchFault::Jump)?;
    match taken {
        Step::Already => {}
        Step::ShadowOn => shadow_on(svc)?,
        Step::ShadowOff => shadow_off(svc).await?,
        Step::GoLive => go_live(svc).await?,
        Step::Rollback => roll_back(svc).await?,
    }
    svc.log.info(
        "overlord mode",
        json!({ "from": from.as_str(), "to": to.as_str() }),
    );
    Ok(Switched {
        from,
        to,
        already: taken == Step::Already,
    })
}
