//! The HTTP layer: the Host, Origin and content-type matrix, the optional `requireToken` mode, the error shape, body limits, page substitution, file routes, the db shim routes, the admin routes, single instance and shutdown. Ported from the Node `core.http.test.mjs`. K2, K4: the refusal matrix, the token mode and the file routes; the pure rules are in `http_rules.rs`.
// test-tag: standard

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::{now_ms, parse_ms};
use argus_core::{Coll, DocChange, Paths};
use argus_server::{ServeError, ServeOptions, serve};
use argus_store::rusqlite::params;
use argus_testkit::{Reply, Request, TestOptions, TestServer};
use serde_json::{Value, json};

use crate::wait::until;

const PNG: [u8; 16] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
];
const JSON: (&str, &str) = ("Content-Type", "application/json");
const ITEM: &str = "/api/p/alpha/db/items/I-1";
const BODY_LIMIT: usize = 1024 * 1024;

type Headers<'a> = Vec<(&'a str, &'a str)>;
type Case<'a> = (&'a str, &'a [(&'a str, &'a str)], u16);

fn seed_files(home: &Path) {
    for key in ["alpha", "beta"] {
        fs::create_dir_all(home.join(key)).expect("board folder");
        fs::write(home.join(key).join("board.json"), "{}").expect("board file");
    }
    fs::create_dir_all(home.join("empty")).expect("empty folder");
    let proofs = home.join("alpha").join("proofs");
    fs::create_dir_all(&proofs).expect("proofs folder");
    fs::write(proofs.join("T-1-1.png"), PNG).expect("proof");
    fs::write(proofs.join("notes.txt"), "x").expect("notes");
    fs::write(home.join("alpha").join("secret.png"), PNG).expect("secret");
}

async fn started(sync: bool, config: Value) -> TestServer {
    let options = TestOptions {
        sync,
        config,
        ..TestOptions::default()
    };
    let t = TestServer::start(options).await;
    seed_files(&t.coord_home);
    t
}

async fn plain() -> TestServer {
    started(false, json!({})).await
}

async fn token_server() -> TestServer {
    started(false, json!({ "requireToken": true })).await
}

async fn synced() -> TestServer {
    let options = TestOptions {
        seed: true,
        ..TestOptions::default()
    };
    TestServer::start(options).await
}

fn web_root(t: &TestServer) -> PathBuf {
    t.dashboard_home.join("no-web-build")
}

fn put_web(t: &TestServer, rel: &str, bytes: &[u8]) {
    let file = web_root(t).join(rel);
    fs::create_dir_all(file.parent().expect("parent")).expect("web folder");
    fs::write(file, bytes).expect("web file");
}

fn req(method: &str, path: &str, headers: &[(&str, &str)]) -> Request {
    headers
        .iter()
        .fold(Request::new(method, path), |r, (k, v)| r.header(k, v))
}

async fn send(t: &TestServer, method: &str, path: &str, headers: &[(&str, &str)]) -> Reply {
    t.send(req(method, path, headers)).await
}

async fn get_status(t: &TestServer, path: &str, headers: &[(&str, &str)]) -> u16 {
    send(t, "GET", path, headers).await.status
}

fn assert_refused(r: &Reply, status: u16, code: &str, why: &str) {
    assert_eq!((r.status, r.code().as_str()), (status, code), "{why}");
}

fn json_with<'a>(extra: &[(&'a str, &'a str)]) -> Headers<'a> {
    let mut all = vec![JSON];
    all.extend_from_slice(extra);
    all
}

