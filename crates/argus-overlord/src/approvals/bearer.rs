//! The bearer check of the private endpoint.

/// True when `header` (the `Authorization` value) is `Bearer <expected>`. No expected bearer (no
/// child is running) never matches. The compare does not stop at the first different byte.
#[must_use]
pub fn bearer_ok(header: Option<&str>, expected: Option<&str>) -> bool {
    let (Some(header), Some(expected)) = (header, expected) else {
        return false;
    };
    let Some(given) = header.strip_prefix("Bearer ") else {
        return false;
    };
    let (a, b) = (given.as_bytes(), expected.as_bytes());
    if expected.is_empty() || a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
