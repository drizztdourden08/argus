//! A small reader of `pnpm-lock.yaml`: the exact versions the `packages:` section resolves for some names. It reads
//! the keys only (`name@version`, with the peer suffix cut), for lockfile versions 5, 6 and 9, and needs no YAML model.

use std::collections::BTreeSet;

/// A top-level line such as `packages:`.
fn section_of(line: &str) -> Option<&str> {
    let first = line.chars().next()?;
    if first == ' ' || first == '#' || first == '-' {
        return None;
    }
    line.split(':').next()
}

/// The key of a two-space line: quoted or plain, up to the colon that ends it.
fn key_of(line: &str) -> Option<&str> {
    let body = line.strip_prefix("  ")?;
    if body.starts_with(' ') || body.starts_with('#') || body.is_empty() {
        return None;
    }
    if let Some(quoted) = body.strip_prefix('\'').or_else(|| body.strip_prefix('"')) {
        return quoted.split(['\'', '"']).next();
    }
    let end = body
        .find(": ")
        .or_else(|| body.strip_suffix(':').map(str::len))?;
    Some(&body[..end])
}

/// `name/1.2.3_peer` (v5): the version is after the last slash and starts with a digit.
fn split_v5(bare: &str) -> Option<(&str, &str)> {
    let (name, tail) = bare.rsplit_once('/')?;
    let version = tail.split('_').next().unwrap_or(tail);
    let digit = version.starts_with(|c: char| c.is_ascii_digit());
    (digit && !version.contains('@')).then_some((name, version))
}

/// `(name, version)` of a key: `/@s/n@1.0.0(peer)` (v6, v9) or `/@s/n/1.0.0_peer` (v5).
fn split_key(key: &str) -> Option<(&str, &str)> {
    let bare = key.strip_prefix('/').unwrap_or(key);
    let (name, version) = split_v5(bare).or_else(|| {
        let plain = bare.split('(').next().unwrap_or(bare);
        let at = plain.rfind('@').filter(|i| *i > 0)?;
        Some((&plain[..at], &plain[at + 1..]))
    })?;
    let plain = !version.is_empty() && !version.contains([':', '/']);
    (plain && !name.is_empty()).then_some((name, version))
}

/// The `(name, version)` pairs of the `packages:` section whose name is in `names`.
pub fn resolved(text: &str, names: &BTreeSet<String>) -> BTreeSet<(String, String)> {
    let mut found = BTreeSet::new();
    let mut in_packages = false;
    for line in text.lines() {
        if let Some(section) = section_of(line) {
            in_packages = section == "packages";
        } else if in_packages
            && let Some((name, version)) = key_of(line).and_then(split_key)
            && names.contains(name)
        {
            found.insert((name.to_owned(), version.to_owned()));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|n| (*n).to_owned()).collect()
    }

    fn pairs(text: &str, list: &[&str]) -> Vec<(String, String)> {
        resolved(text, &names(list)).into_iter().collect()
    }

    fn pair(name: &str, version: &str) -> (String, String) {
        (name.to_owned(), version.to_owned())
    }

    #[test]
    fn v9_keys_lose_their_peer_suffix_and_only_the_packages_section_counts() {
        let text = "lockfileVersion: '9.0'\nimporters:\n  .:\n    dependencies:\n      electron:\n        version: 41.0.0\npackages:\n  electron@42.11.8:\n    resolution: {integrity: x}\n  '@s/n@1.2.3(react@19.0.0)':\n    resolution: {integrity: y}\nsnapshots:\n  electron@40.0.0: {}\n";
        assert_eq!(
            pairs(text, &["electron", "@s/n"]),
            vec![pair("@s/n", "1.2.3"), pair("electron", "42.11.8")]
        );
    }

    #[test]
    fn v6_keys_start_with_a_slash() {
        let text = "lockfileVersion: '6.0'\npackages:\n\n  /@s/n@1.2.3(react@18.0.0):\n    dev: false\n  /electron@42.11.8:\n    dev: true\n";
        assert_eq!(
            pairs(text, &["electron", "@s/n"]),
            vec![pair("@s/n", "1.2.3"), pair("electron", "42.11.8")]
        );
    }

    #[test]
    fn v5_keys_put_the_version_after_a_slash() {
        let text = "lockfileVersion: 5.4\npackages:\n  /@s/n/1.2.3_react@18.0.0:\n    dev: false\n  /electron/42.11.8:\n    dev: true\n";
        assert_eq!(
            pairs(text, &["electron", "@s/n"]),
            vec![pair("@s/n", "1.2.3"), pair("electron", "42.11.8")]
        );
    }

    #[test]
    fn names_that_are_not_cached_and_links_to_folders_are_ignored() {
        let text = "packages:\n  left-pad@1.0.0:\n    x: 1\n  electron@file:../electron:\n    x: 1\n  'electron@https://x.test/e.tgz':\n    x: 1\n";
        assert_eq!(pairs(text, &["electron"]), Vec::new());
    }
}
