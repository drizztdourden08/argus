//! The absolute paths into a board folder that boards and the database carry:
//! `C:\Users\x\.coord\<key>\proofs\a.png` and `/.coord/<key>/...`.

use std::borrow::Cow;
use std::sync::LazyLock;

use regex::{Captures, Regex};
use serde_json::Value;

use crate::map::IdMap;

static PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\.coord)([\\/])([a-z0-9][a-z0-9-]*)([\\/])").expect("the prefix pattern")
});

/// `text` with every `.coord<sep><key><sep>` prefix of a key in `map` pointing at the new name;
/// `None` when nothing changed.
pub fn rewrite_text(text: &str, map: &IdMap) -> Option<String> {
    let out = PREFIX.replace_all(text, |c: &Captures| {
        let name = map.get(&c[3]).map_or(&c[3], String::as_str);
        format!("{}{}{}{}", &c[1], &c[2], name, &c[4])
    });
    match out {
        Cow::Borrowed(_) => None,
        Cow::Owned(s) => (s != text).then_some(s),
    }
}

/// Rewrites the path prefixes in every string of `v` (not in object keys).
pub fn rewrite(v: &mut Value, map: &IdMap) {
    match v {
        Value::String(s) => {
            if let Some(new) = rewrite_text(s, map) {
                *s = new;
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|e| rewrite(e, map)),
        Value::Object(m) => m.values_mut().for_each(|e| rewrite(e, map)),
        _ => {}
    }
}
