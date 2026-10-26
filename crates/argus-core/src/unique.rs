//! The distinct items of a sequence, in the order they first appear.

/// The items of `items` without repeats, first appearance first.
#[must_use]
pub fn unique<T: PartialEq>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut seen: Vec<T> = Vec::new();
    for item in items {
        if !seen.contains(&item) {
            seen.push(item);
        }
    }
    seen
}
