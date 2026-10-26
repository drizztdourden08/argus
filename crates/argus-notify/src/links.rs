//! Where a click on a toast goes: the project's board, else the Notifications tab.

const URL_MAX: usize = 300;
const KEY_MAX: usize = 40;
const HOSTS: [&str; 2] = ["localhost", "127.0.0.1"];
const RESERVED: [&str; 3] = ["portfolio", "argus", "overlord"];

/// The addresses of this machine's dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Links {
    port: u16,
}

impl Links {
    /// The addresses of the dashboard listening on `port`.
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// The address a click opens: `given` when it is an http address on this machine, else the
    /// board of `project` (a project other than `portfolio`, `argus` and `overlord`), else the
    /// Notifications tab.
    #[must_use]
    pub fn url_for(self, project: Option<&str>, given: Option<&str>) -> String {
        if let Some(url) = given.filter(|url| is_local_http(url)) {
            return url.to_owned();
        }
        match project.filter(|key| is_board_key(key)) {
            Some(key) => format!("http://{key}.localhost:{}/", self.port),
            None => format!("http://argus.localhost:{}/#notifications", self.port),
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

fn is_local_http(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("http://") else {
        return false;
    };
    let (authority, path) = rest.find('/').map_or((rest, ""), |at| rest.split_at(at));
    url.chars().count() <= URL_MAX && is_local_authority(authority) && is_plain_path(path)
}

fn is_local_authority(authority: &str) -> bool {
    let (host, port) = authority
        .split_once(':')
        .map_or((authority, None), |(host, port)| (host, Some(port)));
    port.is_none_or(|port| !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()))
        && is_local_host(host)
}

fn is_local_host(host: &str) -> bool {
    HOSTS.iter().any(|local| {
        host == *local
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
    use super::Links;

    const LINKS: Links = Links { port: 47615 };

    #[test]
    fn a_project_opens_its_board_and_the_rest_opens_the_tab() {
        let tab = "http://argus.localhost:47615/#notifications";
        assert_eq!(
            LINKS.url_for(Some("rotp"), None),
            "http://rotp.localhost:47615/"
        );
        for project in [Some("portfolio"), Some("argus"), Some("Not A Key"), None] {
            assert_eq!(LINKS.url_for(project, None), tab);
        }
    }

    #[test]
    fn a_given_address_must_be_http_on_this_machine() {
        let tab = "http://argus.localhost:47615/#notifications";
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
}
