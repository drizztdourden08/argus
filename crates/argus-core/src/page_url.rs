//! The address of a dashboard page in the owner's browser. When the server's `/api/health` says
//! the localhost door is bound (`localhostDoor`, its port), pages use the `.localhost` names, which
//! every browser resolves with no setup: `http://argus.localhost/` for the overview and
//! `http://<key>.localhost/` for a board. Otherwise they use `http://127.0.0.1:<port>`.

use serde_json::Value;

/// The key of the health answer that names the port of the localhost door.
pub const HEALTH_KEY: &str = "localhostDoor";
/// The name of the overview.
pub const OVERVIEW_LABEL: &str = "argus";
/// The name of the board of the project `argus`, whose key is the overview's name.
pub const ARGUS_BOARD_LABEL: &str = "argus-app";
const HTTP_PORT: u16 = 80;

/// A page: the origin and the path under it, kept apart for a sign-in link that carries the path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// `http://argus.localhost` or `http://127.0.0.1:<port>`.
    pub origin: String,
    /// `/` or `/p/<key>/`.
    pub path: String,
}

impl Page {
    /// The whole address.
    #[must_use]
    pub fn url(&self) -> String {
        format!("{}{}", self.origin, self.path)
    }
}

/// The host label of a project's board: its key, `argus-app` for the project `argus`.
#[must_use]
pub fn board_label(key: &str) -> &str {
    if key == OVERVIEW_LABEL {
        ARGUS_BOARD_LABEL
    } else {
        key
    }
}

/// The port of the localhost door a health answer names, when it is bound.
#[must_use]
pub fn localhost_door(health: &Value) -> Option<u16> {
    u16::try_from(health[HEALTH_KEY].as_u64()?).ok()
}

fn named_origin(label: &str, door: u16) -> String {
    if door == HTTP_PORT {
        format!("http://{label}.localhost")
    } else {
        format!("http://{label}.localhost:{door}")
    }
}

/// The overview (no key) or a board of the server on `port`, under the localhost names when
/// `health` says the door is bound.
#[must_use]
pub fn page(health: &Value, port: u16, key: Option<&str>) -> Page {
    match (localhost_door(health), key) {
        (Some(door), _) => Page {
            origin: named_origin(key.map_or(OVERVIEW_LABEL, board_label), door),
            path: "/".to_owned(),
        },
        (None, key) => Page {
            origin: format!("http://127.0.0.1:{port}"),
            path: key.map_or_else(|| "/".to_owned(), |k| format!("/p/{k}/")),
        },
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::page;

    #[test]
    fn with_the_door_bound_pages_use_the_localhost_names() {
        let up = json!({ "ok": true, "localhostDoor": 80 });
        assert_eq!(page(&up, 47615, None).url(), "http://argus.localhost/");
        assert_eq!(
            page(&up, 47615, Some("rotp")).url(),
            "http://rotp.localhost/"
        );
        assert_eq!(
            page(&up, 47615, Some("argus")).url(),
            "http://argus-app.localhost/"
        );
        let other = json!({ "localhostDoor": 8080 });
        assert_eq!(
            page(&other, 47615, Some("rotp")).url(),
            "http://rotp.localhost:8080/"
        );
    }

    #[test]
    fn without_the_door_pages_use_the_loopback_port() {
        for health in [
            json!({ "ok": true }),
            json!(null),
            json!({ "localhostDoor": "80" }),
        ] {
            assert_eq!(page(&health, 4000, None).url(), "http://127.0.0.1:4000/");
            assert_eq!(
                page(&health, 4000, Some("argus")).url(),
                "http://127.0.0.1:4000/p/argus/"
            );
        }
    }
}
