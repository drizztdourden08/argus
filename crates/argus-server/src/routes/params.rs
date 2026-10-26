//! Route params checked as the Node routes check them, and the board keys a page may be served
//! for.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use argus_core::ids::{is_key, is_scope};
use argus_core::{ApiError, Coll};
use argus_sync::SyncService;

use crate::Ctx;

/// The `:scope` param.
///
/// # Errors
/// 400 for anything but a key or `_portfolio`.
pub fn scope(ctx: &Ctx) -> Result<String, ApiError> {
    let s = ctx.param("scope");
    if is_scope(s) {
        Ok(s.to_owned())
    } else {
        Err(ApiError::bad_request(format!("invalid scope \"{s}\"")))
    }
}

/// The `:key` param.
///
/// # Errors
/// 400 for anything but a project key.
pub fn key(ctx: &Ctx) -> Result<String, ApiError> {
    let k = ctx.param("key");
    if is_key(k) {
        Ok(k.to_owned())
    } else {
        Err(ApiError::bad_request(format!(
            "invalid project key \"{k}\""
        )))
    }
}

/// A collection name.
///
/// # Errors
/// 400 for an unknown collection.
pub fn coll(name: &str) -> Result<Coll, ApiError> {
    name.parse()
        .map_err(|_| ApiError::bad_request(format!("unknown collection \"{name}\"")))
}

/// The sync service.
///
/// # Errors
/// 503 when it is off.
pub fn sync(ctx: &Ctx) -> Result<Arc<SyncService>, ApiError> {
    ctx.svc
        .sync
        .clone()
        .ok_or_else(|| ApiError::unavailable("the sync service is not running"))
}

/// Board folders found on disk.
#[must_use]
pub fn scan_boards(coord_home: &Path) -> Vec<String> {
    let Ok(dirs) = std::fs::read_dir(coord_home) else {
        return Vec::new();
    };
    let mut keys: Vec<String> = dirs
        .filter_map(Result::ok)
        .filter(|d| d.path().is_dir())
        .filter_map(|d| d.file_name().to_str().map(str::to_owned))
        .filter(|k| is_key(k) && k != "dashboard" && coord_home.join(k).join("board.json").exists())
        .collect();
    keys.sort();
    keys
}

/// Keys a board page may be served for: the sync service's projects, plus boards on disk.
#[must_use]
pub fn known_keys(ctx: &Ctx) -> BTreeSet<String> {
    let mut keys: BTreeSet<String> = scan_boards(&ctx.svc.paths.coord_home).into_iter().collect();
    if let Some(sync) = &ctx.svc.sync {
        keys.extend(
            sync.projects()
                .into_iter()
                .filter(|p| p.has_board)
                .map(|p| p.key),
        );
    }
    keys
}
