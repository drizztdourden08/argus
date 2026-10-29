//! The path segments of the cache: what a folder name may be, and how a package name is spelled in one.

/// A segment matches `[A-Za-z0-9_.+@-]+` and is not a run of dots (`.` and `..` are never a name).
pub fn is_safe(segment: &str) -> bool {
    !segment.is_empty()
        && segment.chars().any(|c| c != '.')
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.+@-".contains(c))
}

/// The package name of a name folder: the scope slash is a `+` there.
pub fn decode_name(segment: &str) -> String {
    segment.replace('+', "/")
}

/// A child path, forward slashes.
pub fn join(base: &str, child: &str) -> String {
    format!("{}/{child}", base.trim_end_matches('/'))
}

/// The folder a path sits in.
pub fn parent(path: &str) -> Option<&str> {
    path.rsplit_once('/').map(|(head, _)| head)
}
