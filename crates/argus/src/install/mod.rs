//! The verbs that write configuration for the Argus binary: `install-mcp` (the Claude Desktop
//! and Claude Code entries of DESIGN 7.3 and 7.4) and `autostart` (DESIGN 7.5). They know the
//! machine only through [`Env`], so tests run them on temp folders and never touch a real
//! Claude config or Startup folder.

pub mod autostart;
pub mod code;
pub mod desktop;
mod env;
pub mod mcp_config;
pub mod vbs;

pub use env::{Env, Io, Platform};
