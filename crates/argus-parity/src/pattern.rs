//! Path patterns: dot-separated segments over the keys and indices of a JSON value. `*` is one
//! segment, `**` is any number of segments (none included), the empty pattern is the root.

fn segments(pattern: &str) -> Vec<&str> {
    if pattern.is_empty() {
        Vec::new()
    } else {
        pattern.split('.').collect()
    }
}

fn walk(pattern: &[&str], path: &[String]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => (0..=path.len()).any(|skip| walk(rest, &path[skip..])),
        Some((head, rest)) => path
            .split_first()
            .is_some_and(|(seg, tail)| (*head == "*" || head == seg) && walk(rest, tail)),
    }
}

/// Whether `path` (the keys and indices from the root) is matched by `pattern`.
#[must_use]
pub fn matches(pattern: &str, path: &[String]) -> bool {
    walk(&segments(pattern), path)
}

#[cfg(test)]
mod tests {
    use super::matches;

    fn path(text: &str) -> Vec<String> {
        text.split('.')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn a_literal_pattern_matches_only_that_path() {
        assert!(matches("a.b", &path("a.b")));
        assert!(!matches("a.b", &path("a.b.c")));
        assert!(!matches("a.b", &path("a")));
    }

    #[test]
    fn one_star_is_one_segment() {
        assert!(matches("docs.*.data", &path("docs.7.data")));
        assert!(!matches("docs.*.data", &path("docs.data")));
    }

    #[test]
    fn two_stars_are_any_depth() {
        assert!(matches("**.at", &path("a.b.c.at")));
        assert!(matches("**.at", &path("at")));
        assert!(!matches("**.at", &path("at.b")));
    }

    #[test]
    fn the_empty_pattern_is_the_root() {
        assert!(matches("", &[]));
        assert!(!matches("", &path("a")));
    }
}
