//! `GET /api/owner/avatar`: the image the owner wears as an avatar, decoded from the data URL the
//! `_portfolio` board keeps in `owner.avatar`. The export url carries `?v=<hash>`, so the answer is
//! cached for good. An avatar that is not an image, or none at all, is a 404.

use super::blocking::blocking;
use super::stored_image::image_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

async fn serve(ctx: Ctx) -> Reply {
    let svc = ctx.svc.clone();
    let stored = blocking(move || {
        let board = svc.boards().board("_portfolio")?;
        board["owner"]["avatar"].as_str().map(str::to_owned)
    })
    .await?;
    image_reply(stored.as_deref(), "the owner has no image avatar")
}

/// Registers the owner avatar route.
pub fn register(r: &mut Router) {
    r.get("/api/owner/avatar", serve);
}
