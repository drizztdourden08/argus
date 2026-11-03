//! The trail numbers of a board (`argus_board::trail::summaries`, I-77):
//!
//! - `GET /api/p/:key/trails` is `{trails: {"I-n": {depth, open}}}` for every blocked item of
//!   the board that is not settled. The answer is kept per board until the database moves on: a
//!   board changes only through an export, which moves the revision (the seq of the docs), and a
//!   trail reads other boards too, so any change drops the kept answer.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, PoisonError};

use argus_board::trail::{Summary, summaries};
use argus_core::ApiError;
use serde_json::{Map, Value, json};

use super::blocking::blocking;
use super::trail::{known_key, load_boards};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

/// The answers kept: board (with its coord home) to the revision it was made at and the answer.
static KEPT: LazyLock<Mutex<HashMap<String, (i64, Value)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn kept(slot: &str, revision: i64) -> Option<Value> {
    let table = KEPT.lock().unwrap_or_else(PoisonError::into_inner);
    table
        .get(slot)
        .filter(|(at, _)| *at == revision)
        .map(|(_, answer)| answer.clone())
}

fn keep(slot: String, revision: i64, answer: &Value) {
    let mut table = KEPT.lock().unwrap_or_else(PoisonError::into_inner);
    table.insert(slot, (revision, answer.clone()));
}

fn answer(found: &[Summary]) -> Value {
    let trails: Map<String, Value> = found
        .iter()
        .map(|s| (s.item.clone(), json!({ "depth": s.depth, "open": s.open })))
        .collect();
    json!({ "trails": trails })
}

async fn list(ctx: Ctx) -> Reply {
    let key = known_key(&ctx)?;
    let revision = ctx.svc.db.docs().seq()?;
    let slot = format!("{}\n{key}", ctx.svc.paths.coord_home.display());
    if let Some(hit) = kept(&slot, revision) {
        return Ok(json_reply(200, &hit));
    }
    let boards = load_boards(&ctx).await?;
    let body = blocking(move || answer(&summaries(&boards, &key))).await?;
    keep(slot, revision, &body);
    Ok(json_reply(200, &body))
}

/// Registers the trails route.
pub fn register(r: &mut Router) {
    r.get("/api/p/:key/trails", list);
}
