//! Normalising: the one place that says which differences between two runs are not differences.
//! Everything else is compared byte for byte. The rules, in the order they are tried:
//!
//! | Rule | Matches | Becomes |
//! |---|---|---|
//! | home | the root folder of either twin home, plain, with `/` and JSON escaped | `<HOME>` |
//! | iso time | `2026-10-26T15:49:13.648Z` (any fraction, or none) | `<TS>` |
//! | file stamp | `20261017T193800Z` (backup and stamp file names) | `<STAMP>` |
//! | message id | `m-<12+ digits>-` and, when the tail is 4 characters, the tail | `m-<MS>-`, `m-<MS>-<RND>` |
//! | relative time | `12s ago`, `5m ago`, `3h ago`, `2d ago`, with the spaces that pad its column | `<AGO>` |
//! | epoch ms | a 13 digit number from 2017 to 2033 on its own | `<EPOCH_MS>` |
//! | pid | `"pid": 4242` | `"pid": <PID>` |
//! | generated session id | a UUID that is not one a step fixed with `Step::session` | `<UUID>` |
//!
//! A session id a step fixed is the same on both sides by construction, so it stays visible:
//! a verb that stores or prints the wrong one still differs. Paths are not touched but for
//! the home, so a separator or a case that differs is a difference.

mod rules;

use std::path::Path;

use rules::{At, RULES};

/// Applies the rules to text and to file names.
#[derive(Clone, Debug)]
pub struct Normaliser {
    homes: Vec<String>,
    stable_ids: Vec<String>,
}

impl Normaliser {
    /// A normaliser for twin homes at `roots`; `stable_ids` are the session ids the steps fixed.
    #[must_use]
    pub fn new(roots: &[&Path], stable_ids: Vec<String>) -> Self {
        let mut homes = Vec::new();
        for root in roots {
            let plain = root.to_string_lossy().into_owned();
            homes.push(plain.replace('\\', "\\\\"));
            homes.push(plain.replace('\\', "/"));
            homes.push(plain);
        }
        Normaliser { homes, stable_ids }
    }

    /// The text with every rule applied.
    #[must_use]
    pub fn text(&self, text: &str) -> String {
        let mut text = text.to_owned();
        for home in &self.homes {
            text = text.replace(home.as_str(), "<HOME>");
        }
        self.scan(&text)
    }

    /// Bytes: text when they are UTF-8 (lossy otherwise), normalised.
    #[must_use]
    pub fn bytes(&self, bytes: &[u8]) -> String {
        self.text(&String::from_utf8_lossy(bytes))
    }

    fn scan(&self, text: &str) -> String {
        let bytes = text.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            let at = At {
                prev: i.checked_sub(1).map(|p| bytes[p]),
                rest: &bytes[i..],
                stable_ids: &self.stable_ids,
            };
            if let Some(hit) = RULES.iter().find_map(|rule| rule(&at)) {
                out.extend_from_slice(hit.text.as_bytes());
                i += hit.len;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        String::from_utf8(out).expect("rules replace whole ASCII runs")
    }
}
