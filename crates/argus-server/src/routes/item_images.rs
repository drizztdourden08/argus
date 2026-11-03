//! `POST /api/p/:key/items/:id/images?caption=...`: a screenshot pasted or dropped into an item's
//! rich text or its Media tab. The body is the image itself (png, jpeg, gif or webp, 10 MiB at most,
//! the same rules as a proof); the item keeps at most 12 and the same bytes once. The answer is
//! `{item, image: {n, file, url, caption}}`; the text then holds `![caption](img:n)`. In shadow
//! mode it answers `read_only`.

use argus_core::js;
use serde_json::json;

use super::actions::{image_body, writer};
use super::params::key;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

async fn upload(ctx: Ctx) -> Reply {
    let src = image_body(&ctx).await?;
    let key = key(&ctx)?;
    let item = ctx.param("id").to_owned();
    let asked = js::trim(ctx.query("caption").unwrap_or_default()).to_owned();
    let caption = if asked.is_empty() {
        format!("{item} image")
    } else {
        asked
    };
    let image = writer(&ctx)?
        .attach_item_image(&key, &item, (src, &caption), "human")
        .await?;
    let file = image.url.rsplit('/').next().unwrap_or_default().to_owned();
    Ok(json_reply(
        200,
        &json!({ "item": item, "image": { "n": image.n, "file": file, "url": image.url, "caption": caption } }),
    ))
}

/// Registers the item image route.
pub fn register(r: &mut Router) {
    r.add(
        "POST",
        "/api/p/:key/items/:id/images",
        RouteOpts {
            max_bytes: usize::MAX,
            images: true,
            ..RouteOpts::default()
        },
        upload,
    );
}
