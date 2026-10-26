//! `/files`: test proofs kept with the board and run attachments. A name must match a strict
//! pattern and the file must resolve inside its own folder.

use std::path::{Path, PathBuf};

use argus_core::ApiError;
use argus_core::ids::is_key;
use argus_store::rusqlite::{OptionalExtension, params};
use axum::http::header;

use crate::reply::bytes_reply;
use crate::router::Reply;
use crate::web::content_type;
use crate::{Ctx, Router};

const IMAGES: [&str; 4] = ["image/png", "image/jpeg", "image/gif", "image/webp"];
const FILE_POLICY: &str = "default-src 'none'; sandbox";

fn name_ok(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

fn image_type(name: &str) -> Option<&'static str> {
    content_type(name).filter(|t| IMAGES.contains(t))
}

/// `file` when it is a regular file inside `root`.
fn inside(root: &Path, file: &Path) -> Option<PathBuf> {
    let root = std::fs::canonicalize(root).ok()?;
    let full = std::fs::canonicalize(root.join(file)).ok()?;
    (full.starts_with(&root) && full.is_file()).then_some(full)
}

fn send_file(file: &Path, kind: &str, extra: &[(header::HeaderName, &str)]) -> Reply {
    let bytes = std::fs::read(file).map_err(|_| ApiError::not_found("the file is gone"))?;
    let len = bytes.len().to_string();
    let mut headers = vec![
        (header::CONTENT_TYPE, kind),
        (header::CONTENT_LENGTH, len.as_str()),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::CACHE_CONTROL, "private, max-age=300"),
        (header::CONTENT_SECURITY_POLICY, FILE_POLICY),
    ];
    headers.extend(extra.iter().cloned());
    Ok(bytes_reply(200, bytes, &headers))
}

async fn proof(ctx: Ctx) -> Reply {
    let (key, name) = (ctx.param("key"), ctx.param("name"));
    let kind = image_type(name)
        .filter(|_| is_key(key) && name_ok(name))
        .ok_or_else(|| ApiError::not_found("no such proof"))?;
    let dir = ctx.svc.paths.coord_home.join(key).join("proofs");
    let file = inside(&dir, Path::new(name)).ok_or_else(|| ApiError::not_found("no such proof"))?;
    send_file(&file, kind, &[])
}

fn attachment_row(ctx: &Ctx, id: &str, key: &str) -> Option<(String, String)> {
    ctx.svc.db.with_conn(|c| {
        c.query_row(
            "SELECT path, mime FROM attachments WHERE id = ? AND scope = ?",
            params![id, key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .ok()
        .flatten()
    })
}

async fn attachment(ctx: Ctx) -> Reply {
    let (key, id) = (ctx.param("key"), ctx.param("attId"));
    let row = if is_key(key) {
        attachment_row(&ctx, id, key)
    } else {
        None
    };
    let (path, mime) = row.ok_or_else(|| ApiError::not_found("no such attachment"))?;
    let stored = PathBuf::from(&path);
    let file = inside(&ctx.svc.paths.attachments, &stored)
        .or_else(|| inside(&ctx.svc.paths.coord_home.join(key).join("proofs"), &stored));
    let file = file.ok_or_else(|| ApiError::not_found("the attachment file is gone"))?;
    if IMAGES.contains(&mime.as_str()) {
        return send_file(&file, &mime, &[]);
    }
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_owned();
    send_file(
        &file,
        "application/octet-stream",
        &[(
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{name}\""),
        )],
    )
}

/// Registers the file routes.
pub fn register(r: &mut Router) {
    r.get("/files/:key/proofs/:name", proof);
    r.get("/files/:key/a/:attId", attachment);
}
