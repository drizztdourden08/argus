//! The sentence every write of a build uses when a non-master build has no item to serve.

/// Why a build without an item is refused: only the master build goes without one.
pub const NEEDS_LINK: &str =
    "a build needs the item it serves: pass --for I-n. Only the master build goes without one.";
