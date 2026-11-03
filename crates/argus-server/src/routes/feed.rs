//! The update feed: the release store, read only, for the apps' updater.
//!
//! - `GET|HEAD /releases/<project>/<channel>/<path>` on the main listener, and the same at
//!   `http://releases.localhost/<project>/<channel>/<path>` on the localhost door (the Host
//!   `releases.localhost` reads as the `/releases` prefix)
//! - `<channel>/latest.yml` (also `latest-mac.yml`, `latest-linux.yml`) is the newest version's
//!   with its urls pointing into the version folder; `<version>/<file>` is the file itself
//! - one byte range is a 206 (`Accept-Ranges`, `Content-Range`) for differential downloads
//! - no directory listing, and a path that is not plain names inside the store is a 404
//! - no credential is asked: the updater of an app has none, the content is the owner's own
//!   builds, and the Host and peer checks of every request still apply

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use argus_core::ApiError;
use argus_records::releases::range::{Span, span};
use argus_records::releases::{Served, content_type};
use axum::body::Body;
use axum::http::{HeaderMap, Method, header};
use axum::response::Response;
use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};
use tokio_util::io::ReaderStream;

use super::blocking::blocking;
use super::params::records;
use crate::reply::bytes_reply;
use crate::router::Reply;
use crate::security::{Doors, host_alias};
use crate::{AuthMode, Ctx, RouteOpts, Router};

/// The host label of the feed beside the board hosts.
const HOST_LABEL: &str = "releases";
const FILE_POLICY: &str = "default-src 'none'; sandbox";

/// The request path as the router sees it: on the feed host the path gets the `/releases`
/// prefix, so one route serves both forms.
#[must_use]
pub(crate) fn feed_path(headers: &HeaderMap, port: u16, doors: Doors<'_>, path: String) -> String {
    let host = headers.get(header::HOST).and_then(|h| h.to_str().ok());
    let on_feed = host
        .and_then(|h| host_alias(h, port, doors))
        .is_some_and(|a| a.key == HOST_LABEL && a.thread.is_none() && !a.reserved);
    if on_feed {
        format!("/releases{path}")
    } else {
        path
    }
}

fn etag(meta: &std::fs::Metadata) -> String {
    let ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis());
    format!("\"{:x}-{ms:x}\"", meta.len())
}

fn common(kind: &str) -> Vec<(header::HeaderName, String)> {
    vec![
        (header::CONTENT_TYPE, kind.to_owned()),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        (header::CACHE_CONTROL, "no-cache".to_owned()),
        (header::CONTENT_SECURITY_POLICY, FILE_POLICY.to_owned()),
    ]
}

fn reply(code: u16, body: Body, headers: &[(header::HeaderName, String)]) -> Response {
    let pairs: Vec<(header::HeaderName, &str)> = headers
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str()))
        .collect();
    bytes_reply(code, body, &pairs)
}

fn text_reply(text: String, head: bool) -> Response {
    let mut headers = common(content_type("latest.yml"));
    headers.push((header::CONTENT_LENGTH, text.len().to_string()));
    reply(
        200,
        if head {
            Body::empty()
        } else {
            Body::from(text)
        },
        &headers,
    )
}

/// The range the request gets: none when an `If-Range` names another version of the file.
fn asked_span(request: &HeaderMap, tag: &str, len: u64) -> Span {
    let range = request.get(header::RANGE).and_then(|v| v.to_str().ok());
    let stale = request
        .get(header::IF_RANGE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v != tag);
    if stale { Span::Whole } else { span(range, len) }
}

async fn stream(path: &Path, from: u64, count: u64) -> Option<Body> {
    let mut file = tokio::fs::File::open(path).await.ok()?;
    file.seek(std::io::SeekFrom::Start(from)).await.ok()?;
    Some(Body::from_stream(ReaderStream::new(file.take(count))))
}

async fn file_reply(path: PathBuf, request: &HeaderMap, head: bool) -> Reply {
    let gone = || ApiError::not_found("no such file");
    let meta = tokio::fs::metadata(&path).await.map_err(|_| gone())?;
    let (len, tag) = (meta.len(), etag(&meta));
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let mut headers = common(content_type(name));
    headers.push((header::ACCEPT_RANGES, "bytes".to_owned()));
    headers.push((header::ETAG, tag.clone()));
    let (code, from, count) = match asked_span(request, &tag, len) {
        Span::Whole => (200, 0, len),
        Span::Part(a, b) => {
            headers.push((header::CONTENT_RANGE, format!("bytes {a}-{b}/{len}")));
            (206, a, b - a + 1)
        }
        Span::Unsatisfiable => {
            headers.push((header::CONTENT_RANGE, format!("bytes */{len}")));
            headers.push((header::CONTENT_LENGTH, "0".to_owned()));
            return Ok(reply(416, Body::empty(), &headers));
        }
    };
    headers.push((header::CONTENT_LENGTH, count.to_string()));
    let body = if head {
        Body::empty()
    } else {
        stream(&path, from, count).await.ok_or_else(gone)?
    };
    Ok(reply(code, body, &headers))
}

async fn serve(ctx: Ctx) -> Reply {
    let records = records(&ctx)?;
    let rel = ctx.param("*").to_owned();
    let found = blocking(move || records.releases.serve(&rel)).await?;
    let head = ctx.method == Method::HEAD;
    match found.ok_or_else(|| ApiError::not_found("no such file"))? {
        Served::Text(text) => Ok(text_reply(text, head)),
        Served::File(path) => file_reply(path, &ctx.headers, head).await,
    }
}

/// Registers the feed route.
pub fn register(r: &mut Router) {
    let open = RouteOpts {
        auth: AuthMode::None,
        ..RouteOpts::default()
    };
    r.add("GET", "/releases/*", open, serve);
}
