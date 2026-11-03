//! The routes behind the pages' data layer (DESIGN 3.1): reads straight from the doc store, the
//! doc stream, and one write path. A page may only write `entries` (an owner post the sync
//! service applies); every other write is `read_only`.

use argus_core::{ApiError, Coll, ErrorCode, js};
use argus_store::Query;
use argus_sync::Source;
use serde_json::{Value, json};

use super::params::{coll, scope};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::sse::StreamRequest;
use crate::{Ctx, RouteOpts, Router};

/// The most ops one batch holds.
pub const MAX_BATCH: usize = 50;

fn read_only(coll: &str) -> ApiError {
    ApiError::new(
        ErrorCode::ReadOnly,
        format!("{coll} is read-only for pages; only entries can be posted"),
    )
}

fn rows(docs: Vec<argus_store::DocRow>) -> Value {
    Value::Array(
        docs.into_iter()
            .map(|r| json!({ "id": r.id, "data": r.data }))
            .collect(),
    )
}

async fn list(ctx: Ctx) -> Reply {
    let (scope, coll) = (scope(&ctx)?, coll(ctx.param("coll"))?);
    let docs = ctx.svc.db.docs();
    Ok(json_reply(
        200,
        &json!({ "seq": docs.seq()?, "docs": rows(docs.list(&scope, coll)?) }),
    ))
}

async fn get(ctx: Ctx) -> Reply {
    let (scope, coll) = (scope(&ctx)?, coll(ctx.param("coll"))?);
    let docs = ctx.svc.db.docs();
    let row = docs.get(&scope, coll, ctx.param("id"))?;
    let body = json!({ "seq": docs.seq()?, "id": ctx.param("id"), "exists": row.is_some(), "data": row.map(|r| r.data) });
    Ok(json_reply(200, &body))
}

async fn query(ctx: Ctx) -> Reply {
    let (scope, coll) = (scope(&ctx)?, coll(ctx.param("coll"))?);
    let q = Query::parse(&ctx.json().await?)?;
    let docs = ctx.svc.db.docs();
    Ok(json_reply(
        200,
        &json!({ "seq": docs.seq()?, "docs": rows(docs.query(&scope, coll, &q)?) }),
    ))
}

/// One page write: a set on `entries` is an owner post, nothing else is writable.
async fn write_doc(
    ctx: &Ctx,
    scope: &str,
    coll_name: &str,
    id: &Value,
    data: Option<&Value>,
) -> Result<Value, ApiError> {
    if coll_name != "entries" {
        return Err(read_only(coll_name));
    }
    let Some(obj) = data.and_then(Value::as_object) else {
        return Err(ApiError::bad_request("data must be an object"));
    };
    let Some(id) = id.as_str().filter(|i| !i.is_empty() && js::len(i) <= 200) else {
        return Err(ApiError::bad_request("invalid doc id"));
    };
    if obj.get("id").is_some_and(|d| d != id) {
        return Err(ApiError::bad_request(format!(
            "data.id \"{}\" does not match the path id \"{id}\"",
            js::string(&obj["id"])
        )));
    }
    if !ctx.svc.board_writes {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow mode: Argus does not write boards",
        ));
    }
    let sync = ctx
        .svc
        .sync
        .clone()
        .ok_or_else(|| ApiError::unavailable("the sync service is not available"))?;
    let mut body = obj.clone();
    body.insert("id".into(), json!(id));
    let r = sync
        .post_entry(scope, &Value::Object(body), Source::Page, None)
        .await?;
    Ok(serde_json::to_value(r).unwrap_or(Value::Null))
}

async fn put(ctx: Ctx) -> Reply {
    let (scope, coll) = (scope(&ctx)?, coll(ctx.param("coll"))?);
    let body = ctx.json().await?;
    let out = write_doc(
        &ctx,
        &scope,
        coll.as_str(),
        &json!(ctx.param("id")),
        body.get("data"),
    )
    .await?;
    Ok(json_reply(200, &out))
}

