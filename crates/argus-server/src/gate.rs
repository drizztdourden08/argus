//! Every request passes here before its route, in the Node order: Host, peer, route match (404,
//! 400, 405), a wrong bearer, then the token mode's credential and Origin rules or the default
//! mode's write rules, and finally the handler, whose error becomes the one error shape.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use axum::body::Body;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::request::Parts as RequestParts;
use axum::http::{HeaderMap, Method};
use axum::response::Response;
use serde_json::json;

use crate::body::discard;
use crate::ctx::Parts;
use crate::pathing::{normalize, query_pairs};
use crate::reply::{error_reply, html_page, info_page, plain_error};
use crate::router::{AuthMode, Handler, RouteMatch, Router};
use crate::routes::feed::feed_path;
use crate::security::{
    Bearer, Credential, bearer_state, credential, host_ok, looks_like_page, origin_ok, peer_ok,
    write_refusal,
};
use crate::{AuthKind, Ctx, RouteOpts, Services};

/// The router and the services, as the listener's state.
pub struct App {
    pub router: Router,
    pub svc: Arc<Services>,
}

fn is_page(method: &Method, path: &str) -> bool {
    let first = path.split('/').nth(1).unwrap_or_default();
    method == Method::GET && !["api", "mcp", "files", "static"].contains(&first)
}

fn fail(code: ErrorCode, message: &str) -> Response {
    plain_error(&ApiError::new(code, message), None)
}

/// The sign-in page of the token mode.
#[must_use]
pub fn login_page() -> String {
    info_page(
        "Not signed in",
        &[
            "This dashboard asks for a token (<code>requireToken</code> in its config.json). Run <code>dashboard open</code> (or <code>coord dashboard open</code>) to open it with a one-time sign-in.",
        ],
    )
}

fn token_mode(
    svc: &Services,
    h: &HeaderMap,
    req: (&Method, &str),
    mode: AuthMode,
) -> Result<AuthKind, Box<Response>> {
    let (method, path) = req;
    let cred = credential(h, &svc.token());
    let Some(cred) = cred.filter(|c| mode != AuthMode::Bearer || *c == Credential::Bearer) else {
        svc.log.warn(
            "unauthorised request",
            json!({ "req": format!("{method} {path}") }),
        );
        if is_page(method, path) {
            return Err(Box::new(html_page(401, login_page())));
        }
        let msg = if mode == AuthMode::Bearer {
            "a bearer token is required"
        } else {
            "sign in with `dashboard open`"
        };
        return Err(Box::new(fail(ErrorCode::Unauthorized, msg)));
    };
    let write = method != Method::GET && method != Method::HEAD;
    let stream = h
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("event-stream"))
        || path.ends_with("/stream")
        || path == "/api/mux";
    if (write || stream || path.starts_with("/mcp"))
        && !origin_ok(
            h,
            svc.port(),
            svc.doors(),
            write && cred == Credential::Cookie,
        )
    {
        return Err(Box::new(fail(
            ErrorCode::Forbidden,
            "cross-origin request refused",
        )));
    }
    Ok(if cred == Credential::Bearer {
        AuthKind::Cli
    } else {
        AuthKind::Page
    })
}

fn default_mode(
    svc: &Services,
    h: &HeaderMap,
    method: &Method,
    opts: RouteOpts,
    bearer: Bearer,
) -> Result<AuthKind, Box<Response>> {
    if method != Method::GET && method != Method::HEAD {
        if let Some(refusal) = write_refusal(h, opts.images) {
            svc.log
                .warn("write refused", json!({ "code": refusal.code.as_str() }));
            return Err(Box::new(plain_error(&refusal, None)));
        }
    }
    Ok(if bearer == Bearer::Ok || !looks_like_page(h) {
        AuthKind::Cli
    } else {
        AuthKind::Page
    })
}

fn authorize(
    svc: &Services,
    h: &HeaderMap,
    req: (&Method, &str),
    opts: RouteOpts,
) -> Result<Option<AuthKind>, Box<Response>> {
    if opts.auth == AuthMode::None {
        return Ok(None);
    }
    let bearer = bearer_state(h, &svc.token());
    if bearer == Bearer::Bad {
        svc.log.warn(
            "wrong bearer token",
            json!({ "req": format!("{} {}", req.0, req.1) }),
        );
        return Err(Box::new(fail(
            ErrorCode::Unauthorized,
            "the bearer token is wrong",
        )));
    }
    if svc.require_token() {
        token_mode(svc, h, req, opts.auth).map(Some)
    } else {
        default_mode(svc, h, req.0, opts, bearer).map(Some)
    }
}

type Found = (Handler, RouteOpts, HashMap<String, String>);

fn route(router: &Router, method: &Method, path: &str) -> Result<Found, Box<Response>> {
    let refusal = match router.find(method.as_str(), path) {
        RouteMatch::Found(h, o, p) => return Ok((h, o, p)),
        RouteMatch::NotFound => fail(
            ErrorCode::NotFound,
            &format!("no route for {method} {path}"),
        ),
        RouteMatch::BadPath => fail(ErrorCode::BadRequest, "malformed path"),
        RouteMatch::NotAllowed(allow) => {
            let e = ApiError::new(
                ErrorCode::MethodNotAllowed,
                format!("{method} is not allowed here"),
            );
            plain_error(&e, Some(&allow.join(", ")))
        }
    };
    Err(Box::new(refusal))
}

type Admitted = (
    Handler,
    RouteOpts,
    HashMap<String, String>,
    Option<AuthKind>,
);

/// The checks before the handler: Host, peer, route, credentials.
fn admit(
    app: &App,
    peer: SocketAddr,
    parts: &RequestParts,
    path: &str,
) -> Result<Admitted, Box<Response>> {
    let svc = &app.svc;
    if !host_ok(&parts.headers, svc.port(), svc.doors()) {
        svc.log.warn(
            "bad host refused",
            json!({ "req": format!("{} {path}", parts.method) }),
        );
        return Err(Box::new(fail(ErrorCode::BadHost, "unexpected Host header")));
    }
    if !peer_ok(peer.ip(), svc.door()) {
        return Err(Box::new(fail(
            ErrorCode::Forbidden,
            "only this computer may use the dashboard",
        )));
    }
    let (handler, opts, params) = route(&app.router, &parts.method, path)?;
    let auth = authorize(svc, &parts.headers, (&parts.method, path), opts)?;
    Ok((handler, opts, params, auth))
}

/// The listener's one handler. A refused request's body, like any body its handler left unread,
/// is discarded while the answer goes out.
pub async fn dispatch(
    State(app): State<Arc<App>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request<Body>,
) -> Response {
    let (parts, body) = req.into_parts();
    let svc = &app.svc;
    let path = normalize(parts.uri.path());
    let path = feed_path(&parts.headers, svc.port(), svc.doors(), path);
    let (handler, opts, params, auth) = match admit(&app, peer, &parts, &path) {
        Ok(v) => v,
        Err(res) => {
            discard(body);
            return *res;
        }
    };
    let query = query_pairs(parts.uri.query());
    let req = Parts {
        method: parts.method,
        path,
        headers: parts.headers,
        query,
        body,
    };
    let ctx = Ctx::new(Arc::clone(svc), req, params, opts, auth);
    match handler(ctx).await {
        Ok(res) => res,
        Err(e) => {
            if e.code == ErrorCode::Internal {
                svc.log.error("handler failed", json!({ "err": e.message }));
            }
            error_reply(&e)
        }
    }
}
