//! The hosted Overlord: the pure parts of the coordinator that Argus runs as a `claude -p` child
//! on the owner's API key (I-159). Nothing here spawns a process or touches the disk; the server
//! crate wires these pieces to real adapters.
//!
//! Modules land one story at a time. Add the next one (`session`, `spend`, ...) below as a plain
//! `pub mod` line.

pub mod ports;
pub mod protocol;
