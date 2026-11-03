//! K1, K2, K4: the pure rules of the HTTP layer with no server: token and cookie compare, login
//! codes, the bearer state and the loopback peer, the write refusal rules, the page template, the
//! router, the error codes and the per-scope queue.
// test-tag: critical

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use argus_core::clock::parse_ms;
use argus_core::{ApiError, ErrorCode};
use argus_server::security::{
    Bearer, LoginCodes, bearer_state, parse_cookies, peer_ok, safe_next, token_equals,
    write_refusal,
};
use argus_server::{Ctx, PageVars, RouteMatch, Router, render_page};
use argus_sync::ScopeQueue;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::Response;
use serde_json::json;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
    pairs
        .iter()
        .map(|(k, v)| {
            (
                HeaderName::from_bytes(k.as_bytes()).expect("header name"),
                HeaderValue::from_str(v).expect("header value"),
            )
        })
        .collect()
}

type Log = Arc<Mutex<Vec<&'static str>>>;

const VARS: PageVars<'static> = PageVars {
    name: "A<b>&\"",
    key: "alpha",
    base: "/p/alpha/",
    version: "9.9",
};

#[test]
fn token_compare_cookies_and_safe_next() {
    assert!(token_equals("abc", "abc"));
    assert!(
        !token_equals("abc", "abd") && !token_equals("abc", "abcd") && !token_equals("", "abc")
    );
    let cookies = parse_cookies("a=1; coord_dash=xyz; b=2=3");
    assert_eq!(
        cookies,
        map(&[("a", "1"), ("coord_dash", "xyz"), ("b", "2=3")])
    );
    assert_eq!(safe_next(Some("/p/alpha/?x=1")), "/p/alpha/?x=1");
    assert_eq!(safe_next(Some("/a\r\nSet-Cookie: x")), "/");
}

#[test]
fn login_codes_live_sixty_seconds_and_work_once() {
    let codes = LoginCodes::default();
    let t0 = 1_000_000;
    let (first, _) = codes.issue_at(t0);
    assert!(codes.redeem_at(&first, t0 + 59_000));
    assert!(!codes.redeem_at(&first, t0 + 59_000), "single use");
    let (second, expires) = codes.issue_at(t0 + 59_000);
    assert_eq!(parse_ms(&expires), Some(t0 + 59_000 + 60_000));
    assert!(
        !codes.redeem_at(&second, t0 + 59_000 + 60_001),
        "older than 60 s"
    );
    let (last_ms, _) = codes.issue_at(t0);
    assert!(codes.redeem_at(&last_ms, t0 + 59_999));
    let (edge, _) = codes.issue_at(t0);
    assert!(!codes.redeem_at(&edge, t0 + 60_000));
    assert!(!codes.redeem_at("nope", t0));
}

#[test]
fn bearer_state_and_peer_check() {
    let state = |v: &str| bearer_state(&headers(&[("authorization", v)]), "t");
    assert_eq!(bearer_state(&HeaderMap::new(), "t"), Bearer::None);
    assert_eq!(state("Bearer t"), Bearer::Ok);
    assert_eq!(state("Bearer u"), Bearer::Bad);
    assert_eq!(state("Token t"), Bearer::Bad);
    let ip = |s: &str| s.parse::<IpAddr>().expect("an ip");
    for local in ["127.0.0.1", "::1", "::ffff:127.0.0.1"] {
        assert!(peer_ok(ip(local), None), "{local}");
    }
    assert!(!peer_ok(ip("192.168.1.5"), None));
}

#[test]
fn write_refusal_rules() {
    let refusal = |kind: &str, extra: &[(&str, &str)], images: bool| {
        let all = [
            &[("host", "127.0.0.1:1"), ("content-type", kind)][..],
            extra,
        ]
        .concat();
        write_refusal(&headers(&all), images).map(|e| e.code.status())
    };
    let json = "application/json";
    assert_eq!(refusal(json, &[], false), None);
    assert_eq!(
        refusal(json, &[("origin", "http://127.0.0.1:1")], false),
        None
    );
    assert_eq!(
        refusal(json, &[("origin", "http://127.0.0.1:2")], false),
        Some(403)
    );
    assert_eq!(
        refusal(json, &[("sec-fetch-site", "cross-site")], false),
        Some(403)
    );
    assert_eq!(refusal("text/plain", &[], false), Some(415));
    assert_eq!(
        refusal("image/png", &[], false),
        Some(415),
        "images only where the route says so"
    );
    assert_eq!(refusal("image/png", &[], true), None);
}

#[test]
fn render_page_fills_placeholders_and_escapes_the_name() {
    let html = "<!doctype html><html><head><title>__PROJECT__</title></head><body><a href=\"__BASE__\">__KEY__ __DASHBOARD_VERSION__</a><script>1</script></body></html>";
    let out = render_page(html, &VARS);
    assert!(out.contains("<title>A&lt;b&gt;&amp;&quot;</title>"));
    assert!(out.contains("<head><meta name=\"coord-scope\" content=\"alpha\"><title>"));
    assert!(out.contains("<a href=\"/p/alpha/\">alpha 9.9</a>"));
}

