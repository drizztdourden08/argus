//! Version order of the release store: semver, so `1.10.0` is newer than `1.9.0` and a
//! prerelease is older than its release. A name that is not a version sorts below every version.
// test-tag: critical

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Ident {
    Num(u64),
    Text(String),
}

#[derive(Debug, PartialEq, Eq)]
struct Parsed {
    core: [u64; 3],
    pre: Vec<Ident>,
}

fn ident(s: &str) -> Ident {
    s.parse()
        .map_or_else(|_| Ident::Text(s.to_owned()), Ident::Num)
}

fn without_build(version: &str) -> &str {
    let v = version.strip_prefix('v').unwrap_or(version);
    v.split('+').next().unwrap_or(v)
}

fn parse(version: &str) -> Option<Parsed> {
    let (core, pre) = match without_build(version).split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (without_build(version), None),
    };
    let nums: Vec<u64> = core
        .split('.')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let [a, b, c] = nums.as_slice() else {
        return None;
    };
    Some(Parsed {
        core: [*a, *b, *c],
        pre: pre.map_or_else(Vec::new, |p| p.split('.').map(ident).collect()),
    })
}

fn cmp_parsed(a: &Parsed, b: &Parsed) -> Ordering {
    a.core
        .cmp(&b.core)
        .then_with(|| match (a.pre.is_empty(), b.pre.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => a.pre.cmp(&b.pre),
        })
}

/// Orders two versions, oldest first.
#[must_use]
pub fn compare(a: &str, b: &str) -> Ordering {
    match (parse(a), parse(b)) {
        (Some(x), Some(y)) => cmp_parsed(&x, &y),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => a.cmp(b),
    }
}

/// Whether the version has a prerelease part (`1.2.0-beta.1`).
#[must_use]
pub fn is_prerelease(version: &str) -> bool {
    without_build(version).contains('-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_order_as_semver() {
        let mut v = vec![
            "1.9.0",
            "1.10.0",
            "1.10.0-beta.2",
            "1.10.0-beta.10",
            "0.9.9",
            "junk",
        ];
        v.sort_by(|a, b| compare(a, b));
        assert_eq!(
            v,
            [
                "junk",
                "0.9.9",
                "1.9.0",
                "1.10.0-beta.2",
                "1.10.0-beta.10",
                "1.10.0"
            ]
        );
    }

    #[test]
    fn a_prerelease_is_told_by_its_dash_and_a_build_suffix_is_ignored() {
        assert!(is_prerelease("1.2.0-rc.1"));
        assert!(!is_prerelease("1.2.0"));
        assert!(!is_prerelease("1.2.0+build-5"));
        assert_eq!(compare("v1.2.0+a", "1.2.0+b"), Ordering::Equal);
    }
}
