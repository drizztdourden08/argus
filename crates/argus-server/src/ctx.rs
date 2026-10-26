//! What a route handler gets: the services, the request parts, the decoded params and query,
//! who is calling, and the body readers.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use argus_core::{ApiError, ErrorCode};
use axum::body::Body;
use axum::http::{HeaderMap, Method};
use bytes::Bytes;
use serde_json::{Value, json};

use crate::body::read_limited;
use crate::{RouteOpts, Services};

/// Who sent a request: a browser page, or the CLI (a bearer or a caller without fetch metadata).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKind {
    /// The owner's page.
    Page,
    /// The CLI or a script.
    Cli,
}

/// One request, as a handler sees it.
pub struct Ctx {
    /// The services.
    pub svc: Arc<Services>,
    /// The method.
    pub method: Method,
    /// The normalized path.
    pub path: String,
    /// The headers.
    pub headers: HeaderMap,
    /// Who is calling; `None` on routes without checks.
    pub auth: Option<AuthKind>,
    params: HashMap<String, String>,
    query: Vec<(String, String)>,
    opts: RouteOpts,
    body: Mutex<Option<Body>>,
}

fn is_json_type(t: &str) -> bool {
    let lower = t.to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("application/") else {
        return false;
    };
    let sub_end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || "_.+-".contains(c)))
        .unwrap_or(rest.len());
    let sub = &rest[..sub_end];
    sub == "json" || sub.ends_with("+json")
}

/// The parts of a request a [`Ctx`] is built from.
pub(crate) struct Parts {
    pub method: Method,
    pub path: String,
    pub headers: HeaderMap,
    pub query: Vec<(String, String)>,
    pub body: Body,
}

impl Ctx {
    pub(crate) fn new(
        svc: Arc<Services>,
        parts: Parts,
        params: HashMap<String, String>,
        opts: RouteOpts,
        auth: Option<AuthKind>,
    ) -> Self {
        Ctx {
            svc,
            method: parts.method,
            path: parts.path,
            headers: parts.headers,
            auth,
            params,
            query: parts.query,
            opts,
            body: Mutex::new(Some(parts.body)),
        }
    }

    /// A route param (decoded).
    #[must_use]
    pub fn param(&self, name: &str) -> &str {
        self.params.get(name).map_or("", String::as_str)
    }

    /// The first value of a query parameter.
    #[must_use]
    pub fn query(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// A header as text.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    fn take_body(&self) -> Body {
        self.body
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .unwrap_or_else(Body::empty)
    }

    /// The raw body, at most `max` bytes (the route's limit when smaller).
    ///
    /// # Errors
    /// 413 past the limit.
    pub async fn raw(&self, max: usize) -> Result<Bytes, ApiError> {
        read_limited(
            &self.headers,
            self.take_body(),
            max.min(self.opts.max_bytes),
        )
        .await
    }

    /// The JSON body: an empty body is `{}`; anything else must be JSON.
    ///
    /// # Errors
    /// 413 past the route's limit, 415 for another content type, 400 for invalid JSON.
    pub async fn json(&self) -> Result<Value, ApiError> {
        let buf = read_limited(&self.headers, self.take_body(), self.opts.max_bytes).await?;
        if buf.is_empty() {
            return Ok(json!({}));
        }
        if !self.header("content-type").is_some_and(is_json_type) {
            return Err(ApiError::new(
                ErrorCode::UnsupportedMedia,
                "send application/json",
            ));
        }
        serde_json::from_slice(&buf).map_err(|_| ApiError::bad_request("body is not valid JSON"))
    }
}
