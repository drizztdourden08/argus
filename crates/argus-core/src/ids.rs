//! Project keys and scopes.

/// The portfolio scope (the Argus overview), also its folder under `COORD_HOME`.
pub const PORTFOLIO: &str = "_portfolio";

/// The project whose lead is the Overlord (`coord lead portfolio`).
pub const OVERLORD_PROJECT: &str = "portfolio";

/// `^[a-z0-9][a-z0-9-]*$`: a project key.
#[must_use]
pub fn is_key(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Names a board id may never take: the folders and scopes under `COORD_HOME` that are not boards.
pub const RESERVED: [&str; 9] = [
    "argus",
    "argus-app",
    "overlord",
    "dashboard",
    PORTFOLIO,
    "locks",
    "ci",
    "releases",
    "localpkg",
];

/// Whether `s` is a reserved name.
#[must_use]
pub fn is_reserved(s: &str) -> bool {
    RESERVED.contains(&s)
}

/// A project key or `_portfolio`.
#[must_use]
pub fn is_scope(s: &str) -> bool {
    s == PORTFOLIO || is_key(s)
}