async fn put_item(t: &TestServer, headers: &[(&str, &str)]) -> Reply {
    t.send(req("PUT", ITEM, headers).body(r#"{"data":{}}"#))
        .await
}

async fn expect_puts(t: &TestServer, status: u16, code: &str, cases: &[(&str, Headers<'_>)]) {
    for (why, headers) in cases {
        assert_refused(&put_item(t, headers).await, status, code, why);
    }
}

async fn post_json(t: &TestServer, path: &str, headers: &[(&str, &str)], bearer: bool) -> Reply {
    let r = req("POST", path, headers).body("{}");
    t.send(if bearer { t.bearer(r) } else { r }).await
}

fn enc(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn assert_home(reply: &Reply, next: &str) {
    assert_eq!(reply.header("location"), Some("/"), "next {next:?}");
}

const FOREIGN_NEXT: [&str; 5] = [
    "//evil.test/x",
    "/\\evil.test",
    "http://evil.test",
    "evil",
    "",
];

#[tokio::test(flavor = "multi_thread")]
async fn loopback_gets_need_no_credential_a_wrong_bearer_is_401() {
    let t = plain().await;
    let none = t.get("/api/health").await;
    assert_eq!(none.status, 200);
    assert_eq!(none.json()["port"], json!(t.port));
    let valid = t.send(t.bearer(Request::new("GET", "/api/health"))).await;
    assert_eq!(valid.status, 200, "a valid bearer is fine");
    let zeros = format!("Bearer {}", "0".repeat(64));
    for bad in ["Bearer nope", zeros.as_str(), "Basic abc"] {
        let r = send(&t, "GET", "/api/health", &[("Authorization", bad)]).await;
        assert_refused(&r, 401, "unauthorized", bad);
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn health_body_has_the_contract_fields() {
    let t = plain().await;
    let h = t.get("/api/health").await.json();
    let fields = [
        "ok",
        "version",
        "instance",
        "startedAt",
        "port",
        "schema",
        "scopes",
        "watcher",
        "runs",
        "sse",
        "impl",
        "shadow",
    ];
    for k in fields {
        assert!(h.get(k).is_some(), "{k}");
    }
    assert_eq!(h["schema"], json!(t.svc.db.user_version().expect("schema")));
    let runs = h["runs"].as_object().expect("runs object");
    assert_eq!(
        runs.keys().map(String::as_str).collect::<Vec<_>>(),
        ["running", "queued"]
    );
    assert!(h["sse"]["clients"].is_number());
    assert_eq!(
        (h["impl"].as_str(), h["shadow"].as_bool()),
        (Some("argus"), Some(false))
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn version_body_names_the_build_only_when_there_is_one() {
    let t = plain().await;
    let v = t.get("/api/version").await.json();
    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    assert!(v["rust"].is_string());
    assert!(v.get("node").is_none());
    assert!(v["pages"]["board"].is_null() && v["pages"]["portfolio"].is_null());
    put_web(&t, "index.html", b"<title>x</title>");
    let built = t.get("/api/version").await.json();
    let id = built["pages"]["board"].as_str().expect("build id");
    assert_eq!(id.len(), 12);
    assert_eq!(built["pages"]["portfolio"], id);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn host_check_refuses_other_hosts_with_or_without_a_bearer() {
    let t = plain().await;
    let port = t.port;
    let cases = [
        (format!("localhost:{port}"), 200),
        (format!("127.0.0.1:{port}"), 200),
        ("127.0.0.1".to_owned(), 421),
        (format!("127.0.0.1:{}", port + 1), 421),
        ("evil.test".to_owned(), 421),
    ];
    for bearer in [false, true] {
        for (host, want) in &cases {
            let r = req("GET", "/api/health", &[("Host", host)]);
            let r = t.send(if bearer { t.bearer(r) } else { r }).await;
            assert_eq!(r.status, *want, "{host} bearer {bearer}");
            if *want == 421 {
                assert_eq!(r.code(), "bad_host");
            }
        }
    }
    assert_eq!(
        get_status(&t, "/", &[("Host", "evil.test")]).await,
        421,
        "pages too"
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn errors_have_one_shape_404_405_and_bad_path() {
    let t = plain().await;
    let nf = t.get("/api/nothing-here").await;
    assert_eq!(nf.status, 404);
    let body = nf.json();
    let keys: Vec<&String> = body.as_object().expect("object").keys().collect();
    assert_eq!(keys, ["error"]);
    assert_eq!(body["error"]["code"], "not_found");
    assert!(body["error"]["message"].is_string());
    let na = send(&t, "DELETE", "/api/health", &[]).await;
    assert_refused(&na, 405, "method_not_allowed", "DELETE");
    assert!(na.header("allow").unwrap_or_default().contains("GET"));
    assert_eq!(t.get("/api/p/alpha/db/%E0%A4%A").await.status, 400);
    let options = send(&t, "OPTIONS", "/api/health", &[]).await;
    assert_eq!(options.status, 405, "no CORS preflight answers");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn writes_the_checks_accept_reach_the_route() {
    let t = plain().await;
    let o = t.url.as_str();
    let cases = [
        ("a CLI: no Origin, no metadata", json_with(&[])),
        (
            "a same-origin page",
            json_with(&[("Origin", o), ("Sec-Fetch-Site", "same-origin")]),
        ),
        (
            "same-site",
            json_with(&[("Origin", o), ("Sec-Fetch-Site", "same-site")]),
        ),
        (
            "none, typed in the address bar",
            json_with(&[("Sec-Fetch-Site", "none")]),
        ),
        (
            "a charset parameter",
            vec![("Content-Type", "application/json; charset=utf-8")],
        ),
        (
            "a +json type",
            vec![("Content-Type", "application/vnd.api+json")],
        ),
    ];
    expect_puts(&t, 403, "read_only", &cases).await;
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn writes_from_another_origin_are_forbidden() {
    let t = plain().await;
    let o = t.url.as_str();
    let localhost = format!("http://localhost:{}", t.port);
    let cases = [
        (
            "foreign Origin",
            json_with(&[("Origin", "http://evil.test")]),
        ),
        (
            "the other spelling of this host",
            json_with(&[("Origin", localhost.as_str())]),
        ),
        ("opaque Origin", json_with(&[("Origin", "null")])),
        (
            "cross-site metadata",
            json_with(&[("Origin", o), ("Sec-Fetch-Site", "cross-site")]),
        ),
        (
            "cross-site without Origin",
            json_with(&[("Sec-Fetch-Site", "cross-site")]),
        ),
    ];
    expect_puts(&t, 403, "forbidden", &cases).await;
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn writes_with_forms_and_text_plain_are_unsupported_media() {
    let t = plain().await;
    let types = [
        "application/x-www-form-urlencoded",
        "multipart/form-data; boundary=x",
        "text/plain",
        "text/plain;charset=UTF-8",
        "application/octet-stream",
    ];
    for kind in types {
        let r = put_item(&t, &[("Content-Type", kind)]).await;
        assert_refused(&r, 415, "unsupported_media", kind);
    }
    assert_eq!(put_item(&t, &[]).await.status, 415, "no Content-Type");
    let judged = put_item(
        &t,
        &[
            ("Content-Type", "text/plain"),
            ("Origin", "http://evil.test"),
        ],
    )
    .await;
    assert_eq!(judged.status, 403, "the origin is judged first");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn gets_are_not_origin_checked_and_carry_no_cors_headers() {
    let t = plain().await;
    let foreign = [
        ("Sec-Fetch-Site", "cross-site"),
        ("Origin", "http://evil.test"),
    ];
    let ok = send(&t, "GET", "/api/health", &foreign).await;
    assert_eq!(ok.status, 200);
    assert_eq!(ok.header("access-control-allow-origin"), None);
    let stream =
        Request::new("GET", "/api/p/alpha/stream?c=items").header("Sec-Fetch-Site", "cross-site");
    let sse = t.sse_with(stream).await;
    assert_eq!(sse.status, 200);
    sse.close();
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn shutdown_rotate_and_code_routes_refuse_a_web_page() {
    let t = plain().await;
    let page = [
        JSON,
        ("Origin", t.url.as_str()),
        ("Sec-Fetch-Site", "same-origin"),
    ];
    let evil = [JSON, ("Origin", "http://evil.test")];
    let cases: [Case; 5] = [
        ("/api/auth/code", &page, 200),
        ("/api/auth/code", &evil, 403),
        ("/api/admin/shutdown", &evil, 403),
        (
            "/api/admin/shutdown",
            &[("Content-Type", "text/plain")],
            415,
        ),
        (
            "/api/admin/token/rotate",
            &[JSON, ("Sec-Fetch-Site", "cross-site")],
            403,
        ),
    ];
    for (path, headers, want) in cases {
        let r = post_json(&t, path, headers, false).await;
        assert_eq!(r.status, want, "{path} {headers:?}");
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_in_the_default_mode_redirects_to_next_with_no_cookie() {
    let t = plain().await;
    let r = t.get("/auth?code=whatever&next=/p/alpha/").await;
    assert_eq!((r.status, r.header("location")), (302, Some("/p/alpha/")));
    assert_eq!(r.header("set-cookie"), None);
    assert_eq!(t.get("/auth").await.header("location"), Some("/"));
    for next in FOREIGN_NEXT {
        let r = t.get(&format!("/auth?next={}", enc(next))).await;
        assert_home(&r, next);
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn token_mode_refuses_requests_without_a_credential() {
    let t = token_server().await;
    let cookie = format!("coord_dash={}", t.token);
    assert_eq!(get_status(&t, "/api/health", &[]).await, 401);
    let zeros = format!("Bearer {}", "0".repeat(64));
    for bad in ["Bearer nope", zeros.as_str()] {
        assert_eq!(
            get_status(&t, "/api/health", &[("Authorization", bad)]).await,
            401
        );
    }
    let bearer = t.send(t.bearer(Request::new("GET", "/api/health"))).await;
    assert_eq!(bearer.status, 200, "bearer");
    assert_eq!(
        get_status(&t, "/api/health", &[("Cookie", &cookie)]).await,
        200,
        "cookie"
    );
    let page = t.get("/").await;
    assert_eq!(page.status, 401);
    assert!(
        page.header("content-type")
            .unwrap_or_default()
            .contains("text/html")
    );
    assert!(
        page.text().contains("dashboard open"),
        "a 401 page says how to sign in"
    );
    for path in [
        "/p/alpha/",
        "/static/app.js",
        "/files/alpha/proofs/T-1-1.png",
    ] {
        assert_eq!(t.get(path).await.status, 401, "{path}");
    }
    assert_eq!(get_status(&t, "/", &[("Cookie", &cookie)]).await, 200);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn token_mode_bearer_only_routes_refuse_a_cookie() {
    let t = token_server().await;
    let cookie = format!("coord_dash={}", t.token);
    let headers = [
        JSON,
        ("Cookie", cookie.as_str()),
        ("Origin", t.url.as_str()),
    ];
    for path in ["/api/auth/code", "/api/admin/shutdown"] {
        assert_eq!(
            post_json(&t, path, &headers, false).await.status,
            401,
            "{path}"
        );
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn token_mode_cookie_writes_need_a_matching_origin() {
    let t = token_server().await;
    let cookie = format!("coord_dash={}", t.token);
    let bearer = format!("Bearer {}", t.token);
    let (c, o) = (cookie.as_str(), t.url.as_str());
    let evil = "http://evil.test";
    let forbidden = [
        ("cookie, no Origin", json_with(&[("Cookie", c)])),
        (
            "cookie, foreign Origin",
            json_with(&[("Cookie", c), ("Origin", evil)]),
        ),
        (
            "cookie, cross-site",
            json_with(&[
                ("Cookie", c),
                ("Origin", o),
                ("Sec-Fetch-Site", "cross-site"),
            ]),
        ),
        (
            "bearer, foreign Origin",
            json_with(&[("Authorization", &bearer), ("Origin", evil)]),
        ),
    ];
    expect_puts(&t, 403, "forbidden", &forbidden).await;
    let reaches = [
        (
            "cookie with the right Origin",
            json_with(&[("Cookie", c), ("Origin", o)]),
        ),
        ("bearer", json_with(&[("Authorization", &bearer)])),
    ];
    expect_puts(&t, 403, "read_only", &reaches).await;
    t.stop(false).await;
}

async fn issue_code(t: &TestServer) -> Value {
    post_json(t, "/api/auth/code", &[JSON], true).await.json()
}

#[tokio::test(flavor = "multi_thread")]
async fn token_mode_login_code_is_single_use_and_sets_the_cookie() {
    let t = token_server().await;
    let issued = issue_code(&t).await;
    let code = issued["code"].as_str().expect("code");
    assert!(
        code.len() == 32
            && code
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    let left = parse_ms(issued["expiresAt"].as_str().expect("expiry")).expect("a date") - now_ms();
    assert!(left > 0 && left <= 60_000, "{left}");
    let r = t.get(&format!("/auth?code={code}&next=/p/alpha/")).await;
    assert_eq!((r.status, r.header("location")), (302, Some("/p/alpha/")));
    let cookie = r.header("set-cookie").expect("cookie");
    assert!(cookie.starts_with(&format!("coord_dash={};", t.token)));
    for attr in ["HttpOnly", "SameSite=Strict", "Path=/", "Max-Age=31536000"] {
        assert!(cookie.contains(attr), "{attr}");
    }
    for path in [
        format!("/auth?code={code}"),
        "/auth?code=bogus".to_owned(),
        "/auth".to_owned(),
    ] {
        assert_eq!(t.get(&path).await.status, 401, "{path}");
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn token_mode_streams_need_the_token() {
    let t = token_server().await;
    assert_eq!(t.sse("/api/p/alpha/stream?c=items").await.status, 401);
    let cookie = format!("coord_dash={}", t.token);
    let stream = Request::new("GET", "/api/p/alpha/stream?c=items").header("Cookie", &cookie);
    let sse = t.sse_with(stream).await;
    assert_eq!(sse.status, 200);
    sse.close();
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn portfolio_page_has_its_scope_and_the_security_headers() {
    let t = plain().await;
    let r = t.get("/").await;
    assert_eq!(r.status, 200);
    let html = r.text();
    assert!(html.contains("<meta name=\"coord-scope\" content=\"_portfolio\">"));
    assert!(html.contains("<title>Argus</title>"));
    for placeholder in [
        "__KEY__",
        "__PROJECT__",
        "__BASE__",
        "__DASHBOARD_VERSION__",
    ] {
        assert!(!html.contains(placeholder), "{placeholder}");
    }
    let csp = r.header("content-security-policy").expect("csp");
    for part in [
        "default-src 'self'",
        "script-src 'self';",
        "frame-ancestors 'none'",
    ] {
        assert!(csp.contains(part), "{part}");
    }
    assert!(!csp.contains("cdnjs"), "no CDN source");
    assert_eq!(r.header("x-content-type-options"), Some("nosniff"));
    assert_eq!(r.header("referrer-policy"), Some("no-referrer"));
    assert_eq!(r.header("cache-control"), Some("no-store"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn board_page_is_served_for_known_keys_only() {
    let t = plain().await;
    let ok = t.get("/p/alpha/").await;
    assert_eq!(ok.status, 200);
    assert!(
        ok.text()
            .contains("<meta name=\"coord-scope\" content=\"alpha\">")
    );
    let redirect = t.get("/p/alpha").await;
    assert_eq!(
        (redirect.status, redirect.header("location")),
        (302, Some("/p/alpha/"))
    );
    let refused = [
        ("/p/nope/", "unknown key"),
        ("/p/empty/", "a folder without board.json is no board"),
        ("/p/..%2f/", "a dot segment"),
        ("/p/_portfolio/", "the portfolio has its own page"),
    ];
    for (path, why) in refused {
        assert_eq!(t.get(path).await.status, 404, "{why}");
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn static_serves_files_of_the_web_build_only() {
    let t = plain().await;
    put_web(&t, "a.js", b"console.log(1)");
    put_web(&t, "sub/b.css", b"a{}");
    put_web(&t, "x.exe", b"MZ");
    let a = t.get("/static/a.js").await;
    assert_eq!(a.status, 200);
    assert!(
        a.header("content-type")
            .unwrap_or_default()
            .starts_with("text/javascript")
    );
    assert_eq!(a.header("cache-control"), Some("no-cache"));
    assert_eq!(a.text(), "console.log(1)");
    assert_eq!(t.get("/static/sub/b.css").await.status, 200);
    let bad = [
        "/static/nope.js",
        "/static/x.exe",
        "/static/..%2fpackage.json",
        "/static/%2e%2e/package.json",
        "/static/sub",
        "/static/..%5cpackage.json",
        "/static/",
    ];
    for path in bad {
        assert_eq!(t.get(path).await.status, 404, "{path}");
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn proofs_are_served_as_images_by_strict_name_only() {
    let t = plain().await;
    let ok = t.get("/files/alpha/proofs/T-1-1.png").await;
    assert_eq!(ok.status, 200);
    assert_eq!(ok.header("content-type"), Some("image/png"));
    assert_eq!(ok.body, PNG);
    assert!(
        ok.header("content-security-policy")
            .unwrap_or_default()
            .contains("default-src 'none'")
    );
    let bad = [
        "/files/alpha/proofs/nope.png",
        "/files/alpha/proofs/notes.txt",
        "/files/alpha/proofs/..%2fsecret.png",
        "/files/alpha/proofs/%2e%2e",
        "/files/alpha/proofs/.hidden.png",
        "/files/alpha/proofs/a%5cb.png",
        "/files/nokey/proofs/T-1-1.png",
        "/files/ALPHA/proofs/T-1-1.png",
        "/files/alpha/proofs/T-1-1.png%00",
    ];
    for path in bad {
        assert_eq!(t.get(path).await.status, 404, "{path}");
    }
    t.stop(false).await;
}

fn add_attachment(t: &TestServer, id: &str, scope: &str, path: &Path, mime: &str) {
    let sql = "INSERT INTO attachments(id, scope, owner_kind, owner_id, n, path, mime, bytes, sha256, caption, created_by, created_at) VALUES(?, ?, 'run', 'R-1', NULL, ?, ?, 16, 'x', NULL, 'human', '2026-01-01T00:00:00.000Z')";
    let path = path.display().to_string();
    t.svc
        .db
        .with_conn(|c| c.execute(sql, params![id, scope, path, mime]))
        .expect("attachment row");
}

#[tokio::test(flavor = "multi_thread")]
async fn attachments_are_served_from_the_attachments_folder() {
    let t = plain().await;
    let dir = t.svc.paths.attachments.join("alpha");
    fs::create_dir_all(&dir).expect("attachments folder");
    fs::write(dir.join("one.png"), PNG).expect("attachment");
    fs::write(t.coord_home.join("outside.png"), PNG).expect("outside file");
    add_attachment(&t, "att1", "alpha", &dir.join("one.png"), "image/png");
    add_attachment(&t, "att2", "alpha", Path::new("alpha/one.png"), "image/png");
    add_attachment(
        &t,
        "att3",
        "alpha",
        &t.coord_home.join("outside.png"),
        "image/png",
    );
    add_attachment(&t, "att4", "beta", &dir.join("one.png"), "image/png");
    add_attachment(&t, "att5", "alpha", &dir.join("one.png"), "text/html");
    for id in ["att1", "att2"] {
        let r = t.get(&format!("/files/alpha/a/{id}")).await;
        assert_eq!(
            (r.status, r.header("content-type")),
            (200, Some("image/png")),
            "{id}"
        );
    }
    for (id, why) in [
        ("att3", "outside the allowed folders"),
        ("att4", "another scope"),
        ("nope", "unknown"),
    ] {
        assert_eq!(
            t.get(&format!("/files/alpha/a/{id}")).await.status,
            404,
            "{why}"
        );
    }
    let odd = t.get("/files/alpha/a/att5").await;
    assert_eq!(
        odd.header("content-type"),
        Some("application/octet-stream"),
        "non-image types never render"
    );
    assert!(
        odd.header("content-disposition")
            .unwrap_or_default()
            .contains("attachment")
    );
    t.stop(false).await;
}

async fn post_batch(t: &TestServer, kind: &str, body: String) -> Reply {
    let r = Request::new("POST", "/api/p/alpha/db/batch").header("Content-Type", kind);
    t.send(r.body(body)).await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_body_over_the_limit_is_413() {
    let t = plain().await;
    let pad = "x".repeat(BODY_LIMIT + 10);
    let big = post_batch(
        &t,
        "application/json",
        json!({ "ops": [], "pad": pad }).to_string(),
    )
    .await;
    assert_refused(&big, 413, "too_large", "over the limit");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_body_of_exactly_the_limit_is_read() {
    let t = plain().await;
    let bare = json!({ "ops": [], "pad": "" }).to_string().len();
    let body = json!({ "ops": [], "pad": "x".repeat(BODY_LIMIT - bare) }).to_string();
    assert_eq!(body.len(), BODY_LIMIT);
    let r = post_batch(&t, "application/json", body).await;
    assert_refused(
        &r,
        400,
        "bad_request",
        "read, then refused for its empty ops",
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_body_must_be_json_and_valid() {
    let t = plain().await;
    let text = post_batch(&t, "text/plain", r#"{"ops":[]}"#.to_owned()).await;
    assert_refused(&text, 415, "unsupported_media", "text/plain");
    let bad = post_batch(&t, "application/json", "{nope".to_owned()).await;
    assert_eq!(bad.status, 400);
    let empty = post_batch(&t, "application/json", String::new()).await;
    assert_eq!(
        empty.status, 400,
        "an empty body is an empty object, which has no ops"
    );
    assert!(
        empty.json()["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("ops")
    );
    t.stop(false).await;
}

fn seed_docs(t: &TestServer) {
    let docs = [
        DocChange::set(
            Coll::Items,
            "I-1",
            json!({ "id": "I-1", "status": "open", "n": 2 }),
        ),
        DocChange::set(
            Coll::Items,
            "I-2",
            json!({ "id": "I-2", "status": "done", "n": 1 }),
        ),
        DocChange::set(Coll::Meta, "board", json!({ "project": "alpha" })),
    ];
    t.svc.db.docs().apply("alpha", &docs).expect("apply docs");
}

fn ids(v: &Value) -> Vec<&str> {
    let docs = v["docs"].as_array().into_iter().flatten();
    docs.filter_map(|d| d["id"].as_str()).collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn db_routes_list_and_get_docs() {
    let t = plain().await;
    seed_docs(&t);
    let list = t.get("/api/p/alpha/db/items").await.json();
    assert!(list["seq"].is_number());
    assert_eq!(ids(&list), ["I-1", "I-2"]);
    assert_eq!(list["docs"][0]["data"]["status"], "open");
    assert_eq!(
        ids(&t.get("/api/p/alpha/db/tests").await.json()),
        Vec::<&str>::new()
    );
    let one = t.get("/api/p/alpha/db/items/I-1").await.json();
    assert_eq!(
        (&one["id"], &one["exists"], &one["data"]["n"]),
        (&json!("I-1"), &json!(true), &json!(2))
    );
    let none = t.get("/api/p/alpha/db/items/I-99").await;
    assert_eq!(
        (
            none.status,
            none.json()["exists"].as_bool(),
            none.json()["data"].is_null()
        ),
        (200, Some(false), true)
    );
    assert_eq!(
        t.get("/api/p/alpha/db/meta/board").await.json()["data"]["project"],
        "alpha"
    );
    assert_eq!(
        ids(&t.get("/api/p/_portfolio/db/cards").await.json()),
        Vec::<&str>::new()
    );
    t.stop(false).await;
}

async fn query_items(t: &TestServer, body: Value) -> Reply {
    t.write("POST", "/api/p/alpha/db/items/query", &body).await
}

async fn batch(t: &TestServer, ops: Value) -> Reply {
    t.write("POST", "/api/p/alpha/db/batch", &json!({ "ops": ops }))
        .await
}

#[tokio::test(flavor = "multi_thread")]
async fn db_routes_query_and_refuse_bad_requests() {
    let t = plain().await;
    seed_docs(&t);
    let open = query_items(&t, json!({ "where": [["status", "==", "open"]] })).await;
    assert_eq!(ids(&open.json()), ["I-1"]);
    let ordered = query_items(&t, json!({ "orderBy": ["n", "asc"], "limit": 1 })).await;
    assert_eq!(ids(&ordered.json()), ["I-2"]);
    let bad_op = query_items(&t, json!({ "where": [["n", "??", 1]] })).await;
    assert_refused(&bad_op, 400, "bad_request", "unknown operator");
    assert_eq!(t.get("/api/p/alpha/db/bogus").await.code(), "bad_request");
    assert_eq!(
        t.get("/api/p/Bad_Scope/db/items").await.code(),
        "bad_request"
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn db_routes_only_entries_are_writable_and_without_sync_the_post_is_unavailable() {
    let t = plain().await;
    let data = json!({ "data": { "kind": "followup", "text": "hi" } });
    let put = t
        .write("PUT", "/api/p/alpha/db/entries/O-m1x2-ab12", &data)
        .await;
    assert_refused(&put, 503, "unavailable", "no sync service");
    assert_eq!(put.header("retry-after"), None);
    for coll in ["items", "tests", "threads", "runs", "meta"] {
        let r = t
            .write(
                "PUT",
                &format!("/api/p/alpha/db/{coll}/X-1"),
                &json!({ "data": {} }),
            )
            .await;
        assert_refused(&r, 403, "read_only", coll);
    }
    let entry = "/api/p/alpha/db/entries/E-1";
    assert_eq!(
        t.write("PATCH", entry, &json!({ "data": {} })).await.code(),
        "read_only"
    );
    assert_eq!(
        t.write("DELETE", entry, &json!({})).await.code(),
        "read_only"
    );
    assert_eq!(
        t.write("PUT", entry, &json!({})).await.code(),
        "bad_request"
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn db_batch_is_validated_before_any_post() {
    let t = plain().await;
    let over: Vec<Value> = (0..51)
        .map(|i| json!({ "op": "set", "coll": "entries", "id": format!("O-x-{i}"), "data": {} }))
        .collect();
    assert_eq!(batch(&t, json!(over)).await.status, 400, "more than 50 ops");
    assert_eq!(batch(&t, json!([])).await.status, 400, "no ops");
    let zap = batch(&t, json!([{ "op": "zap", "coll": "entries", "id": "x" }]))
        .await
        .json();
    assert_eq!(zap["results"][0]["error"]["code"], "bad_request");
    let writes = json!([
        { "op": "update", "coll": "entries", "id": "E-1", "data": {} },
        { "op": "delete", "coll": "entries", "id": "E-1" },
    ]);
    let upd = batch(&t, writes).await.json();
    assert_eq!(
        upd["results"].as_array().map(Vec::len),
        Some(1),
        "stopped at the first error"
    );
    assert_eq!(upd["results"][0]["error"]["code"], "read_only");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn db_entries_are_posted_through_the_sync_service() {
    let t = synced().await;
    let data = json!({ "data": { "kind": "followup", "text": "hi" } });
    let r = t
        .write("PUT", "/api/p/alpha/db/entries/O-m1x2-ab12", &data)
        .await
        .json();
    assert_eq!(
        (&r["id"], &r["status"]),
        (&json!("O-m1x2-ab12"), &json!("applied"))
    );
    assert!(r["seq"].is_number());
    let stored = &t.board("alpha")["entries"]["O-m1x2-ab12"];
    assert_eq!(
        (&stored["text"], &stored["author"]),
        (&json!("hi"), &json!("human"))
    );
    let note = json!({ "data": { "kind": "followup", "project": "alpha", "text": "x" } });
    let portfolio = t
        .write("PUT", "/api/p/_portfolio/db/entries/O-m1x2-ab13", &note)
        .await;
    assert_eq!(
        (portfolio.status, portfolio.json()["id"].as_str()),
        (200, Some("O-m1x2-ab13"))
    );
    let mismatch = json!({ "data": { "id": "O-other-0000" } });
    let bad = t
        .write("PUT", "/api/p/alpha/db/entries/O-m1x2-ab12", &mismatch)
        .await;
    assert_refused(&bad, 400, "bad_request", "data.id differs from the path id");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn projects_lists_boards_found_on_disk_with_their_urls() {
    let t = plain().await;
    let p = t.get("/api/projects").await.json();
    let list = p.as_array().expect("an array");
    let mut keys: Vec<&str> = list.iter().filter_map(|x| x["key"].as_str()).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["alpha", "beta"]);
    let a = list.iter().find(|x| x["key"] == "alpha").expect("alpha");
    assert_eq!(
        (&a["url"], &a["hasBoard"], &a["running"]),
        (&json!("/p/alpha/"), &json!(true), &json!(0))
    );
    assert!(a.get("name").is_some() && a.get("root").is_some());
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn resync_is_delegated_to_the_sync_service() {
    let t = synced().await;
    let hard = t
        .write(
            "POST",
            "/api/admin/resync",
            &json!({ "project": "alpha", "hard": true }),
        )
        .await;
    assert_eq!(hard.status, 200);
    assert!(hard.json()["scopes"].is_array() && hard.json()["docs"].is_number());
    assert_eq!(
        t.write("POST", "/api/admin/resync", &json!({}))
            .await
            .status,
        200
    );
    let bad = t
        .write("POST", "/api/admin/resync", &json!({ "project": "../x" }))
        .await;
    assert_refused(&bad, 400, "bad_request", "an invalid project");
    t.stop(false).await;
}

async fn health_with(t: &TestServer, name: &str, value: &str) -> u16 {
    get_status(t, "/api/health", &[(name, value)]).await
}

#[tokio::test(flavor = "multi_thread")]
async fn token_rotation_signs_old_bearers_and_cookies_out() {
    let t = plain().await;
    let old = t.token.clone();
    let rotated = post_json(&t, "/api/admin/token/rotate", &[JSON], true)
        .await
        .json();
    let token = rotated["token"].as_str().expect("token").to_owned();
    assert!(token.len() == 64 && token != old);
    assert_eq!(
        health_with(&t, "Authorization", &format!("Bearer {old}")).await,
        401
    );
    assert_eq!(
        health_with(&t, "Authorization", &format!("Bearer {token}")).await,
        200
    );
    let (stale, fresh) = (format!("coord_dash={old}"), format!("coord_dash={token}"));
    assert_eq!(
        health_with(&t, "Cookie", &stale).await,
        200,
        "default mode: a cookie means nothing"
    );
    t.svc.config.write().expect("config lock").require_token = true;
    assert_eq!(health_with(&t, "Cookie", &stale).await, 401);
    assert_eq!(health_with(&t, "Cookie", &fresh).await, 200);
    t.stop(false).await;
}

fn serve_options(t: &TestServer) -> ServeOptions {
    let paths = Paths::new(&t.coord_home, Some(t.dashboard_home.clone()));
    let mut options = ServeOptions::new(paths);
    options.port = Some(0);
    options.cache.enabled = false;
    options
}

fn pid_record(t: &TestServer) -> Value {
    let text = fs::read_to_string(t.dashboard_home.join("server.pid")).expect("pid file");
    serde_json::from_str(&text).expect("pid json")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_second_server_on_the_same_home_is_refused() {
    let t = plain().await;
    let Err(err) = serve(serve_options(&t)).await else {
        panic!("a second server started");
    };
    assert!(
        matches!(err, ServeError::AlreadyRunning { port, .. } if port == t.port),
        "{err}"
    );
    let rec = pid_record(&t);
    assert_eq!(rec["pid"], json!(std::process::id()));
    assert_eq!(
        (&rec["port"], &rec["instance"]),
        (&json!(t.port), &json!(t.svc.instance))
    );
    t.stop(false).await;
}

async fn wait_until_gone(file: &Path) {
    until("the pid file to go", || async {
        (!file.exists()).then_some(())
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn shutdown_stops_the_server_and_frees_the_pid_file() {
    let t = plain().await;
    let r = post_json(&t, "/api/admin/shutdown", &[JSON], true).await;
    assert_eq!(r.status, 202);
    let pid_file = t.dashboard_home.join("server.pid");
    wait_until_gone(&pid_file).await;
    assert!(!pid_file.exists(), "pid file removed");
    let again = tokio::net::TcpStream::connect(("127.0.0.1", t.port)).await;
    assert!(again.is_err(), "the port no longer answers");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stale_pid_file_does_not_block_a_start() {
    let home = tempfile::tempdir().expect("temp dir");
    let stale = json!({ "pid": 2_147_483_000, "port": 1, "startedAt": "x", "instance": "dead" });
    fs::write(home.path().join("server.pid"), stale.to_string()).expect("stale pid file");
    let options = TestOptions {
        dashboard_home: Some(home.path().to_path_buf()),
        sync: false,
        ..TestOptions::default()
    };
    let t = TestServer::start(options).await;
    assert_eq!(t.get("/api/health").await.status, 200);
    assert_eq!(pid_record(&t)["instance"], json!(t.svc.instance));
    t.stop(false).await;
}
