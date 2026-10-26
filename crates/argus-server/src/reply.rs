//! Response builders with the Node headers: JSON answers, error answers, pages and redirects.

use argus_core::{ApiError, ErrorCode};
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, StatusCode, header};
use axum::response::Response;
use serde_json::Value;

/// The JSON content type of every API answer.
pub const JSON_TYPE: &str = "application/json; charset=utf-8";

/// The page policy (DESIGN 8, without the CDN sources the Node pages needed).
pub const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data: blob:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'";

fn status(code: u16) -> StatusCode {
    StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// A response with a body and headers.
#[must_use]
pub fn bytes_reply(code: u16, body: impl Into<Body>, headers: &[(HeaderName, &str)]) -> Response {
    let mut res = Response::new(body.into());
    *res.status_mut() = status(code);
    for (name, value) in headers {
        if let Ok(v) = HeaderValue::from_str(value) {
            res.headers_mut().append(name.clone(), v);
        }
    }
    res
}

fn json_bytes(body: &Value) -> Vec<u8> {
    serde_json::to_vec(body).unwrap_or_default()
}

/// A route's JSON answer: `no-store` and `nosniff`.
#[must_use]
pub fn json_reply(code: u16, body: &Value) -> Response {
    let text = json_bytes(body);
    let len = text.len().to_string();
    bytes_reply(
        code,
        text,
        &[
            (header::CONTENT_TYPE, JSON_TYPE),
            (header::CONTENT_LENGTH, &len),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
    )
}

/// A route's error answer: `Retry-After: 1` on a busy database or a held board lock, and the
/// connection closed after a body that was too large.
#[must_use]
pub fn error_reply(e: &ApiError) -> Response {
    let mut res = json_reply(e.code.status(), &e.body());
    let extra = match e.code {
        ErrorCode::DbBusy | ErrorCode::BoardLocked => Some((header::RETRY_AFTER, "1")),
        ErrorCode::TooLarge => Some((header::CONNECTION, "close")),
        _ => None,
    };
    if let Some((name, value)) = extra {
        res.headers_mut()
            .insert(name, HeaderValue::from_static(value));
    }
    res
}

/// An answer of the checks before any route (no `nosniff`, as in Node).
#[must_use]
pub fn plain_error(e: &ApiError, allow: Option<&str>) -> Response {
    let text = json_bytes(&e.body());
    let len = text.len().to_string();
    let mut headers = vec![
        (header::CONTENT_TYPE, JSON_TYPE),
        (header::CONTENT_LENGTH, len.as_str()),
        (header::CACHE_CONTROL, "no-store"),
    ];
    if let Some(a) = allow {
        headers.push((header::ALLOW, a));
    }
    bytes_reply(e.code.status(), text, &headers)
}

/// An HTML page with the page headers.
#[must_use]
pub fn html_page(code: u16, html: String) -> Response {
    let len = html.len().to_string();
    bytes_reply(
        code,
        html,
        &[
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_LENGTH, &len),
            (header::CONTENT_SECURITY_POLICY, CSP),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::REFERRER_POLICY, "no-referrer"),
            (header::CACHE_CONTROL, "no-store"),
        ],
    )
}

/// A 302 with `no-store`, plus extra headers.
#[must_use]
pub fn redirect(location: &str, extra: &[(HeaderName, &str)]) -> Response {
    let mut headers = vec![
        (header::LOCATION, location),
        (header::CACHE_CONTROL, "no-store"),
    ];
    headers.extend(extra.iter().cloned());
    bytes_reply(302, Body::empty(), &headers)
}

/// HTML escaping of a text.
#[must_use]
pub fn escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_owned(),
            '<' => "&lt;".to_owned(),
            '>' => "&gt;".to_owned(),
            '"' => "&quot;".to_owned(),
            '\'' => "&#39;".to_owned(),
            other => other.to_string(),
        })
        .collect()
}

/// A small page that says one thing (sign-in, not found, missing build).
#[must_use]
pub fn info_page(title: &str, lines: &[&str]) -> String {
    let body = lines
        .iter()
        .fold(String::new(), |acc, l| acc + "<p>" + l + "</p>");
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title>\n<style>body{{font:16px/1.5 system-ui,sans-serif;max-width:34rem;margin:15vh auto;padding:0 1rem;color:#222}}code{{background:#eee;padding:.1em .35em;border-radius:3px}}@media(prefers-color-scheme:dark){{body{{background:#16181c;color:#ddd}}code{{background:#2a2d33}}}}</style></head>\n<body><h1>{}</h1>{body}</body></html>",
        escape(title),
        escape(title)
    )
}
