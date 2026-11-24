//! The localhost door: a second listener on `127.0.0.1` that serves the app for
//! `<key>.localhost`, independent of the local names. Every socket here is on port 0 (never 80,
//! 53 or 47615); the port-80 form of the Host is covered by the pure rules at the end.
//! K2, K4, S3: the localhost door on a socket of port 0; the pure rules are in `localhost_door_rules.rs`.
// test-tag: standard

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use argus_testkit::{Reply, Request, TestOptions, TestServer, request_at};
use serde_json::{Value, json};
use tokio::net::TcpListener;

async fn server(config: Value, door: Option<u16>) -> TestServer {
    let options = TestOptions {
        seed: true,
        sync: false,
        config,
        tweak: Some(Arc::new(move |o| o.localhost_door_port = door)),
        ..TestOptions::default()
    };
    TestServer::start(options).await
}

fn door_port(t: &TestServer) -> u16 {
    t.svc.localhost_door().expect("the localhost door is bound")
}

async fn at_door(t: &TestServer, path: &str, host: &str) -> Reply {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, door_port(t)));
    request_at(addr, &Request::new("GET", path).header("Host", host)).await
}

fn pid_record(t: &TestServer) -> Value {
    let text = std::fs::read_to_string(t.dashboard_home.join("server.pid")).expect("pid file");
    serde_json::from_str(&text).expect("pid json")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_door_redirects_a_lead_and_a_thread_like_the_main_listener() {
    let t = server(json!({}), Some(0)).await;
    let port = door_port(&t);
    let lead = at_door(&t, "/", &format!("alpha.localhost:{port}")).await;
    assert_eq!(lead.status, 302);
    assert_eq!(lead.header("location"), Some("/p/alpha/?thread=lead"));
    let thread = at_door(&t, "/", &format!("ui.alpha.localhost:{port}")).await;
    assert_eq!(thread.status, 302);
    assert_eq!(thread.header("location"), Some("/p/alpha/?thread=ui"));
    let overview = at_door(&t, "/", &format!("argus.localhost:{port}")).await;
    assert_eq!((overview.status, overview.header("location")), (200, None));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_door_serves_the_api_and_refuses_every_other_host() {
    let t = server(json!({}), Some(0)).await;
    let port = door_port(&t);
    let ok = at_door(&t, "/api/health", &format!("alpha.localhost:{port}")).await;
    assert_eq!(ok.status, 200);
    let refused = [
        format!("evil.test:{port}"),
        format!("alpha.localhost:{}", port.wrapping_add(1)),
        format!("alpha.localhost.evil.test:{port}"),
        "alpha.localhost".to_owned(),
        format!("a.b.c.localhost:{port}"),
    ];
    for host in refused {
        let r = at_door(&t, "/api/health", &host).await;
        assert_eq!((r.status, r.code().as_str()), (421, "bad_host"), "{host}");
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_write_through_the_door_needs_that_host_as_its_origin() {
    let t = server(json!({}), Some(0)).await;
    let host = format!("alpha.localhost:{}", door_port(&t));
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, door_port(&t)));
    crate::doors::assert_write_needs_own_origin(addr, &host).await;
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn health_and_the_pid_record_carry_the_door_only_while_it_is_bound() {
    let t = server(json!({}), Some(0)).await;
    let port = door_port(&t);
    let health = t.get("/api/health").await.json();
    assert_eq!(health["localhostDoor"], port);
    assert_eq!(pid_record(&t)["localhostDoor"], port);
    t.stop(false).await;

    let off = server(json!({}), None).await;
    assert_eq!(
        off.get("/api/health").await.json()["localhostDoor"],
        Value::Null
    );
    assert!(pid_record(&off).get("localhostDoor").is_none());
    off.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_config_key_turns_the_door_off() {
    let t = server(json!({ "localhostDoor": false }), Some(0)).await;
    assert_eq!(t.svc.localhost_door(), None);
    assert_eq!(
        t.get("/api/health").await.json()["localhostDoor"],
        Value::Null
    );
    assert!(pid_record(&t).get("localhostDoor").is_none());
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_port_that_cannot_be_bound_leaves_the_door_off_and_the_server_up() {
    let busy = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("a busy port");
    let taken = busy.local_addr().expect("its address").port();
    let t = server(json!({}), Some(taken)).await;
    assert_eq!(t.svc.localhost_door(), None);
    assert_eq!(t.get("/api/health").await.status, 200);
    assert!(pid_record(&t).get("localhostDoor").is_none());
    t.stop(false).await;
    drop(busy);
}
