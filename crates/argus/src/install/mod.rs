//! The verbs that write configuration for the Argus binary: `self-install` (the stable copy),
//! `install-mcp` (the Claude Desktop and Claude Code entries of DESIGN 7.3 and 7.4) and
//! `autostart` (DESIGN 7.5). They know the machine only through [`Env`], so tests run them on
//! temp folders and never touch a real Claude config, Startup folder or program folder.

pub mod autostart;
pub mod code;
pub mod desktop;
mod env;
pub mod mcp_config;
mod replace;
pub mod self_install;
pub mod vbs;

pub use env::{Env, Io, Platform};
