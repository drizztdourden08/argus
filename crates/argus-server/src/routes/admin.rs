//! Health, version, the project list, the admin actions and the sign-in routes.

use std::collections::HashMap;

use argus_core::ApiError;
use argus_core::config::{new_token, save_config};
use argus_core::ids::PORTFOLIO;
use axum::http::header;
use serde_json::{Value, json};
use std::sync::PoisonError;

use super::params::{scan_boards, sync};
use crate::reply::{html_page, info_page, json_reply, redirect};
use crate::router::{AuthMode, Reply};
use crate::security::{cookie_header, safe_next};
use crate::{Ctx, RouteOpts, Router, Services};

/// Live and queued runs, read from the table so it works with or without the runs service.
#[derive(Default)]
struct RunCounts {
    by_scope: HashMap<String, i64>,
    running: i64,
    queued: i64,
}

fn run_counts(svc: &Services) -> RunCounts {
    svc.db.with_conn(|c| {
        let mut out = RunCounts::default();
        let Ok(mut st) = c.prepare("SELECT scope, status, COUNT(*) FROM runs WHERE status IN ('running', 'queued') GROUP BY scope, status") else { return out };
        let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)));
        for (scope, status, n) in rows.into_iter().flatten().flatten() {
            if status == "running" {
                out.running += n;
                out.by_scope.insert(scope, n);
            } else {
                out.queued += n;
            }
        }
        out
    })
}

pub(crate) fn scopes(svc: &Services) -> Vec<String> {
    svc.db.with_conn(|c| {
        let Ok(mut st) = c.prepare("SELECT DISTINCT scope FROM docs ORDER BY scope") else {
            return Vec::new();
        };
        st.query_map([], |r| r.get(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    })
}

/// The body of `GET /api/health`, which the MCP `health` tool answers too.
///
/// # Errors
/// When the schema version cannot be read.
pub(crate) fn health_body(svc: &Services) -> Result<Value, ApiError> {
    let counts = run_counts(svc);
    let watcher = svc.sync.as_ref().map_or_else(
        || json!({ "ok": false, "lastEventAt": null, "errors": 0 }),
        |s| json!(s.health()),
    );
    Ok(json!({
        "ok": true,
        "version": Services::version(),
        "instance": svc.instance,
        "startedAt": svc.started_at,
        "port": svc.port(),
        "schema": svc.db.user_version()?,
        "scopes": scopes(svc),
        "watcher": watcher,
        "runs": { "running": counts.running, "queued": counts.queued },
        "sse": { "clients": svc.streams.load(std::sync::atomic::Ordering::SeqCst) },
        "modules": { "sync": svc.sync.is_some(), "runs": svc.runs.is_some(), "problems": {} },
        "impl": "argus",
        "shadow": svc.shadow,
    }))
}

async fn health(ctx: Ctx) -> Reply {
    Ok(json_reply(200, &health_body(&ctx.svc)?))
}

async fn version(ctx: Ctx) -> Reply {
    let build = ctx.svc.web.build_id();
    let body = json!({
        "version": Services::version(),
        "rust": env!("CARGO_PKG_RUST_VERSION"),
        "schema": ctx.svc.db.user_version()?,
        "pages": { "board": build, "portfolio": build },
    });
    Ok(json_reply(200, &body))
}

/// `[{key, name, root, url, hasBoard, running}]` from the sync service, else from the boards on
/// disk.
#[must_use]
pub fn list_projects(svc: &Services) -> Vec<Value> {
    let running = run_counts(svc).by_scope;
    let base: Vec<Value> = if let Some(s) = &svc.sync {
        s.projects().into_iter().map(|p| json!(p)).collect()
    } else {
        let roots = svc.config().project_roots;
        scan_boards(&svc.paths.coord_home)
            .into_iter()
            .map(|k| json!({ "key": k, "name": k, "root": roots.get(&k), "hasBoard": true, "url": format!("/p/{k}/") }))
            .collect()
    };
    base.into_iter()
        .map(|p| {
            let key = p["key"].as_str().unwrap_or_default().to_owned();
            json!({ "key": key, "name": p["name"], "root": p["root"], "url": p["url"], "hasBoard": p["hasBoard"], "running": running.get(&key).copied().unwrap_or(0) })
        })
        .collect()
}

async fn projects(ctx: Ctx) -> Reply {
    Ok(json_reply(200, &Value::Array(list_projects(&ctx.svc))))
}

async fn resync(ctx: Ctx) -> Reply {
    let sync =
        sync(&ctx).map_err(|_| ApiError::unavailable("the sync service is not available"))?;
    let body = ctx.json().await?;
    let project = match &body["project"] {
        Value::Null => None,
        v => Some(argus_core::js::string(v)),
    };
    if let Some(p) = project
        .as_deref()
        .filter(|p| !argus_core::ids::is_key(p) && *p != PORTFOLIO)
    {
        return Err(ApiError::bad_request(format!("invalid project \"{p}\"")));
    }
    let out = sync
        .resync(project.as_deref(), body["hard"] == json!(true))
        .await?;
    Ok(json_reply(200, &json!(out)))
}

async fn shutdown(ctx: Ctx) -> Reply {
    ctx.svc.stop.cancel();
    Ok(json_reply(202, &json!({ "ok": true })))
}

async fn rotate(ctx: Ctx) -> Reply {
    let svc = &ctx.svc;
    let config = {
        let mut c = svc.config.write().unwrap_or_else(PoisonError::into_inner);
        c.token = new_token();
        c.clone()
    };
    save_config(&svc.paths, &config, svc.env_port).map_err(|_| ApiError::internal())?;
    svc.codes.clear();
    svc.log.info("token rotated", json!({}));
    Ok(json_reply(200, &json!({ "token": config.token })))
}

async fn code(ctx: Ctx) -> Reply {
    let (code, expires_at) = ctx.svc.codes.issue();
    Ok(json_reply(
        200,
        &json!({ "code": code, "expiresAt": expires_at }),
    ))
}

async fn auth(ctx: Ctx) -> Reply {
    let next = safe_next(ctx.query("next")).to_owned();
    if !ctx.svc.require_token() {
        return Ok(redirect(&next, &[(header::REFERRER_POLICY, "no-referrer")]));
    }
    if !ctx.svc.codes.redeem(ctx.query("code").unwrap_or_default()) {
        let page = info_page(
            "Sign-in link expired",
            &[
                "That link was used already or is older than a minute. Run <code>dashboard open</code> again.",
            ],
        );
        return Ok(html_page(401, page));
    }
    let cookie = cookie_header(&ctx.svc.token());
    Ok(redirect(
        &next,
        &[
            (header::SET_COOKIE, &cookie),
            (header::REFERRER_POLICY, "no-referrer"),
        ],
    ))
}

/// Registers the admin routes.
pub fn register(r: &mut Router) {
    let bearer = RouteOpts {
        auth: AuthMode::Bearer,
        ..RouteOpts::default()
    };
    r.get("/api/health", health);
    r.get("/api/version", version);
    r.get("/api/projects", projects);
    r.post("/api/admin/resync", resync);
    r.add("POST", "/api/admin/shutdown", bearer, shutdown);
    r.add("POST", "/api/admin/token/rotate", bearer, rotate);
    r.add("POST", "/api/auth/code", bearer, code);
    r.add(
        "GET",
        "/auth",
        RouteOpts {
            auth: AuthMode::None,
            ..RouteOpts::default()
        },
        auth,
    );
}
