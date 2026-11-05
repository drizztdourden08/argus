//! `GET /api/p/:key/icon`: the image a board wears as its icon, decoded from the data URL the board
//! keeps. The card's url carries `?v=<hash>`, so the answer is cached for good. A board whose icon
//! is not an image, or that has none, is a 404.

use super::blocking::blocking;
use super::params::key;
use super::stored_image::image_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

async fn serve(ctx: Ctx) -> Reply {
    let key = key(&ctx)?;
    let svc = ctx.svc.clone();
    let stored = blocking(move || {
        let board = svc.boards().board(&key)?;
        board["icon"].as_str().map(str::to_owned)
    })
    .await?;
    image_reply(stored.as_deref(), "this board has no image icon")
}

/// Registers the board icon route.
pub fn register(r: &mut Router) {
    r.get("/api/p/:key/icon", serve);
}
