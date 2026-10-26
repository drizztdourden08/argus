//! Request paths and queries as the Node server sees them through `new URL()`: dot segments
//! (also percent-encoded) resolved, backslashes read as slashes, and `decodeURIComponent` for
//! route params (a malformed escape is an error).

fn is_dot(seg: &str) -> bool {
    seg == "." || seg.eq_ignore_ascii_case("%2e")
}

fn is_dot_dot(seg: &str) -> bool {
    matches!(
        seg.to_ascii_lowercase().as_str(),
        ".." | ".%2e" | "%2e." | "%2e%2e"
    )
}

/// The pathname after URL parsing: dot segments resolved, `\` read as `/`.
#[must_use]
pub fn normalize(raw: &str) -> String {
    let path = raw.replace('\\', "/");
    let segments: Vec<&str> = path.split('/').skip(1).collect();
    let mut out: Vec<&str> = Vec::new();
    let last = segments.len().saturating_sub(1);
    for (i, seg) in segments.iter().enumerate() {
        if is_dot_dot(seg) {
            out.pop();
        } else if !is_dot(seg) {
            out.push(seg);
            continue;
        }
        if i == last {
            out.push("");
        }
    }
    format!("/{}", out.join("/"))
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// `decodeURIComponent`: `None` on a malformed escape or bytes that are not UTF-8.
#[must_use]
pub fn decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hi = hex(*bytes.get(i + 1)?)?;
            let lo = hex(*bytes.get(i + 2)?)?;
            out.push(hi << 4 | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The query as `URLSearchParams` reads it.
#[must_use]
pub fn query_pairs(query: Option<&str>) -> Vec<(String, String)> {
    form_urlencoded::parse(query.unwrap_or_default().as_bytes())
        .into_owned()
        .collect()
}
