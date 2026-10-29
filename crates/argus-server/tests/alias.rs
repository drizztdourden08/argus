//! Board alias hosts: `<key>.localhost:<port>` and `<thread>.<key>.localhost:<port>` are loopback aliases like `127.0.0.1` and `localhost` (the Host check, the same-origin check of writes). `/` on one redirects to that board with the `?thread=` claim, an alias of no board gets the Argus overview page, and every other host stays 421. The reserved names `argus.localhost` (the overview, no redirect) and `overlord.localhost` (the Overlord's own board) are never routed to a project. Ported from the Node `http.alias.test.mjs`.

use std::fs;

use argus_server::security::{Alias, Doors, RESERVED_ALIASES, host_ok, origin_ok, parse_alias};
use argus_testkit::{Reply, Request, TestOptions, TestServer};
use axum::http::{HeaderMap, HeaderValue};
use serde_json::json;

const NODE_PORT: u16 = 47615;

fn write_board(t: &TestServer, key: &str) {
    let dir = t.coord_home.join(key);
    fs::create_dir_all(&dir).expect("board folder");
    fs::write(dir.join("board.json"), "{}").expect("board file");
}

async fn alias_server() -> TestServer {
    let options = TestOptions {
        sync: false,
        ..TestOptions::default()
    };
    let t = TestServer::start(options).await;
    for key in ["alpha", "beta-2", "argus", "overlord"] {
        write_board(&t, key);
    }
    t
}

async fn get(t: &TestServer, path: &str, host: &str) -> Reply {
    t.send(Request::new("GET", path).header("Host", host)).await
}

async fn put_item(t: &TestServer, host: &str, origin: Option<&str>) -> Reply {
    let req = Request::new("PUT", "/api/p/alpha/db/items/I-1").header("Host", host);
    let req = match origin {
        Some(o) => req.header("Origin", o),
        None => req,
    };
    t.send(req.json(&json!({ "data": {} }))).await
}

fn assert_alias(host: &str, key: &str, thread: Option<&str>, reserved: bool) {
    let want = Alias {
        key: key.to_owned(),
        thread: thread.map(str::to_owned),
        reserved,
    };
    assert_eq!(parse_alias(host, NODE_PORT), Some(want), "{host}");
}

fn pair(host: &str, origin: Option<&str>) -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert("host", HeaderValue::from_str(host).expect("host"));
    if let Some(o) = origin {
        h.insert("origin", HeaderValue::from_str(o).expect("origin"));
    }
    h
}

fn location(r: &Reply) -> Option<&str> {
    r.header("location")
}

async fn assert_bad_host(t: &TestServer, host: &str) {
    let r = get(t, "/api/health", host).await;
    assert_eq!((r.status, r.code().as_str()), (421, "bad_host"), "{host}");
}

