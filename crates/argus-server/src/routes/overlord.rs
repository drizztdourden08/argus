//! The Anthropic key of the hosted Overlord, for the owner's settings page only:
//!
//! - `GET /api/overlord/key` is `{set, last4, savedAt}` (`last4` and `savedAt` are `null` when no key is saved)
//! - `PUT /api/overlord/key` takes `{key}` and answers the same shape
//! - `DELETE /api/overlord/key` removes the key and answers the same shape
//!
//! Every verb needs the page's own origin: a CLI, a bearer or a foreign origin is refused with 403. The key is
//! never in a reply, a log line, an activity entry or an error text.

use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use argus_core::clock::iso_from_ms;
use argus_core::{ApiError, ErrorCode};
use argus_overlord::ports::{SecretFault, SecretStore};
use serde_json::{Value, json};

use super::blocking::blocking;
use crate::adapters::overlord::key::DpapiKeyStore;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::security::origin_ok;
use crate::{AuthKind, Ctx, RouteOpts, Router};

const SMALL: usize = 8 * 1024;

fn web_only(ctx: &Ctx, strict: bool) -> Result<(), ApiError> {
    let page = ctx.auth == Some(AuthKind::Page)
        && origin_ok(&ctx.headers, ctx.svc.port(), ctx.svc.doors(), strict);
    if page {
        return Ok(());
    }
    Err(ApiError::new(
        ErrorCode::Forbidden,
        "only the owner's page handles the Overlord key",
    ))
}

fn store_error(_: &SecretFault) -> ApiError {
    ApiError::new(ErrorCode::Internal, "the key store failed")
}

fn status(store: &DpapiKeyStore) -> Result<Value, ApiError> {
    let last4 = store.hint().map_err(|e| store_error(&e))?;
    let saved_at = last4.as_ref().and_then(|_| store.saved_at()).and_then(|t| {
        let ms = t.duration_since(UNIX_EPOCH).ok()?.as_millis();
        i64::try_from(ms).ok().map(iso_from_ms)
    });
    Ok(json!({ "set": last4.is_some(), "last4": last4, "savedAt": saved_at }))
}

async fn with_store<T: Send + 'static>(
    ctx: &Ctx,
    f: impl FnOnce(&DpapiKeyStore) -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    let home: PathBuf = ctx.svc.paths.home.clone();
    blocking(move || f(&DpapiKeyStore::new(&home))).await?
}

async fn show(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    let body = with_store(&ctx, status).await?;
    Ok(json_reply(200, &body))
}

async fn save(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let body = ctx.json().await?;
    let key = body["key"]
        .as_str()
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| ApiError::bad_request("send the key as text"))?
        .to_owned();
    let body = with_store(&ctx, move |s| {
        s.save(&key).map_err(|e| store_error(&e))?;
        status(s)
    })
    .await?;
    Ok(json_reply(200, &body))
}

async fn remove(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let body = with_store(&ctx, |s| {
        s.clear().map_err(|e| store_error(&e))?;
        status(s)
    })
    .await?;
    Ok(json_reply(200, &body))
}

/// Registers the Overlord key routes.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    r.get("/api/overlord/key", show);
    r.add("PUT", "/api/overlord/key", small, save);
    r.add("DELETE", "/api/overlord/key", small, remove);
}
