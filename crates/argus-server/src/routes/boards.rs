//! `GET /api/boards` lists every board with its id, name, old keys and role. `PATCH
//! /api/boards/:id` with `{name}` renames one; the id may also be an old key.

use serde_json::{Value, json};

use super::blocking::blocking;
use super::board_table;
use super::write::{object_body, writable};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

async fn listing(ctx: Ctx) -> Reply {
    let home = ctx.svc.paths.coord_home.clone();
    let rows = blocking(move || board_table::list(&home)).await?;
    Ok(json_reply(200, &Value::Array(rows)))
}

async fn rename(ctx: Ctx) -> Reply {
    writable(&ctx)?;
    let body = object_body(&ctx).await?;
    let name = body["name"].as_str().unwrap_or_default().to_owned();
    let home = ctx.svc.paths.coord_home.clone();
    let id = ctx.param("id").to_owned();
    let done = blocking(move || board_table::rename(&home, &id, &name)).await??;
    Ok(json_reply(200, &json!(done)))
}

/// Registers the board routes.
pub fn register(r: &mut Router) {
    r.get("/api/boards", listing);
    r.add("PATCH", "/api/boards/:id", RouteOpts::default(), rename);
}
