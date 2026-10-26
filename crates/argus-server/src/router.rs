//! Routes: a method and a path pattern with `:params` and one trailing `*`. Matching is the
//! Node router's: a path that matches with another method answers 405 with `Allow`, HEAD uses
//! the GET route, a param that does not decode is a 400.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use argus_core::ApiError;
use axum::response::Response;

use crate::Ctx;
use crate::pathing::decode;

/// What a route answers.
pub type Reply = Result<Response, ApiError>;

/// A route handler.
pub type Handler = Arc<dyn Fn(Ctx) -> Pin<Box<dyn Future<Output = Reply> + Send>> + Send + Sync>;

/// Which credential a route takes when the token is required (and always for `Bearer`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMode {
    /// A cookie or a bearer.
    #[default]
    Any,
    /// A bearer only.
    Bearer,
    /// No check at all (`/auth`).
    None,
}

/// Options of a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteOpts {
    /// The credential it takes.
    pub auth: AuthMode,
    /// The largest body.
    pub max_bytes: usize,
    /// A write may also send an `image/*` body.
    pub images: bool,
}

impl Default for RouteOpts {
    fn default() -> Self {
        RouteOpts {
            auth: AuthMode::Any,
            max_bytes: 1024 * 1024,
            images: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Lit(String),
    Param(String),
    Rest,
}

struct Route {
    method: &'static str,
    parts: Vec<Part>,
    handler: Handler,
    opts: RouteOpts,
}

/// The routes.
#[derive(Default)]
pub struct Router {
    routes: Vec<Route>,
}

/// The result of matching a request.
pub enum RouteMatch {
    /// A route, its options and its decoded params.
    Found(Handler, RouteOpts, HashMap<String, String>),
    /// The path matches other methods only.
    NotAllowed(Vec<&'static str>),
    /// A param did not decode.
    BadPath,
    /// No route.
    NotFound,
}

fn compile(pattern: &str) -> Vec<Part> {
    pattern
        .split('/')
        .map(|seg| match seg.strip_prefix(':') {
            Some(name) => Part::Param(name.to_owned()),
            None if seg == "*" => Part::Rest,
            None => Part::Lit(seg.to_owned()),
        })
        .collect()
}

fn capture(parts: &[Part], path: &str) -> Option<Vec<(String, String)>> {
    let segs: Vec<&str> = path.split('/').collect();
    let mut out = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        match part {
            Part::Rest => {
                out.push(("*".to_owned(), segs.get(i..)?.join("/")));
                return Some(out);
            }
            Part::Lit(l) if segs.get(i) == Some(&l.as_str()) => {}
            Part::Param(name) if segs.get(i).is_some_and(|s| !s.is_empty()) => {
                out.push((name.clone(), segs[i].to_owned()));
            }
            _ => return None,
        }
    }
    (segs.len() == parts.len()).then_some(out)
}

fn decoded(raw: Vec<(String, String)>) -> Option<HashMap<String, String>> {
    raw.into_iter()
        .map(|(k, v)| decode(&v).map(|d| (k, d)))
        .collect()
}

macro_rules! verb_route {
    ($name:ident, $method:literal, $doc:literal) => {
        #[doc = $doc]
        pub fn $name<F, Fut>(&mut self, pattern: &str, f: F)
        where
            F: Fn(Ctx) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Reply> + Send + 'static,
        {
            self.add($method, pattern, RouteOpts::default(), f);
        }
    };
}

impl Router {
    /// An empty router.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a route.
    pub fn add<F, Fut>(&mut self, method: &'static str, pattern: &str, opts: RouteOpts, f: F)
    where
        F: Fn(Ctx) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Reply> + Send + 'static,
    {
        let handler: Handler = Arc::new(move |ctx| Box::pin(f(ctx)));
        self.routes.push(Route {
            method,
            parts: compile(pattern),
            handler,
            opts,
        });
    }

    verb_route!(get, "GET", "Adds a GET route with the default options.");
    verb_route!(post, "POST", "Adds a POST route with the default options.");

    /// Matches a method and a normalized path.
    #[must_use]
    pub fn find(&self, method: &str, path: &str) -> RouteMatch {
        let want = if method == "HEAD" { "GET" } else { method };
        let mut allow: Vec<&'static str> = Vec::new();
        for r in &self.routes {
            let Some(raw) = capture(&r.parts, path) else {
                continue;
            };
            if r.method != want {
                if !allow.contains(&r.method) {
                    allow.push(r.method);
                }
                continue;
            }
            return match decoded(raw) {
                Some(params) => RouteMatch::Found(Arc::clone(&r.handler), r.opts, params),
                None => RouteMatch::BadPath,
            };
        }
        if allow.is_empty() {
            RouteMatch::NotFound
        } else {
            RouteMatch::NotAllowed(allow)
        }
    }
}
