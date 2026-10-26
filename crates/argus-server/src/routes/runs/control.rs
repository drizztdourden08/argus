//! Launches, starting, stopping, typing into and listing runs.

use argus_core::ApiError;
use argus_launchers::{ListFilter, Origin, StartOptions, check_key};
use axum::response::Response;
use serde_json::{Value, json, to_value};

use super::{actor, ids, is_page, object, refusal, service, unknown_run, writable};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

const STDIN_ROUTE_BYTES: usize = 16 * 1024;

fn answer(body: &Value) -> Response {
    json_reply(200, body)
}

fn key_of(ctx: &Ctx) -> Result<String, ApiError> {
    check_key(ctx.param("key"))
        .map(str::to_owned)
        .map_err(|e| refusal(&e))
}

async fn launches(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    let defs = runs.defs(&key_of(&ctx)?).map_err(|e| refusal(&e))?;
    Ok(answer(&json!({ "defs": defs })))
}

fn by_of(ctx: &Ctx, body: &Value) -> String {
    let named = body["by"].as_str().map(str::trim).filter(|b| !b.is_empty());
    if is_page(ctx) {
        "human".to_owned()
    } else {
        named.unwrap_or("cli").to_owned()
    }
}

fn start_options(ctx: &Ctx, body: &Value) -> StartOptions {
    StartOptions {
        origin: if is_page(ctx) {
            Origin::Page
        } else {
            Origin::Cli
        },
        by: Some(by_of(ctx, body)),
        agent: body["agent"].as_str().map(str::to_owned),
        reason: body["reason"].as_str().map(str::to_owned),
        visible: body["visible"].as_bool(),
    }
}

async fn start(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    writable(&ctx)?;
    let body = object(&ctx).await?;
    let name = body["name"].as_str().unwrap_or_default();
    let out = runs
        .start(ctx.param("key"), name, &start_options(&ctx, &body))
        .map_err(|e| refusal(&e))?;
    Ok(answer(
        &json!({ "run": to_value(&out.run).unwrap_or(Value::Null), "already": out.already }),
    ))
}

async fn stop(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    writable(&ctx)?;
    let (key, rid) = ids(&ctx)?;
    let run = runs
        .stop(&key, &rid, actor(&ctx))
        .await
        .map_err(|e| refusal(&e))?;
    Ok(answer(&json!({ "run": run })))
}

async fn stdin(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    writable(&ctx)?;
    let (key, rid) = ids(&ctx)?;
    let body = object(&ctx).await?;
    let Some(text) = body["text"].as_str() else {
        runs.get(&key, &rid).ok_or_else(|| unknown_run(&key, &rid))?;
        return Err(ApiError::bad_request("text must be a string"));
    };
    let bytes = runs
        .send_input(&key, &rid, text)
        .await
        .map_err(|e| refusal(&e))?;
    Ok(answer(&json!({ "ok": true, "bytes": bytes })))
}

fn filter_of(ctx: &Ctx) -> ListFilter {
    let limit = ctx.query("limit").and_then(|n| n.trim().parse::<i64>().ok());
    ListFilter {
        name: ctx.query("name").filter(|n| !n.is_empty()).map(str::to_owned),
        limit,
        ..ListFilter::default()
    }
    .with_statuses(ctx.query("status").unwrap_or_default())
}

async fn list(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    let rows = runs.list(&key_of(&ctx)?, &filter_of(&ctx));
    Ok(answer(&json!({ "runs": rows })))
}

async fn list_all(ctx: Ctx) -> Reply {
    let rows = service(&ctx)?.list_all(&filter_of(&ctx));
    Ok(answer(&json!({ "runs": rows })))
}

async fn one(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    let (key, rid) = ids(&ctx)?;
    let run = runs.get(&key, &rid).ok_or_else(|| unknown_run(&key, &rid))?;
    Ok(answer(&json!({ "run": run })))
}

pub(super) fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: STDIN_ROUTE_BYTES,
        ..RouteOpts::default()
    };
    r.get("/api/p/:key/launches", launches);
    r.post("/api/p/:key/runs", start);
    r.get("/api/p/:key/runs", list);
    r.get("/api/runs", list_all);
    r.get("/api/p/:key/runs/:rid", one);
    r.post("/api/p/:key/runs/:rid/stop", stop);
    r.add("POST", "/api/p/:key/runs/:rid/stdin", small, stdin);
}
