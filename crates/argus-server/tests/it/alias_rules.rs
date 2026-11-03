//! K1, K4: the alias host rules in isolation: which hosts parse as a board alias (with the
//! reserved names `argus` and `overlord`), and which Origin counts against which alias host.
// test-tag: critical

use argus_server::security::{Alias, Doors, RESERVED_ALIASES, host_ok, origin_ok, parse_alias};
use axum::http::{HeaderMap, HeaderValue};

const NODE_PORT: u16 = 47615;

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
