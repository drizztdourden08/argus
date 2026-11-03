//! The dependency trail of one item (`argus_board::trail`, I-77):
//!
//! - `GET /api/p/:key/items/:id/trail` is the tree `{depth, open, root}`: the item and, under it
//!   in `children`, everything it waits on across boards, each node with its project, item,
//!   title, `type`, `lane`, `stage`, `assignee` and the `edge` from its parent. A node already on
//!   the path has `cycle: true` and no children. 404 for a board or an item that is not there.
//!
//! `GET /api/p/:key/trails`, the numbers of every blocked item of a board, is in `trails`.

use std::sync::Arc;

use argus_board::json::doc;
use argus_board::trail::{Boards, walk};
use argus_core::ApiError;
use argus_core::ids::PORTFOLIO;
use serde_json::to_value;

use super::blocking::blocking;
use super::params::{key, known_keys};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

/// The key of the `:key` param, when a board of it is known.
///
/// # Errors
/// 400 for a bad key, 404 when there is no such board.
pub(super) fn known_key(ctx: &Ctx) -> Result<String, ApiError> {
    let key = key(ctx)?;
    if known_keys(ctx).contains(&key) {
        Ok(key)
    } else {
        Err(ApiError::not_found(format!("no board for \"{key}\"")))
    }
}

/// Every known board and the portfolio board (the ledger), as the walk reads them.
///
/// # Errors
/// 500 when the reading thread panicked.
pub(super) async fn load_boards(ctx: &Ctx) -> Result<Boards, ApiError> {
    let svc = Arc::clone(&ctx.svc);
    let mut keys: Vec<String> = known_keys(ctx).into_iter().collect();
    keys.push(PORTFOLIO.to_owned());
    blocking(move || {
        let reader = svc.boards();
        keys.into_iter()
            .filter_map(|key| reader.board(&key).map(|board| (key, board)))
            .collect()
    })
    .await
}

async fn tree(ctx: Ctx) -> Reply {
    let key = known_key(&ctx)?;
    let item = ctx.param("id").to_owned();
    let boards = load_boards(&ctx).await?;
    let found = boards.get(&key).and_then(|b| doc(b, "items", &item));
    if found.is_none() {
        return Err(ApiError::not_found(format!("no item {item} on {key}")));
    }
    let trail = blocking(move || walk(&boards, &key, &item)).await?;
    Ok(json_reply(200, &to_value(&trail).unwrap_or_default()))
}

/// Registers the trail route.
pub fn register(r: &mut Router) {
    r.get("/api/p/:key/items/:id/trail", tree);
}
