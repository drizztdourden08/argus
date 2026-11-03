//! `argus dns status`: whether the local names are on, whether the responder really answers (a
//! query to its socket), where the front door is, and whether this machine's resolver already
//! sends the suffix there. Read-only.

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::time::Duration;

use argus_core::Paths;
use argus_core::names::{HTTP_PORT, LocalNames};
use argus_dns::resolve;
use serde_json::Value;
use tokio::time::timeout;

use crate::lifecycle::live::up;

const DNS_PORT: u16 = 53;
const ASK_WAIT: Duration = Duration::from_secs(2);
const SYSTEM_WAIT: Duration = Duration::from_secs(4);

/// What was found.
pub struct Report {
    names: LocalNames,
    /// Whether the server answered its health call.
    server_up: bool,
    /// The front door port the server reported.
    door_port: Option<u16>,
    /// The localhost door port the server reported.
    localhost_door: Option<u16>,
    /// Where the responder was asked.
    dns: SocketAddr,
    /// Whether it answered with the address of Argus.
    answering: bool,
    /// Whether the system resolver gives the address of Argus for `argus.<suffix>`.
    system: Option<bool>,
}

fn dns_of(health: &Value, names: &LocalNames) -> SocketAddr {
    health["localNames"]["dns"]
        .as_str()
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(|| (names.address, DNS_PORT).into())
}

async fn answers(dns: SocketAddr, names: &LocalNames) -> bool {
    let name = format!("argus.{}", names.suffix);
    resolve(dns, &name, ASK_WAIT)
        .await
        .is_ok_and(|found| found == [names.address])
}

/// Asks the machine's own resolver, as a browser's process would.
async fn system_resolves(names: &LocalNames) -> bool {
    let host = format!("argus.{}", names.suffix);
    let address = names.address;
    let lookup = tokio::task::spawn_blocking(move || {
        (host.as_str(), HTTP_PORT)
            .to_socket_addrs()
            .is_ok_and(|mut found| found.any(|a| a.ip() == IpAddr::V4(address)))
    });
    timeout(SYSTEM_WAIT, lookup)
        .await
        .is_ok_and(|done| done.unwrap_or(false))
}

fn port_at(value: &Value) -> Option<u16> {
    value.as_u64().and_then(|p| u16::try_from(p).ok())
}

/// Finds out how the names stand.
pub async fn gather(paths: &Paths, names: LocalNames) -> Report {
    let found = up(paths).await;
    let health = found.as_ref().map_or(Value::Null, |u| u.health.clone());
    let dns = dns_of(&health, &names);
    let answering = answers(dns, &names).await;
    let system = if answering {
        Some(system_resolves(&names).await)
    } else {
        None
    };
    Report {
        door_port: port_at(&health["localNames"]["doorPort"]),
        localhost_door: port_at(&health["localhostDoor"]),
        server_up: found.is_some(),
        names,
        dns,
        answering,
        system,
    }
}

fn door_line(report: &Report) -> String {
    let suffix = &report.names.suffix;
    match (report.server_up, report.door_port) {
        (false, _) => "front door: the server is not running".to_owned(),
        (true, None) => "front door: not bound (see the server log)".to_owned(),
        (true, Some(HTTP_PORT)) => format!("front door: port 80, http://argus.{suffix}/"),
        (true, Some(port)) => format!(
            "front door: port {port} (port 80 was not available), http://argus.{suffix}:{port}/"
        ),
    }
}

fn localhost_line(report: &Report) -> String {
    match (report.server_up, report.localhost_door) {
        (false, _) => "localhost door: the server is not running".to_owned(),
        (true, None) => {
            "localhost door: not bound (port 80 is taken, or \"localhostDoor\" is false)".to_owned()
        }
        (true, Some(HTTP_PORT)) => "localhost door: port 80, http://argus.localhost/".to_owned(),
        (true, Some(port)) => {
            format!("localhost door: port {port}, http://argus.localhost:{port}/")
        }
    }
}

fn system_line(report: &Report) -> String {
    let (suffix, address) = (&report.names.suffix, report.names.address);
    match report.system {
        None => "system resolver: not checked (the responder does not answer)".to_owned(),
        Some(true) => format!("system resolver: argus.{suffix} resolves to {address}"),
        Some(false) => format!(
            "system resolver: argus.{suffix} does not resolve yet; run `argus dns install --print` and run its text as administrator"
        ),
    }
}

