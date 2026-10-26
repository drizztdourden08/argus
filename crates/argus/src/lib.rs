//! The parts of the `argus` binary that tests drive: the verbs that are clients of the server, the
//! stdio MCP bridge and the verbs that write configuration for the binary.
#![deny(clippy::cognitive_complexity)]

#[cfg(windows)]
pub mod detach;
pub mod install;
pub mod mcp;
pub mod wake_stream;
