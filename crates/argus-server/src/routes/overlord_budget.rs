//! The monthly budget of the hosted Overlord, for the owner's page only: `PUT /api/overlord/budget`
//! with `{budgetUsd}` (a positive number, or null for no cap) saves `overlord.budgetUsd` in `config.json` and answers like the spend route.

use argus_core::{ApiError, ErrorCode};

use super::overlord::web_only;
use super::overlord_spend::body;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::budget::save_budget;
use crate::{Ctx, RouteOpts, Router};

const SMALL: usize = 8 * 1024;
const BUDGET_MAX: f64 = 10_000.0;

fn amount(raw: &serde_json::Value) -> Result<Option<f64>, ApiError> {
    let bad = || ApiError::bad_request("budgetUsd must be null or a number above 0 up to 10000");
    match &raw["budgetUsd"] {
        serde_json::Value::Null if raw.get("budgetUsd").is_some() => Ok(None),
        value => value
            .as_f64()
            .filter(|v| *v > 0.0 && *v <= BUDGET_MAX)
            .map(Some)
            .ok_or_else(bad),
    }
}

async fn put(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let usd = amount(&ctx.json().await?)?;
    save_budget(&ctx.svc, usd)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "the budget could not be saved"))?;
    Ok(json_reply(200, &body(&ctx).await?))
}

/// Registers the budget route.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    r.add("PUT", "/api/overlord/budget", small, put);
}
