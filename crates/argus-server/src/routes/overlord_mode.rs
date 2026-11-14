//! The switch-over and the rollback, for the owner's page only: `POST /api/overlord/mode` with
//! `{mode: off|shadow|live}` runs one step and answers `{from, to, already}`. A jump from off to live
//! or from live to shadow is 409 with the fault.

use argus_core::ApiError;
use argus_overlord::mode::{Mode, ModeFault};
use serde_json::json;

use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::switch::{SwitchFault, switch};
use crate::{Ctx, RouteOpts, Router};

const SMALL: usize = 8 * 1024;

fn refusal(fault: SwitchFault) -> ApiError {
    let error = ApiError::new(fault.code(), fault.sentence());
    match fault {
        SwitchFault::Jump(ModeFault::Jump { from, to }) => {
            error.with_details(json!({ "from": from.as_str(), "to": to.as_str() }))
        }
        _ => error,
    }
}

async fn change(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let body = ctx.json().await?;
    let to = body["mode"]
        .as_str()
        .and_then(Mode::parse)
        .ok_or_else(|| ApiError::bad_request("mode must be off, shadow or live"))?;
    let done = switch(&ctx.svc, to).await.map_err(refusal)?;
    Ok(json_reply(
        200,
        &json!({
            "from": done.from.as_str(),
            "to": done.to.as_str(),
            "already": done.already,
        }),
    ))
}

/// Registers the mode route.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    r.add("POST", "/api/overlord/mode", small, change);
}
