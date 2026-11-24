//! Board alias hosts: `<key>.localhost:<port>` and `<thread>.<key>.localhost:<port>` are loopback aliases like `127.0.0.1` and `localhost` (the Host check, the same-origin check of writes). `/` on one redirects to that board with the `?thread=` claim, an alias of no board gets the Argus overview page, and every other host stays 421. The reserved names `argus.localhost` (the overview, no redirect) and `overlord.localhost` (the Overlord's own board) are never routed to a project. Ported from the Node `http.alias.test.mjs`. K2, K4: the redirect and overview answers and the host and origin refusals; the pure parse rules are in `alias_rules.rs`.
// test-tag: standard

use std::fs;

use argus_testkit::{Reply, Request, TestOptions, TestServer};
use serde_json::json;

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

fn location(r: &Reply) -> Option<&str> {
    r.header("location")
}

async fn assert_bad_host(t: &TestServer, host: &str) {
    let r = get(t, "/api/health", host).await;
    assert_eq!((r.status, r.code().as_str()), (421, "bad_host"), "{host}");
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
            (302, Some("/p/alpha/?thread=lead")),
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
            "/p/alpha/?thread=lead",
        ),
        (
            format!("coordinator.alpha.localhost:{port}"),
            "/",
            "/p/alpha/?thread=lead",
        ),
        (
            format!("alpha.localhost:{port}"),
            "/?thread=coordinator",
            "/p/alpha/?thread=lead",
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
            (302, Some("/p/portfolio/?thread=lead")),
            "{path}"
        );
    }
    let shouting = get(&t, "/", &format!("OVERLORD.localhost:{port}")).await;
    assert_eq!(location(&shouting), Some("/p/portfolio/?thread=lead"));
    t.stop(false).await;
}