async fn refuse(ctx: Ctx) -> Reply {
    Err(read_only(coll(ctx.param("coll"))?.as_str()))
}

async fn batch_op(ctx: &Ctx, scope: &str, op: &Value) -> Result<Value, ApiError> {
    let kind = op["op"]
        .as_str()
        .filter(|k| ["set", "update", "delete"].contains(k));
    let Some(kind) = kind.filter(|_| op.is_object()) else {
        return Err(ApiError::bad_request(
            "each op is {op: set|update|delete, coll, id, data?}",
        ));
    };
    let name = op["coll"].as_str().unwrap_or_default();
    let c: Coll = coll(name).map_err(|_| {
        ApiError::bad_request(format!(
            "unknown collection \"{}\"",
            js::string(&op["coll"])
        ))
    })?;
    if kind != "set" {
        return Err(read_only(c.as_str()));
    }
    write_doc(ctx, scope, c.as_str(), &op["id"], op.get("data")).await
}

/// The `ops` list of a batch body: 1 to [`MAX_BATCH`] ops.
///
/// # Errors
/// 400 when the list is missing, empty or too long.
pub fn batch_ops(body: &Value) -> Result<&Vec<Value>, ApiError> {
    let ops = body["ops"]
        .as_array()
        .filter(|o| !o.is_empty())
        .ok_or_else(|| ApiError::bad_request("ops must be a non-empty array"))?;
    if ops.len() > MAX_BATCH {
        return Err(ApiError::bad_request(format!(
            "a batch holds at most {MAX_BATCH} ops"
        )));
    }
    Ok(ops)
}

async fn batch(ctx: Ctx) -> Reply {
    let scope = scope(&ctx)?;
    let body = ctx.json().await?;
    let ops = batch_ops(&body)?;
    let mut results = Vec::new();
    for op in ops {
        match batch_op(&ctx, &scope, op).await {
            Ok(r) => results.push(r),
            Err(e) => {
                results.push(e.body());
                break;
            }
        }
    }
    Ok(json_reply(200, &json!({ "results": results })))
}

fn stream_colls(raw: Option<&str>) -> Result<Vec<Coll>, ApiError> {
    let Some(raw) = raw.filter(|r| !r.is_empty()) else {
        return Ok(Coll::ALL.to_vec());
    };
    let mut out: Vec<Coll> = Vec::new();
    for name in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let c = coll(name)?;
        if !out.contains(&c) {
            out.push(c);
        }
    }
    if out.is_empty() {
        Err(ApiError::bad_request("c names no collection"))
    } else {
        Ok(out)
    }
}

fn since_of(ctx: &Ctx) -> Option<i64> {
    let raw = ctx.header("last-event-id").or_else(|| ctx.query("since"))?;
    js::trim(raw).parse::<i64>().ok().filter(|n| *n >= 0)
}

async fn stream(ctx: Ctx) -> Reply {
    let scope = scope(&ctx)?;
    let req = StreamRequest {
        scope: &scope,
        colls: stream_colls(ctx.query("c"))?,
        since: since_of(&ctx),
    };
    ctx.svc
        .fanout
        .attach(&ctx.svc.db, &req, ctx.svc.heartbeat, &ctx.svc.streams)
}

/// Registers the db shim routes.
pub fn register(r: &mut Router) {
    r.get("/api/p/:scope/db/:coll", list);
    r.get("/api/p/:scope/db/:coll/:id", get);
    r.post("/api/p/:scope/db/:coll/query", query);
    r.add(
        "PUT",
        "/api/p/:scope/db/:coll/:id",
        RouteOpts::default(),
        put,
    );
    r.add(
        "PATCH",
        "/api/p/:scope/db/:coll/:id",
        RouteOpts::default(),
        refuse,
    );
    r.add(
        "DELETE",
        "/api/p/:scope/db/:coll/:id",
        RouteOpts::default(),
        refuse,
    );
    r.post("/api/p/:scope/db/batch", batch);
    r.get("/api/p/:scope/stream", stream);
}
