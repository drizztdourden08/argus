//! The matchers of the normaliser. Each one looks at the start of the rest of the text (and at the
//! byte before it) and, when it recognises a volatile value, says how many bytes it covers and
//! what replaces them. Everything here is ASCII, so scanning bytes is safe for UTF-8 text.

/// A recognised value: how many bytes it covers and its replacement.
pub struct Hit {
    pub len: usize,
    pub text: &'static str,
}

/// Where a matcher looks.
pub struct At<'a> {
    pub prev: Option<u8>,
    pub rest: &'a [u8],
    /// Session ids that are the same on both sides by construction; never replaced.
    pub stable_ids: &'a [String],
}

/// A matcher.
pub type Rule = fn(&At<'_>) -> Option<Hit>;

/// The rules, tried in this order at every position.
pub const RULES: [Rule; 7] = [
    iso_time,
    file_stamp,
    message_id,
    relative_time,
    epoch_ms,
    pid,
    uuid,
];

fn hit(len: usize, text: &'static str) -> Hit {
    Hit { len, text }
}

/// The number of leading bytes of `rest` that are ASCII digits.
fn digits(rest: &[u8]) -> usize {
    rest.iter().take_while(|b| b.is_ascii_digit()).count()
}

fn word_before(prev: Option<u8>) -> bool {
    prev.is_some_and(|b| b.is_ascii_alphanumeric())
}

/// Whether `rest` starts with `n` digits exactly and then `sep` (an empty `sep` always fits).
fn fixed(rest: &[u8], n: usize, sep: &[u8]) -> Option<usize> {
    let ok = digits(rest) >= n && rest[n..].starts_with(sep);
    ok.then_some(n + sep.len())
}

/// An ISO time as `new Date().toISOString()` writes it: `2026-10-26T15:49:13.648Z`.
fn iso_time(at: &At<'_>) -> Option<Hit> {
    let mut at_byte = 0;
    for (n, sep) in [
        (4, &b"-"[..]),
        (2, b"-"),
        (2, b"T"),
        (2, b":"),
        (2, b":"),
        (2, b""),
    ] {
        at_byte += fixed(&at.rest[at_byte..], n, sep)?;
    }
    if at.rest[at_byte..].starts_with(b".") {
        let frac = digits(&at.rest[at_byte + 1..]);
        at_byte += 1 + frac;
    }
    at.rest[at_byte..].starts_with(b"Z").then_some(Hit {
        len: at_byte + 1,
        text: "<TS>",
    })
}

/// A time safe in a file name, as in `board.v1.20261017T193800Z.json`.
fn file_stamp(at: &At<'_>) -> Option<Hit> {
    let len = fixed(at.rest, 8, b"T")?;
    let len = len + fixed(&at.rest[len..], 6, b"Z")?;
    Some(hit(len, "<STAMP>"))
}

/// An inbox message id: `m-<epoch ms>-<4 random characters or a name>`. The epoch is replaced;
/// a 4 character tail is replaced too (it is random for a plain message and a thread, lock
/// or item name for a system one, so a 4 character name is not told apart: a known blind spot).
fn message_id(at: &At<'_>) -> Option<Hit> {
    if word_before(at.prev) || !at.rest.starts_with(b"m-") {
        return None;
    }
    let ms = digits(&at.rest[2..]);
    if ms < 12 || at.rest.get(2 + ms) != Some(&b'-') {
        return None;
    }
    let head = 2 + ms + 1;
    let tail = at.rest[head..]
        .iter()
        .take_while(|b| b.is_ascii_alphanumeric())
        .count();
    if tail == 4 {
        return Some(hit(head + 4, "m-<MS>-<RND>"));
    }
    Some(hit(head, "m-<MS>-"))
}

/// The number of leading bytes of `rest` that are spaces.
fn spaces(rest: &[u8]) -> usize {
    rest.iter().take_while(|b| **b == b' ').count()
}

/// The length of `12s ago` at the start of `rest`, or none.
fn ago_len(rest: &[u8]) -> Option<usize> {
    let n = digits(rest);
    let unit = rest.get(n)?;
    let ok = n > 0 && b"smhd".contains(unit) && rest[n + 1..].starts_with(b" ago");
    ok.then_some(n + 5)
}

/// A relative time from `ago()`: `12s ago`, `5m ago`, `3h ago`, `2d ago`, with the padding of its
/// table column (`padStart` spaces before, `padEnd` spaces after). The cell widens and narrows
/// with the digits (`9s ago ` against `10s ago`) and the two engines run a moment apart, so the
/// padding is part of the volatile value. Padding with text on its side becomes one space (a lost
/// column separator still differs); at the start or the end of a line it is dropped, since it may
/// be empty on one side.
fn relative_time(at: &At<'_>) -> Option<Hit> {
    let lead = spaces(at.rest);
    if lead > 0 && at.prev == Some(b' ') {
        return None;
    }
    if lead == 0 && word_before(at.prev) {
        return None;
    }
    let end = lead + ago_len(&at.rest[lead..])?;
    let trail = spaces(&at.rest[end..]);
    let line_start = at.prev.is_none_or(|b| b == b'\n');
    let line_end = at
        .rest
        .get(end + trail)
        .is_none_or(|b| *b == b'\n' || *b == b'\r');
    let text = match (lead > 0 && !line_start, trail > 0 && !line_end) {
        (true, true) => " <AGO> ",
        (true, false) => " <AGO>",
        (false, true) => "<AGO> ",
        (false, false) => "<AGO>",
    };
    Some(Hit {
        len: end + trail,
        text,
    })
}

/// A 13 digit epoch in milliseconds (the listen stamps): the years 2017 to 2033.
fn epoch_ms(at: &At<'_>) -> Option<Hit> {
    let lone = !word_before(at.prev);
    let run = digits(at.rest);
    let years = at.rest.first().is_some_and(|b| *b == b'1')
        && at.rest.get(1).is_some_and(|b| (b'5'..=b'9').contains(b));
    (lone && run == 13 && years).then_some(Hit {
        len: 13,
        text: "<EPOCH_MS>",
    })
}

/// A process id written as JSON: `"pid": 4242`.
fn pid(at: &At<'_>) -> Option<Hit> {
    const KEY: &[u8] = b"\"pid\": ";
    if !at.rest.starts_with(KEY) {
        return None;
    }
    let n = digits(&at.rest[KEY.len()..]);
    (n > 0).then_some(Hit {
        len: KEY.len() + n,
        text: "\"pid\": <PID>",
    })
}

/// A generated session id (a UUID) that is not one the steps fixed.
fn uuid(at: &At<'_>) -> Option<Hit> {
    const DASHES: [usize; 4] = [8, 13, 18, 23];
    if word_before(at.prev) || at.rest.len() < 36 {
        return None;
    }
    let text = &at.rest[..36];
    let shaped = text.iter().enumerate().all(|(i, b)| {
        if DASHES.contains(&i) {
            *b == b'-'
        } else {
            b.is_ascii_hexdigit()
        }
    });
    let ends = !at.rest.get(36).is_some_and(u8::is_ascii_alphanumeric);
    let fixed_id = at.stable_ids.iter().any(|id| id.as_bytes() == text);
    (shaped && ends && !fixed_id).then_some(Hit {
        len: 36,
        text: "<UUID>",
    })
}
