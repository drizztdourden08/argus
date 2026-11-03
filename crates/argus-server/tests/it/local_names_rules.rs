//! K1, K4: the host rules of the local names with no socket: a name under the suffix is an alias
//! with the port of the door, the localhost aliases keep working, and the address of Argus is a
//! loopback peer only while the names are up.
// test-tag: critical

use std::net::Ipv4Addr;

use argus_core::names::Door;
use argus_server::security::{Doors, host_alias, parse_door_alias, peer_ok};

use crate::doors::alias;

const ADDRESS: Ipv4Addr = Ipv4Addr::new(127, 65, 82, 72);

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
