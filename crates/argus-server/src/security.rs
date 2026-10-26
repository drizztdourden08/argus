//! Who may talk to the server (DESIGN 8, the Node `security.mjs`): loopback peers, the Host
//! check against DNS rebinding (with the board alias hosts), the Origin, Sec-Fetch-Site and
//! Content-Type rules of writes, the bearer, and the optional token mode with its cookie and
//! one-time login codes.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};

use argus_core::clock::{iso_from_ms, now_ms};
use argus_core::{ApiError, ErrorCode};
use axum::http::HeaderMap;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// The cookie of the token mode.
pub const COOKIE: &str = "coord_dash";
/// Its lifetime.
pub const COOKIE_MAX_AGE: u32 = 31_536_000;
/// How long a login code lives.
pub const CODE_TTL_MS: i64 = 60_000;
const MAX_CODES: usize = 50;
/// The alias names that are never a project.
pub const RESERVED_ALIASES: [&str; 2] = ["argus", "overlord"];

/// A board alias host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    /// The project key (or the reserved name).
    pub key: String,
    /// The thread label, when given.
    pub thread: Option<String>,
    /// `argus` or `overlord`.
    pub reserved: bool,
}

fn label_ok(s: &str, first_strict: bool) -> bool {
    let valid = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
    !s.is_empty() && s.len() <= 63 && s.chars().all(valid) && (!first_strict || !s.starts_with('-'))
}

/// `<key>.localhost:<port>` or `<thread>.<key>.localhost:<port>`, case-insensitive, our port only.
#[must_use]
pub fn parse_alias(host: &str, port: u16) -> Option<Alias> {
    let lower = host.to_ascii_lowercase();
    let name = lower.strip_suffix(&format!(".localhost:{port}"))?;
    let labels: Vec<&str> = name.split('.').collect();
    let (thread, key) = match labels.as_slice() {
        [key] => (None, *key),
        [thread, key] => (Some(*thread), *key),
        _ => return None,
    };
    if !label_ok(key, true) || !thread.is_none_or(|t| label_ok(t, false)) {
        return None;
    }
    let reserved = RESERVED_ALIASES.contains(&key);
    let key = if key == "argus-app" { "argus" } else { key };
    Some(Alias {
        key: key.to_owned(),
        thread: thread.map(str::to_owned),
        reserved,
    })
}

/// Equal-length digests first, so the time does not depend on the length.
#[must_use]
pub fn token_equals(a: &str, b: &str) -> bool {
    let ha = Sha256::digest(a.as_bytes());
    let hb = Sha256::digest(b.as_bytes());
    bool::from(ha.ct_eq(&hb)) && a.len() == b.len()
}

/// The cookies of a `Cookie` header.
#[must_use]
pub fn parse_cookies(header: &str) -> HashMap<String, String> {
    header
        .split(';')
        .filter_map(|part| {
            let i = part.find('=').filter(|i| *i > 0)?;
            Some((part[..i].trim().to_owned(), part[i + 1..].trim().to_owned()))
        })
        .collect()
}

/// `next` of `/auth`: a path on this server, never `//host` or `/\host`.
#[must_use]
pub fn safe_next(next: Option<&str>) -> &str {
    match next {
        Some(n)
            if n.starts_with('/')
                && !n[1..].starts_with(['/', '\\'])
                && !n.contains(['\r', '\n']) =>
        {
            n
        }
        _ => "/",
    }
}

/// The bearer of a request: absent, ours, or sent and wrong (always a 401).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bearer {
    /// No `Authorization` header.
    None,
    /// Our token.
    Ok,
    /// Anything else.
    Bad,
}

fn header<'a>(h: &'a HeaderMap, name: &str) -> Option<&'a str> {
    h.get(name).and_then(|v| v.to_str().ok())
}

/// The bearer state of a request.
#[must_use]
pub fn bearer_state(h: &HeaderMap, token: &str) -> Bearer {
    let Some(raw) = header(h, "authorization").filter(|v| !v.trim().is_empty()) else {
        return Bearer::None;
    };
    let mut parts = raw.split_whitespace();
    let ok = parts
        .next()
        .is_some_and(|s| s.eq_ignore_ascii_case("bearer"))
        && parts.next().is_some_and(|t| token_equals(t, token))
        && parts.next().is_none();
    if ok { Bearer::Ok } else { Bearer::Bad }
}

/// The socket peer must be loopback.
#[must_use]
pub fn peer_ok(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.octets() == [127, 0, 0, 1],
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6
                    .to_ipv4_mapped()
                    .is_some_and(|v4| v4.octets() == [127, 0, 0, 1])
        }
    }
}

/// `Host` is `127.0.0.1:<port>`, `localhost:<port>` or a board alias.
#[must_use]
pub fn host_ok(h: &HeaderMap, port: u16) -> bool {
    let host = header(h, "host").unwrap_or_default().to_ascii_lowercase();
    host == format!("127.0.0.1:{port}")
        || host == format!("localhost:{port}")
        || parse_alias(&host, port).is_some()
}

