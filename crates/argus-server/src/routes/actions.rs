//! The action routes of DESIGN 3.3: owner posts, one-click test results, proof images and notes
//! sent down from the overview. A post the board refused or found in conflict is still a 200
//! whose body says so. In shadow mode they all answer `read_only`.

use std::sync::Arc;

use argus_core::{ApiError, ErrorCode, ids::is_key, js};
use argus_sync::{ProofSource, Source, SyncService, new_post_id};
use serde_json::{Value, json};

use super::params::{key, sync};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{AuthKind, Ctx, RouteOpts, Router};

const OUTCOMES: [&str; 4] = ["pass", "fail", "partial", "blocked"];
const IMAGE_TYPES: [&str; 4] = ["image/png", "image/jpeg", "image/gif", "image/webp"];
const ENTRY_BYTES: usize = 256 * 1024;

fn source_of(ctx: &Ctx) -> Source {
    if ctx.auth == Some(AuthKind::Cli) {
        Source::Cli
    } else {
        Source::Page
    }
}

fn writer(ctx: &Ctx) -> Result<Arc<SyncService>, ApiError> {
    if ctx.svc.shadow {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow mode: Argus does not write boards",
        ));
    }
    sync(ctx)
}

fn answer(r: &impl serde::Serialize) -> axum::response::Response {
    json_reply(200, &serde_json::to_value(r).unwrap_or(Value::Null))
}

async fn entries(ctx: Ctx) -> Reply {
    let body = ctx.json().await?;
    let source = source_of(&ctx);
    let key = key(&ctx)?;
    Ok(answer(
        &writer(&ctx)?
            .post_entry(&key, &body, source, Some(source.as_str()))
            .await?,
    ))
}

async fn result(ctx: Ctx) -> Reply {
    let body = ctx.json().await?;
    if !body["outcome"]
        .as_str()
        .is_some_and(|o| OUTCOMES.contains(&o))
    {
        return Err(ApiError::bad_request(format!(
            "outcome must be one of {}",
            OUTCOMES.join(", ")
        )));
    }
    if !matches!(body["text"], Value::Null | Value::String(_)) {
        return Err(ApiError::bad_request("text must be a string"));
    }
    let source = source_of(&ctx);
    let text = body["text"].as_str().unwrap_or_default();
    let entry = json!({ "id": new_post_id(), "kind": "result", "outcome": body["outcome"], "text": text, "replyTo": ctx.param("id"), "item": null, "thread": null });
    let key = key(&ctx)?;
    Ok(answer(
        &writer(&ctx)?
            .post_entry(&key, &entry, source, Some(source.as_str()))
            .await?,
    ))
}

/// The body of an image upload as a proof source: the declared type must be an image type and the
/// body is capped at the attach limit.
async fn image_body(ctx: &Ctx) -> Result<ProofSource, ApiError> {
    let mime = ctx
        .header("content-type")
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if !IMAGE_TYPES.contains(&mime.as_str()) {
        return Err(ApiError::new(
            ErrorCode::UnsupportedMedia,
            "send the image as image/png, image/jpeg, image/gif or image/webp",
        ));
    }
    let max = usize::try_from(ctx.svc.config().attach.max_bytes).unwrap_or(usize::MAX);
    let bytes = ctx.raw(max).await?;
    Ok(ProofSource::Bytes {
        bytes: bytes.to_vec(),
        mime: Some(mime),
    })
}

/// `?option=N`: the 1-based choice a picture belongs to.
fn option_param(ctx: &Ctx) -> Result<Option<i64>, ApiError> {
    ctx.query("option")
        .filter(|o| !o.is_empty())
        .map(|o| {
            o.parse::<i64>()
                .map_err(|_| ApiError::bad_request("option must be a number"))
        })
        .transpose()
}

async fn entry_images(ctx: Ctx) -> Reply {
    let src = image_body(&ctx).await?;
    let key = key(&ctx)?;
    let caption = ctx.query("caption").unwrap_or_default().to_owned();
    let entry = ctx.param("id").to_owned();
    let option = option_param(&ctx)?;
    let image = writer(&ctx)?
        .attach_entry_image(&key, (&entry, option), (src, &caption), "human")
        .await?;
    let shown = if js::trim(&caption).is_empty() {
        format!("{entry} image")
    } else {
        js::trim(&caption).to_owned()
    };
    Ok(json_reply(
        200,
        &json!({ "entry": entry, "image": { "n": image.n, "url": image.url, "caption": shown, "option": option } }),
    ))
}

async fn proofs(ctx: Ctx) -> Reply {
    let src = image_body(&ctx).await?;
    let key = key(&ctx)?;
    let caption = ctx.query("caption").unwrap_or_default().to_owned();
    let test = ctx.param("id").to_owned();
    let proof = writer(&ctx)?
        .attach_proof(&key, &test, src, &caption, "human")
        .await?;
    let shown = if js::trim(&caption).is_empty() {
        format!("{test} proof")
    } else {
        js::trim(&caption).to_owned()
    };
    Ok(json_reply(
        200,
        &json!({ "test": test, "proof": { "n": proof.n, "url": proof.url, "caption": shown } }),
    ))
}

async fn send(ctx: Ctx) -> Reply {
    let body = ctx.json().await?;
    let project = body["project"]
        .as_str()
        .filter(|p| *p == "all" || is_key(p));
    let Some(project) = project else {
        return Err(ApiError::bad_request(
            "project must be a project key or \"all\"",
        ));
    };
    let Some(text) = body["text"].as_str().filter(|t| !js::trim(t).is_empty()) else {
        return Err(ApiError::bad_request("text is required"));
    };
    let via = source_of(&ctx).as_str();
    Ok(answer(
        &writer(&ctx)?
            .send_to_lead(project, text, body["decision"] == json!(true), via)
            .await?,
    ))
}

/// Registers the action routes.
pub fn register(r: &mut Router) {
    r.add(
        "POST",
        "/api/p/:key/entries",
        RouteOpts {
            max_bytes: ENTRY_BYTES,
            ..RouteOpts::default()
        },
        entries,
    );
    r.post("/api/p/:key/tests/:id/result", result);
    r.add(
        "POST",
        "/api/p/:key/tests/:id/proofs",
        RouteOpts {
            max_bytes: usize::MAX,
            images: true,
            ..RouteOpts::default()
        },
        proofs,
    );
    r.add(
        "POST",
        "/api/p/:key/entries/:id/images",
        RouteOpts {
            max_bytes: usize::MAX,
            images: true,
            ..RouteOpts::default()
        },
        entry_images,
    );
    r.post("/api/portfolio/send", send);
}
