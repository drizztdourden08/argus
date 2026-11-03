//! Cross-board dependencies (`docs/architecture.md`, `argus_board::deps`). The coord verbs call
//! these routes; Argus alone writes the ledger and the two mirrors. A success answers
//! `{dep, text}` (the list `{deps, text}`, one dependency with both items `{dep, requester,
//! assignee, text}`); a refusal answers the usual error body with the message of the rule and
//! 400, 404 or 409.
//!
//! - `POST /api/deps` asks: `{project, item, actor, target, to?, title, description?, ac?, onto?}`
//! - `POST /api/deps/:id/close` `{as, reason?, actor}`, `.../reopen` `{actor}`, `.../cancel`
//!   `{reason?, actor}`; the owner's page is always the actor `owner`
//! - `GET /api/deps?project=&view=waiting|for-me|all&state=` lists
//! - `GET /api/deps/:id?project=` shows one, `GET /api/deps/by-item/:project/:item` finds the one
//!   of an item
//!
//! While `deps.enabled` is false every route answers the 404 `dependencies are off`; in shadow
//! mode the writes answer `read_only`.

use argus_board::deps::{
    AskRequest, BoardRoster, CancelRequest, CloseRequest, Dep, ListFilter, ReopenRequest, ask,
    by_item, cancel, close, lines, list, reopen, show,
};
use argus_core::ApiError;
use axum::response::Response;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::deps::{act, run};
use crate::{AuthKind, Ctx, Router};

/// The body as the type a verb takes. The owner's page is the actor `owner`, whatever it sends.
async fn body_of<T: DeserializeOwned>(ctx: &Ctx) -> Result<T, ApiError> {
    let mut body = ctx.json().await?;
    let page = ctx.auth == Some(AuthKind::Page);
    if let (true, Some(map)) = (page, body.as_object_mut()) {
        map.insert("actor".to_owned(), json!("owner"));
    }
    serde_json::from_value(body).map_err(|e| ApiError::bad_request(format!("invalid body: {e}")))
}

fn answer(dep: &Dep, text: &str) -> Response {
    json_reply(200, &json!({ "dep": dep.to_value(), "text": text }))
}

async fn open(ctx: Ctx) -> Reply {
    let request: AskRequest = body_of(&ctx).await?;
    let done = act(&ctx.svc, move |home| {
        ask(home, &request, &BoardRoster::new(home))
    })
    .await?;
    Ok(answer(&done.dep, &done.text))
}

async fn close_one(ctx: Ctx) -> Reply {
    let request: CloseRequest = body_of(&ctx).await?;
    let id = ctx.param("id").to_owned();
    let done = act(&ctx.svc, move |home| close(home, &id, &request)).await?;
    Ok(answer(&done.dep, &done.text))
}

async fn reopen_one(ctx: Ctx) -> Reply {
    let request: ReopenRequest = body_of(&ctx).await?;
    let id = ctx.param("id").to_owned();
    let done = act(&ctx.svc, move |home| reopen(home, &id, &request)).await?;
    Ok(answer(&done.dep, &done.text))
}

async fn cancel_one(ctx: Ctx) -> Reply {
    let request: CancelRequest = body_of(&ctx).await?;
    let id = ctx.param("id").to_owned();
    let done = act(&ctx.svc, move |home| cancel(home, &id, &request)).await?;
    Ok(answer(&done.dep, &done.text))
}

async fn listing(ctx: Ctx) -> Reply {
    let query = |name: &str| ctx.query(name).map(str::to_owned);
    let (project, view, state) = (query("project"), query("view"), query("state"));
    let deps = run(&ctx.svc, move |home| {
        let filter = ListFilter::parse(project.as_deref(), view.as_deref(), state.as_deref())?;
        list(home, &filter)
    })
    .await?;
    let records: Vec<Value> = deps.iter().map(Dep::to_value).collect();
    Ok(json_reply(
        200,
        &json!({ "deps": records, "text": lines(&deps) }),
    ))
}

async fn one(ctx: Ctx) -> Reply {
    let id = ctx.param("id").to_owned();
    let project = ctx.query("project").map(str::to_owned);
    let shown = run(&ctx.svc, move |home| show(home, &id, project.as_deref())).await?;
    Ok(json_reply(
        200,
        &json!({
            "dep": shown.dep.to_value(),
            "requester": shown.requester,
            "assignee": shown.assignee,
            "text": shown.dep.line(),
        }),
    ))
}

async fn of_item(ctx: Ctx) -> Reply {
    let (project, item) = (
        ctx.param("project").to_owned(),
        ctx.param("item").to_owned(),
    );
    let dep = run(&ctx.svc, move |home| by_item(home, &project, &item)).await?;
    Ok(answer(&dep, &dep.line()))
}

/// Registers the dependency routes.
pub fn register(r: &mut Router) {
    r.post("/api/deps", open);
    r.post("/api/deps/:id/close", close_one);
    r.post("/api/deps/:id/reopen", reopen_one);
    r.post("/api/deps/:id/cancel", cancel_one);
    r.get("/api/deps", listing);
    r.get("/api/deps/:id", one);
    r.get("/api/deps/by-item/:project/:item", of_item);
}
