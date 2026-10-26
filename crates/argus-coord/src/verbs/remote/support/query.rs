//! Query strings as JavaScript builds them: `URLSearchParams` and `encodeURIComponent`.

use std::fmt::Write;

pub use argus_core::js::encode_uri_component as uri_component;

fn form_encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        match byte {
            b' ' => out.push('+'),
            b'*' | b'-' | b'.' | b'_' => out.push(char::from(byte)),
            b if b.is_ascii_alphanumeric() => out.push(char::from(b)),
            b => {
                write!(out, "%{b:02X}").ok();
            }
        }
    }
    out
}

/// `new URLSearchParams()` with the pairs set in order.
#[derive(Debug, Default)]
pub struct Query {
    pairs: Vec<String>,
}

impl Query {
    /// An empty query.
    #[must_use]
    pub fn new() -> Self {
        Query::default()
    }

    /// `query.set(key, value)` for a key not set yet.
    #[must_use]
    pub fn with(mut self, key: &str, value: &str) -> Self {
        self.pairs
            .push(format!("{}={}", form_encode(key), form_encode(value)));
        self
    }

    /// `?a=b&c=d`, or nothing for an empty query.
    #[must_use]
    pub fn suffix(&self) -> String {
        if self.pairs.is_empty() {
            String::new()
        } else {
            format!("?{}", self.pairs.join("&"))
        }
    }
}
