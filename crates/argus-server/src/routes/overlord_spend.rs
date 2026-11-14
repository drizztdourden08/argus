//! The spend meter of the hosted Overlord, for the owner's page only:
//! `GET /api/overlord/spend` is `{month, spentUsd, budgetUsd, warnAt, leftUsd, note}`. The figure
//! is the sum of the turns' costs this month by the local calendar, an estimate.

use argus_accounts::alerts::local_offset_seconds;
use argus_core::clock::now_ms;
use argus_core::{ApiError, ErrorCode};
use argus_overlord::ports::SpendLedger;
use argus_overlord::spend::{left_usd, month_key};
use serde_json::{Value, json};

use super::blocking::blocking;
use super::overlord::web_only;
use crate::adapters::overlord::spend::SqlLedger;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::config::budget;
use crate::{Ctx, Router};

const NOTE: &str = "estimate; the Console is the bill";

fn ledger_error() -> ApiError {
    ApiError::new(ErrorCode::Internal, "the spend ledger failed")
}

/// The reply body of the spend meter.
pub(super) async fn body(ctx: &Ctx) -> Result<Value, ApiError> {
    let month = month_key(now_ms(), local_offset_seconds());
    let db = ctx.svc.db.clone();
    let key = month.clone();
    let spent = blocking(move || {
        SqlLedger::open(db)
            .and_then(|ledger| ledger.month_total(&key))
            .map_err(|_| ledger_error())
    })
    .await??;
    let setting = budget(&ctx.svc);
    Ok(json!({
        "month": month,
        "spentUsd": spent,
        "budgetUsd": setting.budget_usd,
        "warnAt": setting.warn_at,
        "leftUsd": left_usd(spent, setting.budget_usd),
        "note": NOTE,
    }))
}

async fn show(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    Ok(json_reply(200, &body(&ctx).await?))
}

/// Registers the spend route.
pub fn register(r: &mut Router) {
    r.get("/api/overlord/spend", show);
}