#[test]
fn render_page_adds_the_scope_meta_only_when_missing() {
    let has =
        "<html><head><meta name=\"coord-scope\" content=\"__KEY__\"></head><body></body></html>";
    let out = render_page(has, &VARS);
    assert_eq!(out.matches("coord-scope").count(), 1);
    assert!(out.contains("content=\"alpha\""));
    let bare = render_page("<p>no head</p>", &VARS);
    assert_eq!(
        bare,
        "<meta name=\"coord-scope\" content=\"alpha\"><p>no head</p>"
    );
}

fn nothing(_ctx: Ctx) -> std::future::Ready<Result<Response, ApiError>> {
    std::future::ready(Err(ApiError::not_found("x")))
}

fn params_of(m: RouteMatch) -> HashMap<String, String> {
    match m {
        RouteMatch::Found(_, _, params) => params,
        _ => panic!("the route was not found"),
    }
}

#[test]
fn router_matches_params_wildcards_and_method_sets() {
    let mut r = Router::new();
    r.get("/a/:x/b/:y", nothing);
    r.post("/a/:x/b/:y", nothing);
    r.get("/s/*", nothing);
    r.get("/", nothing);
    let both = map(&[("x", "1"), ("y", "hello w")]);
    assert_eq!(params_of(r.find("GET", "/a/1/b/hello%20w")), both);
    assert_eq!(
        params_of(r.find("HEAD", "/a/1/b/2")),
        map(&[("x", "1"), ("y", "2")])
    );
    let RouteMatch::NotAllowed(mut allow) = r.find("PUT", "/a/1/b/2") else {
        panic!("PUT is not allowed");
    };
    allow.sort_unstable();
    assert_eq!(allow, ["GET", "POST"]);
    assert_eq!(
        params_of(r.find("GET", "/s/deep/path.js"))["*"],
        "deep/path.js"
    );
    assert!(matches!(r.find("GET", "/"), RouteMatch::Found(..)));
}

#[test]
fn router_trailing_slash_is_exact_and_a_bad_escape_is_a_bad_path() {
    let mut r = Router::new();
    r.get("/a/:x/b/:y", nothing);
    assert!(matches!(r.find("GET", "/a/1/b"), RouteMatch::NotFound));
    assert!(matches!(r.find("GET", "/a/1/b/2/"), RouteMatch::NotFound));
    assert!(matches!(r.find("GET", "/a/%E0/b/2"), RouteMatch::BadPath));
}

#[test]
fn errors_map_codes_to_statuses_and_hide_internals() {
    let e = ApiError::new(ErrorCode::Conflict, "m").with_details(json!({ "a": 1 }));
    let want = json!({ "error": { "code": "conflict", "message": "m", "details": { "a": 1 } } });
    assert_eq!((e.code.status(), e.body()), (409, want));
    let statuses = [
        (ErrorCode::BoardLocked, 503),
        (ErrorCode::DbBusy, 503),
        (ErrorCode::UnknownRun, 404),
        (ErrorCode::Internal, 500),
    ];
    for (code, status) in statuses {
        assert_eq!(code.status(), status, "{}", code.as_str());
    }
    assert_eq!(ApiError::internal().message, "internal error");
}

fn job(q: &ScopeQueue, scope: &'static str, log: &Log, what: &'static str) -> JoinHandle<()> {
    let (q, log) = (q.clone(), Arc::clone(log));
    tokio::spawn(async move {
        q.run(scope, async { log.lock().expect("log").push(what) })
            .await;
    })
}

fn seen(log: &Log) -> Vec<&'static str> {
    log.lock().expect("log").clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_scope_queue_serialises_a_scope_and_lets_scopes_run_side_by_side() {
    let q = ScopeQueue::new();
    let log: Log = Arc::default();
    let (open, gate) = oneshot::channel::<()>();
    let (held_tx, held) = oneshot::channel::<()>();
    let a1 = {
        let (q, log) = (q.clone(), Arc::clone(&log));
        tokio::spawn(async move {
            q.run("qa", async {
                log.lock().expect("log").push("a1 start");
                let _ = held_tx.send(());
                let _ = gate.await;
                log.lock().expect("log").push("a1 end");
            })
            .await;
        })
    };
    held.await.expect("a1 holds the scope");
    let a2 = job(&q, "qa", &log, "a2");
    job(&q, "qb", &log, "b1").await.expect("b1");
    assert_eq!(seen(&log), ["a1 start", "b1"], "qb ran while qa was held");
    open.send(()).expect("open the gate");
    a1.await.expect("a1");
    a2.await.expect("a2");
    assert_eq!(seen(&log), ["a1 start", "b1", "a1 end", "a2"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_scope_queue_keeps_running_after_a_failed_job() {
    let q = ScopeQueue::new();
    let failed: Result<(), &str> = q.run("qc", async { Err("first fails") }).await;
    assert_eq!(failed, Err("first fails"));
    assert_eq!(q.run("qc", async { "still runs" }).await, "still runs");
}
