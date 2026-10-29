//! The local names in the server: the DNS responder and the front door on a loopback address of
//! their own, off by default. Every socket is on port 0 (never 53, 80, 47615 or 47616), and the
//! address is not the default one, so a running Argus of the owner is never in the way.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use argus_core::names::Door;
use argus_dns::{TYPE_AAAA, ask, resolve};
use argus_server::LiveNames;
use argus_server::security::{Doors, host_alias, parse_door_alias, peer_ok};
use argus_testkit::{Reply, Request, TestOptions, TestServer, request_at};
use serde_json::{Value, json};
use tokio::net::TcpListener;

#[path = "support/doors.rs"]
mod doors;

use doors::alias;

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
    doors::assert_write_needs_own_origin(addr, &host).await;
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

fn door(port: u16) -> Door {
    Door {
        address: ADDRESS,
        suffix: "argus".to_owned(),
        port,
    }
}

fn names_only(door: &Door) -> Doors<'_> {
    Doors {
        names: Some(door),
        ..Doors::default()
    }
}

#[test]
fn a_name_under_the_suffix_is_an_alias_with_the_port_of_the_door() {
    let (at_80, at_port) = (door(80), door(47615));
    assert_eq!(
        parse_door_alias("rotp.argus", &at_80),
        Some(alias("rotp", None, false))
    );
    assert_eq!(
        parse_door_alias("UI.Rotp.argus", &at_80),
        Some(alias("rotp", Some("ui"), false))
    );
    assert_eq!(
        parse_door_alias("argus.argus", &at_80),
        Some(alias("argus", None, true))
    );
    assert_eq!(
        parse_door_alias("argus", &at_80),
        Some(alias("argus", None, true))
    );
    assert_eq!(
        parse_door_alias("overlord.argus:47615", &at_port),
        Some(alias("overlord", None, true))
    );
    for (host, door) in [
        ("rotp.argus:80", &at_80),
        ("rotp.argus", &at_port),
        ("rotp.argus:47616", &at_port),
        ("a.b.c.argus", &at_80),
        ("rotp.argus.evil.test", &at_80),
        ("rotp.localhost", &at_80),
        ("-x.argus", &at_80),
        (".argus", &at_80),
    ] {
        assert_eq!(parse_door_alias(host, door), None, "{host}");
    }
}

#[test]
fn the_localhost_aliases_still_work_beside_the_names() {
    let at_80 = door(80);
    assert_eq!(
        host_alias("rotp.localhost:4000", 4000, names_only(&at_80)),
        Some(alias("rotp", None, false))
    );
    assert_eq!(
        host_alias("rotp.argus", 4000, names_only(&at_80)),
        Some(alias("rotp", None, false))
    );
    assert_eq!(host_alias("rotp.argus", 4000, Doors::default()), None);
}

#[test]
fn the_address_of_argus_is_a_loopback_peer_only_while_the_names_are_up() {
    let at_80 = door(80);
    let ip = std::net::IpAddr::V4(ADDRESS);
    assert!(peer_ok(ip, Some(&at_80)));
    assert!(!peer_ok(ip, None));
    assert!(!peer_ok(
        std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
        Some(&at_80)
    ));
}