fn base_type(h: &HeaderMap) -> String {
    header(h, "content-type")
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn sub_type_ok(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.+-".contains(c))
}

fn json_type(t: &str) -> bool {
    t.strip_prefix("application/")
        .is_some_and(|s| sub_type_ok(s) && (s == "json" || s.ends_with("+json")))
}

fn image_type(t: &str) -> bool {
    t.strip_prefix("image/").is_some_and(sub_type_ok)
}

/// The default mode's rule for writes; `None` when fine.
#[must_use]
pub fn write_refusal(h: &HeaderMap, images: bool) -> Option<ApiError> {
    let host = header(h, "host").unwrap_or_default().to_ascii_lowercase();
    if header(h, "origin").is_some_and(|o| o.to_ascii_lowercase() != format!("http://{host}")) {
        return Some(ApiError::new(
            ErrorCode::Forbidden,
            "cross-origin request refused",
        ));
    }
    if header(h, "sec-fetch-site")
        .is_some_and(|s| !["same-origin", "same-site", "none"].contains(&s))
    {
        return Some(ApiError::new(
            ErrorCode::Forbidden,
            "cross-site request refused",
        ));
    }
    let t = base_type(h);
    if !(json_type(&t) || images && image_type(&t)) {
        let message = if images {
            "send application/json, or the image itself"
        } else {
            "send application/json"
        };
        return Some(ApiError::new(ErrorCode::UnsupportedMedia, message));
    }
    None
}

/// A browser marks its requests (Sec-Fetch-Site, or Origin on a write); a CLI does not.
#[must_use]
pub fn looks_like_page(h: &HeaderMap) -> bool {
    h.contains_key("sec-fetch-site") || h.contains_key("origin")
}

/// Origin and Sec-Fetch-Site, when sent, must be ours; `strict` also requires an Origin.
#[must_use]
pub fn origin_ok(h: &HeaderMap, port: u16, strict: bool) -> bool {
    let host = header(h, "host").unwrap_or_default().to_ascii_lowercase();
    let own = |o: &str| {
        o == format!("http://127.0.0.1:{port}")
            || o == format!("http://localhost:{port}")
            || (parse_alias(&host, port).is_some() && o == format!("http://{host}"))
    };
    let origin = header(h, "origin").map(str::to_ascii_lowercase);
    if origin.as_deref().is_some_and(|o| !own(o)) {
        return false;
    }
    if header(h, "sec-fetch-site").is_some_and(|s| s != "same-origin" && s != "none") {
        return false;
    }
    !(strict && origin.is_none())
}

/// The token mode's credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Credential {
    /// A valid bearer.
    Bearer,
    /// A valid cookie.
    Cookie,
}

/// The credential of a request in token mode.
#[must_use]
pub fn credential(h: &HeaderMap, token: &str) -> Option<Credential> {
    if bearer_state(h, token) == Bearer::Ok {
        return Some(Credential::Bearer);
    }
    let cookies = parse_cookies(header(h, "cookie").unwrap_or_default());
    cookies
        .get(COOKIE)
        .is_some_and(|c| token_equals(c, token))
        .then_some(Credential::Cookie)
}

/// The cookie set by a redeemed login code.
#[must_use]
pub fn cookie_header(token: &str) -> String {
    format!("{COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age={COOKIE_MAX_AGE}")
}

/// One-time login codes, kept in memory for 60 s.
#[derive(Default)]
pub struct LoginCodes {
    codes: Mutex<Vec<(String, i64)>>,
}

impl LoginCodes {
    /// A new code: `(code, expiresAt)`.
    #[must_use]
    pub fn issue(&self) -> (String, String) {
        self.issue_at(now_ms())
    }

    /// A new code issued at `t`.
    #[must_use]
    pub fn issue_at(&self, t: i64) -> (String, String) {
        let mut codes = self.codes.lock().unwrap_or_else(PoisonError::into_inner);
        codes.retain(|(_, exp)| *exp > t);
        while codes.len() >= MAX_CODES {
            codes.remove(0);
        }
        let mut b = [0u8; 16];
        getrandom::fill(&mut b).unwrap_or_default();
        let code = argus_core::config::hex(&b);
        codes.push((code.clone(), t + CODE_TTL_MS));
        (code, iso_from_ms(t + CODE_TTL_MS))
    }

    /// Single use: a redeemed or expired code is gone.
    #[must_use]
    pub fn redeem_at(&self, code: &str, t: i64) -> bool {
        let mut codes = self.codes.lock().unwrap_or_else(PoisonError::into_inner);
        let found = codes
            .iter()
            .position(|(c, _)| c == code)
            .map(|i| codes.remove(i));
        found.is_some_and(|(_, exp)| exp > t)
    }

    /// [`Self::redeem_at`] now.
    #[must_use]
    pub fn redeem(&self, code: &str) -> bool {
        self.redeem_at(code, now_ms())
    }

    /// Forgets every code (token rotation).
    pub fn clear(&self) {
        self.codes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }
}