#[test]
fn parse_alias_reads_key_and_thread_labels_and_the_port_only() {
    let p = NODE_PORT;
    assert_alias("alpha.localhost:47615", "alpha", None, false);
    assert_alias("ui-2.beta-2.localhost:47615", "beta-2", Some("ui-2"), false);
    assert_alias("ALPHA.Localhost:47615", "alpha", None, false);
    let bad = [
        "localhost:47615",
        "alpha.localhost",
        "alpha.localhost:47616",
        "alpha.localhost:47615.evil.test",
        "a.b.c.localhost:47615",
        "alpha.localhost.evil.test:47615",
        "-x.localhost:47615",
        "_x.localhost:47615",
        "a_b.localhost:47615",
        "x.a.localhost:47615 ",
        ".localhost:47615",
        "alpha.127.0.0.1:47615",
        "",
    ];
    for host in bad {
        assert_eq!(parse_alias(host, p), None, "{host:?}");
    }
    let long = format!("{}.localhost:1", "a".repeat(64));
    assert_eq!(
        parse_alias(&long, 1),
        None,
        "a label is 63 characters at most"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_alias_host_is_allowed_on_our_port_and_any_other_host_is_421() {
    let t = alias_server().await;
    let port = t.port;
    for host in [
        format!("alpha.localhost:{port}"),
        format!("ui.alpha.localhost:{port}"),
        format!("nosuch.localhost:{port}"),
    ] {
        assert_eq!(get(&t, "/api/health", &host).await.status, 200, "{host}");
    }
    let refused = [
        format!("alpha.localhost:{}", port + 1),
        "alpha.localhost".to_owned(),
        format!("alpha.localhost.evil.test:{port}"),
        format!("evil.test:{port}"),
        format!("a.b.c.localhost:{port}"),
        format!("alpha.evil.test:{port}"),
    ];
    for host in &refused {
        assert_bad_host(&t, host).await;
    }
    assert_eq!(
        get(&t, "/", &format!("evil.test:{port}")).await.status,
        421,
        "pages too"
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_root_of_an_alias_host_redirects_to_that_board_with_a_thread_claim() {
    let t = alias_server().await;
    let port = t.port;
    for path in ["/", "/index.html"] {
        let lead = get(&t, path, &format!("alpha.localhost:{port}")).await;
        assert_eq!(
            (lead.status, location(&lead)),
            (302, Some("/p/alpha/?thread=coordinator")),
            "{path}"
        );
        let thread = get(&t, path, &format!("ui.beta-2.localhost:{port}")).await;
        assert_eq!(
            (thread.status, location(&thread)),
            (302, Some("/p/beta-2/?thread=ui")),
            "{path}"
        );
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_thread_given_in_the_address_wins_over_the_label_and_the_default() {
    let t = alias_server().await;
    let port = t.port;
    let cases = [
        (
            format!("alpha.localhost:{port}"),
            "/?thread=ui",
            "/p/alpha/?thread=ui",
        ),
        (
            format!("alpha.localhost:{port}"),
            "/?thread=RotP%20lead",
            "/p/alpha/?thread=RotP%20lead",
        ),
        (
            format!("ui.alpha.localhost:{port}"),
            "/?thread=web",
            "/p/alpha/?thread=web",
        ),
        (
            format!("alpha.localhost:{port}"),
            "/?thread=",
            "/p/alpha/?thread=coordinator",
        ),
    ];
    for (host, path, target) in cases {
        let r = get(&t, path, &host).await;
        assert_eq!(
            (r.status, location(&r)),
            (302, Some(target)),
            "{host}{path}"
        );
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_board_opened_on_its_alias_host_holds_its_presence() {
    let options = TestOptions {
        seed: true,
        ..TestOptions::default()
    };
    let t = TestServer::start(options).await;
    let host = format!("alpha.localhost:{}", t.port);
    let path = "/api/p/alpha/presence?thread=ui&tab=t1";
    let s = t
        .sse_with(Request::new("GET", path).header("Host", &host))
        .await;
    assert_eq!(s.status, 200);
    s.next(Some("ready"), 5000).await.expect("ready");
    let open = t.get("/api/presence").await.json();
    let rows = open["open"].as_array().cloned().unwrap_or_default();
    assert!(
        rows.iter()
            .any(|r| r["project"] == "alpha" && r["thread"] == "ui"),
        "{open}"
    );
    s.close();
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_redirect_target_and_the_api_are_served_on_the_alias_host() {
    let t = alias_server().await;
    let port = t.port;
    fs::create_dir_all(t.dashboard_home.join("no-web-build")).expect("web root");
    fs::write(t.dashboard_home.join("no-web-build").join("app.js"), "1").expect("web file");
    let page = get(
        &t,
        "/p/alpha/?thread=ui",
        &format!("ui.alpha.localhost:{port}"),
    )
    .await;
    assert_eq!(page.status, 200);
    assert!(page.text().contains("coord-scope"));
    let host = format!("alpha.localhost:{port}");
    assert_eq!(get(&t, "/static/app.js", &host).await.status, 200);
    assert_eq!(get(&t, "/api/projects", &host).await.status, 200);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_alias_of_a_project_with_no_board_gets_the_overview_page() {
    let t = alias_server().await;
    let port = t.port;
    let none = get(&t, "/", &format!("nosuch.localhost:{port}")).await;
    assert_eq!((none.status, location(&none)), (200, None));
    assert_eq!(
        get(&t, "/", &format!("ui.nosuch.localhost:{port}"))
            .await
            .status,
        200
    );
    for host in [format!("127.0.0.1:{port}"), format!("localhost:{port}")] {
        assert_eq!(
            get(&t, "/", &host).await.status,
            200,
            "{host} is not redirected"
        );
    }
    assert_eq!(
        get(&t, "/index.html", &format!("127.0.0.1:{port}"))
            .await
            .status,
        200
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn writes_on_an_alias_host_accept_only_that_host_as_origin() {
    let t = alias_server().await;
    let port = t.port;
    let host = format!("ui.alpha.localhost:{port}");
    let own = format!("http://{host}");
    let shouting = format!("HTTP://{}", host.to_uppercase());
    for origin in [Some(own.as_str()), Some(shouting.as_str()), None] {
        let r = put_item(&t, &host, origin).await;
        assert_eq!(
            (r.status, r.code().as_str()),
            (403, "read_only"),
            "{origin:?}"
        );
    }
    let foreign = [
        format!("http://alpha.localhost:{port}"),
        format!("http://127.0.0.1:{port}"),
        format!("http://localhost:{port}"),
        "https://evil.test".to_owned(),
        format!("http://ui.alpha.localhost:{}", port + 1),
        format!("https://{host}"),
    ];
    for origin in &foreign {
        let r = put_item(&t, &host, Some(origin)).await;
        assert_eq!(
            (r.status, r.code().as_str()),
            (403, "forbidden"),
            "{origin}"
        );
    }
    let plain = format!("127.0.0.1:{port}");
    let cross = put_item(&t, &plain, Some(&format!("http://alpha.localhost:{port}"))).await;
    assert_eq!((cross.status, cross.code().as_str()), (403, "forbidden"));
    t.stop(false).await;
}

#[test]
fn an_alias_origin_counts_only_against_its_own_alias_host() {
    let ok = |host: &str, origin: &str, strict: bool| {
        origin_ok(&pair(host, Some(origin)), 4000, Doors::default(), strict)
    };
    let ui = "ui.alpha.localhost:4000";
    assert!(ok(ui, "http://ui.alpha.localhost:4000", false));
    assert!(ok(ui, "http://ui.alpha.localhost:4000", true));
    assert!(!ok(
        "127.0.0.1:4000",
        "http://ui.alpha.localhost:4000",
        false
    ));
    assert!(!ok(ui, "http://alpha.localhost:4000", false));
    assert!(!ok("evil.test", "http://evil.test", false));
    assert!(ok("127.0.0.1:4000", "http://127.0.0.1:4000", false));
    assert!(host_ok(
        &pair("alpha.localhost:4000", None),
        4000,
        Doors::default()
    ));
    assert!(!host_ok(
        &pair("alpha.localhost:4001", None),
        4000,
        Doors::default()
    ));
}

#[test]
fn reserved_aliases_parse_like_any_alias_but_carry_the_reserved_flag() {
    assert_eq!(RESERVED_ALIASES, ["argus", "overlord"]);
    assert_alias("argus.localhost:47615", "argus", None, true);
    assert_alias("OverLord.Localhost:47615", "overlord", None, true);
    assert_alias("ui.argus.localhost:47615", "argus", Some("ui"), true);
    assert_alias("alpha.localhost:47615", "alpha", None, false);
    assert_alias("argus-app.localhost:47615", "argus", None, false);
    assert_alias("ui.argus-app.localhost:47615", "argus", Some("ui"), false);
    assert_alias("argus.alpha.localhost:47615", "alpha", Some("argus"), false);
}

#[test]
fn reserved_names_are_reserved_on_our_port_only() {
    let p = NODE_PORT;
    let bad = [
        "argus.localhost",
        "argus.localhost:47616",
        "argus.localhost:47615.evil.test",
        "argus.localhost.evil.test:47615",
        "overlord.evil.test:47615",
        "overlord.localhost:4761",
        "xargus.localhost:1",
    ];
    for host in bad {
        assert!(parse_alias(host, p).is_none_or(|a| !a.reserved), "{host}");
    }
    assert_eq!(parse_alias("argus.localhost:47616", p), None);
    assert_eq!(parse_alias("overlord.localhost", p), None);
}

#[tokio::test(flavor = "multi_thread")]
async fn reserved_aliases_pass_the_host_check_on_our_port_and_nowhere_else() {
    let t = alias_server().await;
    let port = t.port;
    for host in [
        format!("argus.localhost:{port}"),
        format!("overlord.localhost:{port}"),
        format!("ARGUS.LOCALHOST:{port}"),
    ] {
        assert_eq!(get(&t, "/api/health", &host).await.status, 200, "{host}");
    }
    let refused = [
        format!("argus.localhost:{}", port + 1),
        "overlord.localhost".to_owned(),
        format!("overlord.localhost.evil.test:{port}"),
        format!("argus.evil.test:{port}"),
    ];
    for host in &refused {
        assert_bad_host(&t, host).await;
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn reserved_aliases_follow_the_same_origin_rule_as_every_alias() {
    let t = alias_server().await;
    let host = format!("overlord.localhost:{}", t.port);
    let own = put_item(&t, &host, Some(&format!("http://{host}"))).await;
    assert_eq!(own.code(), "read_only", "own Origin accepted by the checks");
    let other = put_item(
        &t,
        &host,
        Some(&format!("http://argus.localhost:{}", t.port)),
    )
    .await;
    assert_eq!(other.code(), "forbidden");
    t.stop(false).await;
}

#[test]
fn reserved_hosts_and_origins_in_the_unit_rules() {
    assert!(host_ok(
        &pair("argus.localhost:4000", None),
        4000,
        Doors::default()
    ));
    assert!(!host_ok(
        &pair("overlord.localhost:4001", None),
        4000,
        Doors::default()
    ));
    assert!(origin_ok(
        &pair(
            "overlord.localhost:4000",
            Some("http://overlord.localhost:4000")
        ),
        4000,
        Doors::default(),
        false
    ));
    assert!(!origin_ok(
        &pair(
            "overlord.localhost:4000",
            Some("http://argus.localhost:4000")
        ),
        4000,
        Doors::default(),
        false
    ));
    assert!(!origin_ok(
        &pair("127.0.0.1:4000", Some("http://argus.localhost:4000")),
        4000,
        Doors::default(),
        false
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn argus_localhost_serves_the_overview_at_the_root_even_when_a_project_is_called_argus() {
    let t = alias_server().await;
    let port = t.port;
    for path in ["/", "/index.html"] {
        let r = get(&t, path, &format!("argus.localhost:{port}")).await;
        assert_eq!((r.status, location(&r)), (200, None), "{path}");
        let html = r.text();
        assert!(html.contains("<title>Argus</title>"), "{path}");
        assert!(
            html.contains("<meta name=\"coord-scope\" content=\"_portfolio\">"),
            "{path}"
        );
    }
    assert_eq!(
        get(&t, "/", &format!("ui.argus.localhost:{port}"))
            .await
            .status,
        200,
        "a thread label in front is no route"
    );
    assert_eq!(
        get(&t, "/api/projects", &format!("argus.localhost:{port}"))
            .await
            .status,
        200
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn overlord_localhost_redirects_to_the_overlord_board_once_it_has_one() {
    let t = alias_server().await;
    let port = t.port;
    let host = format!("overlord.localhost:{port}");
    assert!(
        !t.coord_home.join("portfolio").join("board.json").exists(),
        "no Overlord board yet"
    );
    let before = get(&t, "/", &host).await;
    assert_eq!(
        (before.status, location(&before)),
        (200, None),
        "a project called overlord exists and is still not the target"
    );
    write_board(&t, "portfolio");
    for path in ["/", "/index.html"] {
        let r = get(&t, path, &host).await;
        assert_eq!(
            (r.status, location(&r)),
            (302, Some("/p/portfolio/?thread=coordinator")),
            "{path}"
        );
    }
    let shouting = get(&t, "/", &format!("OVERLORD.localhost:{port}")).await;
    assert_eq!(
        location(&shouting),
        Some("/p/portfolio/?thread=coordinator")
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn overlord_alias_edge_cases_and_the_board_page_name() {
    let t = alias_server().await;
    let port = t.port;
    write_board(&t, "portfolio");
    let thread = get(&t, "/", &format!("ui.overlord.localhost:{port}")).await;
    assert_eq!(thread.status, 200, "a thread label in front is no route");
    let board = get(
        &t,
        "/p/portfolio/?thread=coordinator",
        &format!("overlord.localhost:{port}"),
    )
    .await;
    assert_eq!(
        board.status, 200,
        "the redirect target is served on that host"
    );
    assert!(board.text().contains("coord-scope"));
    let plain = get(&t, "/", &format!("portfolio.localhost:{port}")).await;
    assert_eq!(
        location(&plain),
        Some("/p/portfolio/?thread=coordinator"),
        "the plain project alias keeps working"
    );
    assert_eq!(
        get(&t, "/", &format!("argus.localhost:{port}"))
            .await
            .status,
        200
    );
    let named = get(&t, "/p/portfolio/", &format!("127.0.0.1:{port}")).await;
    assert_eq!(named.status, 200);
    assert!(
        !named.text().contains("<title>Portfolio"),
        "the Overlord's project is not called Portfolio"
    );
    t.stop(false).await;
}
