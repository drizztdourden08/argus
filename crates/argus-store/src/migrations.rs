//! The six Node migrations, statement for statement. `PRAGMA user_version` counts the applied
//! ones, so Argus and Node open each other's database.

/// Every migration, in order. A new one is added at the end, never edited.
pub const MIGRATIONS: [&str; 6] = [
    include_str!("../migrations/0001.sql"),
    include_str!("../migrations/0002.sql"),
    include_str!("../migrations/0003.sql"),
    include_str!("../migrations/0004.sql"),
    include_str!("../migrations/0005.sql"),
    include_str!("../migrations/0006.sql"),
];
