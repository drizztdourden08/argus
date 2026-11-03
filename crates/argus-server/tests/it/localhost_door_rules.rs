//! K1, K4: the host and origin rules of the localhost door at port 80, where the Host carries no
//! port. No socket.
// test-tag: critical

use argus_server::security::{Doors, host_alias, host_ok, origin_ok};
use axum::http::{HeaderMap, HeaderValue};

use crate::doors::alias;

fn headers(host: &str, origin: Option<&str>) -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert("host", HeaderValue::from_str(host).expect("a host"));
    if let Some(origin) = origin {
        h.insert("origin", HeaderValue::from_str(origin).expect("an origin"));
    }
    h
}

#[test]
fn at_port_80_the_host_has_no_port() {
    let at_80 = Doors {
        localhost: Some(80),
        ..Doors::default()
    };
    let cases = [
        ("rotp.localhost", Some(alias("rotp", None, false))),
        ("UI.Rotp.localhost", Some(alias("rotp", Some("ui"), false))),
        ("argus.localhost", Some(alias("argus", None, true))),
        ("argus-app.localhost", Some(alias("argus", None, false))),
        ("rotp.localhost:80", None),
        ("rotp.localhost.evil.test", None),
        ("a.b.c.localhost", None),
        (".localhost", None),
        ("localhost", None),
    ];
    for (host, want) in cases {
        assert_eq!(host_alias(host, 47615, at_80), want, "{host}");
    }
    assert_eq!(host_alias("rotp.localhost", 47615, Doors::default()), None);
}

#[test]
fn at_port_80_the_host_and_origin_checks_accept_the_bare_name() {
    let at_80 = Doors {
        localhost: Some(80),
        ..Doors::default()
    };
    assert!(host_ok(&headers("rotp.localhost", None), 47615, at_80));
    assert!(!host_ok(
        &headers("rotp.localhost", None),
        47615,
        Doors::default()
    ));
    let same = headers("rotp.localhost", Some("http://rotp.localhost"));
    assert!(origin_ok(&same, 47615, at_80, true));
    let cross = headers("rotp.localhost", Some("http://evil.test"));
    assert!(!origin_ok(&cross, 47615, at_80, false));
    let other = headers("rotp.localhost", Some("http://argus.localhost"));
    assert!(!origin_ok(&other, 47615, at_80, false));
}
