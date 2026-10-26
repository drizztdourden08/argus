//! What every verb group shares, so no two groups rebuild it: the caller as the board ops take it
//! ([`caller`]) and the Node refusal of a taken board lock ([`lock`]). WP2, WP5 and WP6 use these
//! and add nothing of the kind to their own folders.

pub mod caller;
pub mod lock;