/// The lines to print.
#[must_use]
pub fn lines(report: &Report) -> Vec<String> {
    let names = &report.names;
    let state = if names.enabled { "on" } else { "off" };
    let mut out = vec![format!(
        "local names: {state} (address {}, suffix .{})",
        names.address, names.suffix
    )];
    if !names.enabled {
        out.push(
            "  turn them on with \"localNames\": { \"enabled\": true } in config.json".to_owned(),
        );
        out.push(localhost_line(report));
        return out;
    }
    let answering = if report.answering {
        "answering"
    } else {
        "not answering"
    };
    out.push(format!("dns: {answering} at {}", report.dns));
    out.push(door_line(report));
    out.push(localhost_line(report));
    out.push(system_line(report));
    out
}

fn print_lines(report: &Report) {
    for line in lines(report) {
        println!("{line}");
    }
}

/// Prints the status and answers 0, or 1 when the names are on and the responder is silent.
pub async fn run(paths: &Paths, names: LocalNames) -> u8 {
    if !names.enabled {
        let health = up(paths).await.map(|u| u.health);
        let report = Report {
            dns: (names.address, DNS_PORT).into(),
            names,
            server_up: health.is_some(),
            door_port: None,
            localhost_door: health.as_ref().and_then(|h| port_at(&h["localhostDoor"])),
            answering: false,
            system: None,
        };
        print_lines(&report);
        return 0;
    }
    let report = gather(paths, names).await;
    print_lines(&report);
    u8::from(!report.answering)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K2: the lines of argus dns status.
    use argus_core::names::LocalNames;

    use super::{Report, lines};

    fn report(enabled: bool) -> Report {
        let names = LocalNames {
            enabled,
            ..LocalNames::default()
        };
        Report {
            dns: (names.address, 53).into(),
            names,
            server_up: true,
            door_port: Some(80),
            localhost_door: Some(80),
            answering: true,
            system: Some(true),
        }
    }

    #[test]
    fn off_says_how_to_turn_them_on_and_nothing_else() {
        let text = lines(&report(false));
        assert_eq!(text.len(), 3);
        assert!(text[0].starts_with("local names: off (address 127.65.82.71, suffix .argus)"));
        assert!(text[1].contains("\"enabled\": true"));
        assert_eq!(text[2], "localhost door: port 80, http://argus.localhost/");
    }

    #[test]
    fn on_and_working_names_the_dns_the_door_and_the_resolver() {
        let text = lines(&report(true));
        assert_eq!(
            text[0],
            "local names: on (address 127.65.82.71, suffix .argus)"
        );
        assert_eq!(text[1], "dns: answering at 127.65.82.71:53");
        assert_eq!(text[2], "front door: port 80, http://argus.argus/");
        assert_eq!(text[3], "localhost door: port 80, http://argus.localhost/");
        assert_eq!(
            text[4],
            "system resolver: argus.argus resolves to 127.65.82.71"
        );
    }

    #[test]
    fn a_door_on_another_port_says_why_and_carries_the_port() {
        let mut r = report(true);
        r.door_port = Some(47615);
        assert_eq!(
            lines(&r)[2],
            "front door: port 47615 (port 80 was not available), http://argus.argus:47615/"
        );
    }

    #[test]
    fn a_silent_responder_and_a_stopped_server_are_said_plainly() {
        let mut r = report(true);
        (r.answering, r.system, r.server_up, r.door_port) = (false, None, false, None);
        let text = lines(&r);
        assert_eq!(text[1], "dns: not answering at 127.65.82.71:53");
        assert_eq!(text[2], "front door: the server is not running");
        assert_eq!(text[3], "localhost door: the server is not running");
        assert!(text[4].contains("not checked"));
        r.server_up = true;
        assert!(lines(&r)[2].contains("not bound"));
    }

    #[test]
    fn the_localhost_door_says_its_port_or_why_it_is_not_bound() {
        let mut r = report(true);
        r.localhost_door = Some(8080);
        assert_eq!(
            lines(&r)[3],
            "localhost door: port 8080, http://argus.localhost:8080/"
        );
        r.localhost_door = None;
        assert!(lines(&r)[3].starts_with("localhost door: not bound"));
    }

    #[test]
    fn a_resolver_that_does_not_know_the_suffix_points_at_the_setup() {
        let mut r = report(true);
        r.system = Some(false);
        assert!(lines(&r)[4].contains("argus dns install --print"));
    }
}
