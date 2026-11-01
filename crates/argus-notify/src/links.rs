//! Where a click on a toast goes: the project's board, else the Notifications tab. The addresses
//! follow the one link rule of `argus_core::page_url`: the `.localhost` names while the server's
//! localhost door is bound, else `http://127.0.0.1:<port>`. The local names (`http://rotp.argus/`)
//! are used only when the owner opted into them and the localhost door is not bound.
// test-tag: critical

use argus_core::names::Door;
use argus_core::page_url::{OVERVIEW_LABEL, page_at};

const URL_MAX: usize = 300;
const KEY_MAX: usize = 40;
const HOSTS: [&str; 2] = ["localhost", "127.0.0.1"];
const RESERVED: [&str; 3] = ["portfolio", "argus", "overlord"];

/// The addresses of this machine's dashboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Links {
    port: u16,
    door: Option<Door>,
    localhost_door: Option<u16>,
}

impl Links {
    /// The addresses of the dashboard listening on `port`: under the `.localhost` names when
    /// `localhost_door` (its port) is bound, else under the names of `door` when the local names
    /// are on, else on the loopback port.
    #[must_use]
    pub fn new(port: u16, door: Option<Door>, localhost_door: Option<u16>) -> Self {
        Self {
            port,
            door,
            localhost_door,
        }
    }

    fn page(&self, key: Option<&str>) -> String {
        match (&self.door, self.localhost_door) {
            (Some(door), None) => {
                format!("http://{}/", door.host(key.unwrap_or(OVERVIEW_LABEL)))
            }
            (_, localhost) => page_at(localhost, self.port, key).url(),
        }
    }

    /// The address a click opens: `given` when it is an http address on this machine, else the
    /// board of `project` (a project other than `portfolio`, `argus` and `overlord`), else the
    /// Notifications tab.
    #[must_use]
    pub fn url_for(&self, project: Option<&str>, given: Option<&str>) -> String {
        let suffix = self.door.as_ref().map(|door| door.suffix.as_str());
        if let Some(url) = given.filter(|url| is_local_http(url, suffix)) {
            return url.to_owned();
        }
        match project.filter(|key| is_board_key(key)) {
            Some(key) => self.page(Some(key)),
            None => format!("{}#notifications", self.page(None)),
        }
    }
}

fn is_board_key(key: &str) -> bool {
    let mut chars = key.chars();
    let first_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    first_ok
        && key.len() <= KEY_MAX
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !RESERVED.contains(&key)
}

fn is_local_http(url: &str, suffix: Option<&str>) -> bool {
    let lower = url.to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("http://") else {
        return false;
    };
    let (authority, path) = rest.find('/').map_or((rest, ""), |at| rest.split_at(at));
    url.chars().count() <= URL_MAX && is_local_authority(authority, suffix) && is_plain_path(path)
}

fn is_local_authority(authority: &str, suffix: Option<&str>) -> bool {
    let (host, port) = authority
        .split_once(':')
        .map_or((authority, None), |(host, port)| (host, Some(port)));
    port.is_none_or(|port| !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()))
        && is_local_host(host, suffix)
}

fn is_local_host(host: &str, suffix: Option<&str>) -> bool {
    HOSTS.iter().copied().chain(suffix).any(|local| {
        host == local
            || host
                .strip_suffix(local)
                .and_then(|prefix| prefix.strip_suffix('.'))
                .is_some_and(|labels| labels.split('.').all(is_label))
    })
}

fn is_label(label: &str) -> bool {
    !label.is_empty()
        && label
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn is_plain_path(path: &str) -> bool {
    path.chars()
        .all(|c| !c.is_whitespace() && !matches!(c, '"' | '<' | '>'))
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use argus_core::names::Door;

    use super::Links;

    const LINKS: Links = Links {
        port: 47615,
        door: None,
        localhost_door: None,
    };

    fn named(port: u16, localhost_door: Option<u16>) -> Links {
        let door = Door {
            address: Ipv4Addr::new(127, 65, 82, 71),
            suffix: "argus".to_owned(),
            port,
        };
        Links::new(47615, Some(door), localhost_door)
    }

    #[test]
    fn a_project_opens_its_board_and_the_rest_opens_the_tab() {
        let tab = "http://127.0.0.1:47615/#notifications";
        assert_eq!(
            LINKS.url_for(Some("rotp"), None),
            "http://127.0.0.1:47615/p/rotp/"
        );
        for project in [Some("portfolio"), Some("argus"), Some("Not A Key"), None] {
            assert_eq!(LINKS.url_for(project, None), tab);
        }
    }

    #[test]
    fn with_the_localhost_door_bound_the_links_use_the_localhost_names() {
        let up = Links::new(47615, None, Some(80));
        assert_eq!(up.url_for(Some("rotp"), None), "http://rotp.localhost/");
        assert_eq!(
            up.url_for(None, None),
            "http://argus.localhost/#notifications"
        );
        let other = Links::new(47615, None, Some(8080));
        assert_eq!(
            other.url_for(Some("rotp"), None),
            "http://rotp.localhost:8080/"
        );
    }

    #[test]
    fn a_given_address_must_be_http_on_this_machine() {
        let tab = "http://127.0.0.1:47615/#notifications";
        let local = "http://rotp.localhost:47615/#x";
        assert_eq!(LINKS.url_for(None, Some(local)), local);
        assert_eq!(
            LINKS.url_for(None, Some("HTTP://127.0.0.1:80/a")),
            "HTTP://127.0.0.1:80/a"
        );
        for bad in [
            "file:///c:/windows/system32/calc.exe",
            "http://evil.example.com/",
            "http://localhost.evil.com/",
            "http://evil.com/localhost",
            "https://rotp.localhost/",
            "http://localhost:/",
            "http://localhost/a b",
        ] {
            assert_eq!(LINKS.url_for(None, Some(bad)), tab, "{bad}");
        }
    }

    #[test]
    fn the_local_names_replace_the_loopback_address_while_the_localhost_door_is_not_bound() {
        let at_80 = named(80, None);
        assert_eq!(at_80.url_for(Some("rotp"), None), "http://rotp.argus/");
        assert_eq!(
            at_80.url_for(None, None),
            "http://argus.argus/#notifications"
        );
        assert_eq!(
            at_80.url_for(Some("portfolio"), None),
            "http://argus.argus/#notifications"
        );
        let at_port = named(47615, None);
        assert_eq!(
            at_port.url_for(Some("rotp"), None),
            "http://rotp.argus:47615/"
        );
    }

    #[test]
    fn the_localhost_door_wins_over_the_local_names() {
        let both = named(80, Some(80));
        assert_eq!(both.url_for(Some("rotp"), None), "http://rotp.localhost/");
        assert_eq!(
            both.url_for(None, None),
            "http://argus.localhost/#notifications"
        );
    }

    #[test]
    fn a_given_address_may_use_the_local_names_only_while_they_are_on() {
        let given = "http://ui.rotp.argus/#x";
        assert_eq!(named(80, None).url_for(None, Some(given)), given);
        assert_eq!(
            named(80, None).url_for(None, Some("http://rotp.argus.evil.com/")),
            "http://argus.argus/#notifications"
        );
        assert_eq!(
            LINKS.url_for(None, Some(given)),
            "http://127.0.0.1:47615/#notifications"
        );
    }
}
