//! The local names in the server: the DNS responder and the front door on a loopback address of
//! their own, off by default. Every socket is on port 0 (never 53, 80, 47615 or 47616), and the
//! address is not the default one, so a running Argus of the owner is never in the way.
//! K2, K4, S3: the responder, the front door and its host refusals on sockets of port 0; the pure host rules are in `local_names_rules.rs`.
// test-tag: standard

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use argus_dns::{TYPE_AAAA, ask, resolve};
use argus_server::LiveNames;
use argus_testkit::{Reply, Request, TestOptions, TestServer, request_at};
use serde_json::{Value, json};
use tokio::net::TcpListener;

const ADDRESS: Ipv4Addr = Ipv4Addr::new(127, 65, 82, 72);
const WAIT: Duration = Duration::from_secs(5);

fn names_config(extra: &Value) -> Value {
    let mut names = json!({ "enabled": true, "address": ADDRESS.to_string() });
    names.as_object_mut().expect("object").extend(
        extra
            .as_object()
            .expect("an object")
            .iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    json!({ "localNames": names })
}

async fn server(config: Value, door_port: u16) -> TestServer {
    let options = TestOptions {
        seed: true,
        sync: false,
        config,
        tweak: Some(Arc::new(move |o| {
            o.local_names.dns_port = 0;
            o.local_names.door_port = door_port;
        })),
        ..TestOptions::default()
    };
    TestServer::start(options).await
}

fn up(t: &TestServer) -> LiveNames {
    t.svc.names().expect("the names are up").clone()
}

async fn at_door(live: &LiveNames, path: &str, host: &str) -> Reply {
    let addr = SocketAddr::from((live.door.address, live.door.port));
    request_at(addr, &Request::new("GET", path).header("Host", host)).await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_names_are_off_unless_the_config_turns_them_on() {
    let t = server(json!({}), 0).await;
    assert!(t.svc.names().is_none());
    let health = t.get("/api/health").await.json();
    assert!(health.get("localNames").is_none());
    let host = format!("alpha.argus:{}", t.port);
    let r = t.send(Request::new("GET", "/").header("Host", &host)).await;
    assert_eq!(r.code(), "bad_host");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_refused_setting_leaves_the_names_off_and_the_server_up() {
    let bad = json!({ "localNames": { "enabled": true, "address": "10.1.2.3" } });
    let t = server(bad, 0).await;
    assert!(t.svc.names().is_none());
    assert_eq!(t.get("/api/health").await.status, 200);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_responder_answers_for_the_suffix_and_only_for_it() {
    let t = server(names_config(&json!({})), 0).await;
    let live = up(&t);
    assert_eq!(live.dns.ip(), ADDRESS);
    let found = resolve(live.dns, "alpha.argus", WAIT)
        .await
        .expect("resolves");
    assert_eq!(found, [ADDRESS]);
    let aaaa = ask(live.dns, "alpha.argus", TYPE_AAAA, WAIT)
        .await
        .expect("answers");
    assert_eq!((aaaa.rcode, aaaa.records), (0, 0));
    let other = ask(live.dns, "example.com", 1, WAIT)
        .await
        .expect("answers");
    assert_eq!(other.rcode, 5);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_front_door_routes_by_host_like_the_main_listener() {
    let t = server(names_config(&json!({})), 0).await;
    let live = up(&t);
    let port = live.door.port;
    let board = at_door(&live, "/", &format!("alpha.argus:{port}")).await;
    assert_eq!(board.status, 302);
    assert_eq!(
        board.header("location"),
        Some("/p/alpha/?thread=coordinator")
    );
    let thread = at_door(&live, "/", &format!("ui.alpha.argus:{port}")).await;
    assert_eq!(thread.header("location"), Some("/p/alpha/?thread=ui"));
    for host in [
        format!("argus.argus:{port}"),
        format!("argus:{port}"),
        format!("nosuch.argus:{port}"),
    ] {
        let page = at_door(&live, "/", &host).await;
        assert_eq!(
            (page.status, page.header("location")),
            (200, None),
            "{host}"
        );
    }
    assert_eq!(
        at_door(&live, "/api/health", &format!("alpha.argus:{port}"))
            .await
            .status,
        200
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_front_door_refuses_every_other_host() {
    let t = server(names_config(&json!({ "suffix": "home.argus" })), 0).await;
    let live = up(&t);
    let port = live.door.port;
    let refused = [
        format!("evil.test:{port}"),
        format!("alpha.argus:{port}"),
        format!("alpha.home.argus:{}", port.wrapping_add(1)),
        format!("alpha.home.argus.evil.test:{port}"),
        "alpha.home.argus".to_owned(),
    ];
    for host in refused {
        let r = at_door(&live, "/api/health", &host).await;
        assert_eq!((r.status, r.code().as_str()), (421, "bad_host"), "{host}");
    }
    let ok = at_door(&live, "/api/health", &format!("alpha.home.argus:{port}")).await;
    assert_eq!(ok.status, 200);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_write_through_a_name_needs_that_name_as_its_origin() {
    let t = server(names_config(&json!({})), 0).await;
    let live = up(&t);
    let host = format!("alpha.argus:{}", live.door.port);
    let addr = SocketAddr::from((live.door.address, live.door.port));
    crate::doors::assert_write_needs_own_origin(addr, &host).await;
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_health_answer_tells_the_cli_where_the_names_are() {
    let t = server(names_config(&json!({})), 0).await;
    let live = up(&t);
    let health = t.get("/api/health").await.json()["localNames"].clone();
    assert_eq!(health["address"], ADDRESS.to_string());
    assert_eq!(health["suffix"], "argus");
    assert_eq!(health["doorPort"], live.door.port);
    assert_eq!(health["dns"], live.dns.to_string());
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_busy_port_for_the_door_falls_back_to_the_main_port() {
    let busy = TcpListener::bind((ADDRESS, 0)).await.expect("a busy port");
    let taken = busy.local_addr().expect("its address").port();
    let t = server(names_config(&json!({})), taken).await;
    let live = up(&t);
    assert_eq!(live.door.port, t.port, "the names carry the main port");
    let host = format!("alpha.argus:{}", t.port);
    assert_eq!(at_door(&live, "/api/health", &host).await.status, 200);
    t.stop(false).await;
    drop(busy);
}
