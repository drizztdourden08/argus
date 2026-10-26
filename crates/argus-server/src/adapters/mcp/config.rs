//! What the tools read of the live config and the machine.

use std::path::PathBuf;

use argus_core::config::{AttachConfig, McpConfig};

use crate::Services;

/// `config.mcp`, read on every call.
pub fn policy(svc: &Services) -> McpConfig {
    svc.config().mcp
}

/// `config.attach`.
pub fn attach(svc: &Services) -> AttachConfig {
    svc.config().attach
}

/// The folders a proof path may live in besides the temp folder and the attach roots: the
/// configured project roots, the roots of the registered projects and the coord home.
pub fn file_roots(svc: &Services) -> Vec<PathBuf> {
    let configured = svc.config().project_roots.into_values();
    let registered = svc
        .sync
        .iter()
        .flat_map(|s| s.projects())
        .filter_map(|p| p.root);
    configured
        .chain(registered)
        .map(PathBuf::from)
        .chain([svc.paths.coord_home.clone()])
        .collect()
}

/// The base of the urls a tool hands out when the request named no Host.
pub fn base_url(svc: &Services) -> String {
    format!("http://127.0.0.1:{}", svc.port())
}
