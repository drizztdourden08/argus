//! The pages and the build's files. `/` is the overview; on a board alias host it is a 302 to
//! that board with the `?thread=` claim. `argus.localhost` is always the overview and
//! `overlord.localhost` the Overlord's own board. `/p/<key>/` is the board page of a known key.

use argus_board::names::{ARGUS, MAIN_KEY, display_name};
use argus_core::ApiError;
use argus_core::ids::is_key;
use axum::http::header;
use axum::response::Response;

use super::params::known_keys;
use crate::reply::{bytes_reply, escape, html_page, info_page, redirect};
use crate::router::Reply;
use crate::security::host_alias;
use crate::web::{PageVars, content_type, render_page};
use crate::{Ctx, Router, Services};

fn page(ctx: &Ctx, name: &str, key: &str, base: &str) -> Response {
    let vars = PageVars {
        name,
        key,
        base,
        version: Services::version(),
    };
    html_page(200, render_page(&ctx.svc.web.index(), &vars))
}

async fn home(ctx: Ctx) -> Reply {
    let alias = ctx
        .header("host")
        .and_then(|h| host_alias(h, ctx.svc.port(), ctx.svc.doors()));
    match alias {
        Some(a) if a.reserved => {
            if a.key == "overlord" && a.thread.is_none() && known_keys(&ctx).contains(MAIN_KEY) {
                return Ok(redirect(&format!("/p/{MAIN_KEY}/?thread=coordinator"), &[]));
            }
        }
        Some(a) if known_keys(&ctx).contains(&a.key) => {
            let thread = ctx
                .query("thread")
                .filter(|t| !t.is_empty())
                .or(a.thread.as_deref())
                .unwrap_or("coordinator");
            let target = format!("/p/{}/?thread={}", a.key, encode(thread));
            return Ok(redirect(&target, &[]));
        }
        _ => {}
    }
    Ok(page(&ctx, ARGUS, "_portfolio", "/"))
}

fn encode(s: &str) -> String {
    s.bytes().fold(String::new(), |mut out, b| {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            out.push(char::from(b));
        } else {
            out.push('%');
            out.push_str(&argus_core::config::hex(&[b]).to_uppercase());
        }
        out
    })
}

fn query_suffix(ctx: &Ctx) -> String {
    if ctx.query_all().is_empty() {
        return String::new();
    }
    let pairs = ctx
        .query_all()
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()));
    let query = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    format!("?{query}")
}

async fn slash(ctx: Ctx) -> Reply {
    let target = format!("/p/{}/{}", encode(ctx.param("key")), query_suffix(&ctx));
    Ok(redirect(&target, &[]))
}

async fn board(ctx: Ctx) -> Reply {
    let key = ctx.param("key").to_owned();
    if !is_key(&key) || !known_keys(&ctx).contains(&key) {
        let what = escape(&format!("There is no board for \"{key}\"."));
        return Ok(html_page(404, info_page("Not found", &[&what])));
    }
    let board = ctx.svc.sync.as_ref().and_then(|s| s.board(&key));
    let named = board
        .as_ref()
        .and_then(|b| b["name"].as_str().or_else(|| b["project"].as_str()));
    Ok(page(
        &ctx,
        &display_name(&key, named, None),
        &key,
        &format!("/p/{key}/"),
    ))
}

fn build_file(ctx: &Ctx, rel: &str, cache: &str) -> Reply {
    let bad = rel.is_empty() || rel.contains(['\0', '\\']) || rel.split('/').any(|s| s == "..");
    let file = if bad { None } else { ctx.svc.web.file(rel) };
    let (Some(file), Some(kind)) = (file, content_type(rel)) else {
        return Err(ApiError::not_found("no such file"));
    };
    let len = file.len().to_string();
    Ok(bytes_reply(
        200,
        file.into_owned(),
        &[
            (header::CONTENT_TYPE, kind),
            (header::CONTENT_LENGTH, &len),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, cache),
        ],
    ))
}

async fn static_file(ctx: Ctx) -> Reply {
    let rel = ctx.param("*").to_owned();
    build_file(&ctx, &rel, "no-cache")
}

async fn asset(ctx: Ctx) -> Reply {
    let rel = format!("assets/{}", ctx.param("*"));
    build_file(&ctx, &rel, "public, max-age=31536000, immutable")
}

/// Registers the page routes.
pub fn register(r: &mut Router) {
    r.get("/", home);
    r.get("/index.html", home);
    r.get("/p/:key", slash);
    r.get("/p/:key/", board);
    r.get("/static/*", static_file);
    r.get("/assets/*", asset);
}
