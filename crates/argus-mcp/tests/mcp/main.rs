//! The MCP layer against the fake backend and a local HTTP stub: the cases of the Node
//! `mcp.tools`, `mcp.http`, `mcp.server` (protocol side) and `mcp.stdio` tests.

mod bridge;
mod bridge_stub;
mod defs;
mod deps;
mod http;
mod node_parity;
mod node_results;
mod notify;
mod policy;
mod proof;
mod read_tools;
mod resources;
mod support;
mod write_tools;
