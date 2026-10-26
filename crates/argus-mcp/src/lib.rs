//! The Argus MCP layer (DESIGN 7.1): the JSON-RPC protocol, the 23 tools and the resources of
//! the `coord-dashboard` server, the streamable HTTP handler the server mounts at `POST /mcp`,
//! and the stdio bridge `argus mcp` runs.
//!
//! Everything the tools read or do goes through [`McpBackend`]; the server implements it over
//! its services and the tests use [`MemoryBackend`]. The tool names, schemas, descriptions and
//! result texts are the Node server's, because Claude sessions depend on them.

#![deny(clippy::cognitive_complexity)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod agents;
mod args;
mod backend;
mod bridge;
mod call;
mod context;
mod defs;
mod error;
mod files;
mod host;
mod http;
mod ids;
mod image;
mod json;
mod launchers;
mod lists;
mod memory;
mod memory_runs;
mod outcome;
mod policy;
mod project;
mod proof;
mod read_tools;
mod resources;
mod rpc;
mod runs_view;
mod schema;
mod search;
mod summary;
mod tools;
mod write_tools;

pub use backend::{Image, LogQuery, McpBackend, NotifyRequest, RunFilter, StartOpts};
pub use bridge::{BridgeEnv, BridgeOptions, CLOSED_FROM_TRAY, Endpoint, run_bridge};
pub use context::CallContext;
pub use defs::{is_mutating, server_info, tool_def, tool_defs};
pub use error::ToolError;
pub use host::ToolHost;
pub use http::{MAX_BYTES, McpReply, McpRequest, handle_other, handle_post};
pub use memory::{Calls, MemoryBackend};
pub use outcome::ToolOutcome;
pub use rpc::{
    INSTRUCTIONS, PROTOCOL_VERSION, RpcHost, SUPPORTED_VERSIONS, code, handle_rpc, rpc_error,
};
pub use write_tools::RESULT_NOTE_REQUIRED;
