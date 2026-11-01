//! The rules the monitor seeds, and the protection no rule can lift.

use crate::normalize::normalize_rule;
use crate::rule::Rule;
use serde_json::{Value, json};

/// The processes the monitor never kills, by executable name (lower case, no `.exe`).
pub const PROTECTED_NAMES: [&str; 22] = [
    "system",
    "idle",
    "registry",
    "secure system",
    "memory compression",
    "smss",
    "csrss",
    "wininit",
    "winlogon",
    "services",
    "lsass",
    "lsaiso",
    "svchost",
    "explorer",
    "dwm",
    "fontdrvhost",
    "sihost",
    "claude",
    "init",
    "systemd",
    "launchd",
    "kthreadd",
];

/// And by command line: the dashboard itself and its helpers, Claude Code sessions.
pub const PROTECTED_CMD: [&str; 6] = [
    r"(?i)dashboard[.]mjs",
    r"(?i)tray[.]ps1",
    r"(?i)\bhook\s+listen\b",
    r"(?i)\bwake-stream\b",
    r"(?i)claude-code",
    r"(?i)@anthropic-ai/claude",
];

/// The Rust dashboard by command line: `argus serve`, `argus tray` and the `argus mcp` bridges,
/// wherever the binary lives (an installed copy or a worktree's `target`).
pub const ARGUS_CMD: &str = r#"(?i)(?:^|[\s/"'])argus(?:[.]exe)?["']?\s+(?:serve|tray|mcp)\b"#;

fn dashboard_ignore() -> Value {
    json!([
        "dashboard[.]mjs",
        r"hook\s+listen",
        "wake-stream",
        "tray[.]ps1"
    ])
}

fn claude_rules() -> Vec<Value> {
    vec![
        json!({
            "name": "claude",
            "match": {"name": "^claude([.]exe)?$"},
            "maxMemMB": 4000, "maxGroupMemMB": 8000, "killable": false,
            "note": "Claude Desktop, its helpers and the native Claude Code CLI: protected, listed so their memory shows; over the limit, tell the owner."
        }),
        json!({
            "name": "claude-code-node",
            "match": {"name": "^node([.]exe)?$", "cmd": "claude-code|@anthropic-ai/claude"},
            "maxMemMB": 3000, "killable": false,
            "note": "Claude Code sessions run on node: protected, listed only."
        }),
    ]
}

fn node_rules() -> Vec<Value> {
    vec![
        json!({
            "name": "electron-dev",
            "match": {"name": "^electron([.]exe)?$", "cmd": "x:/"},
            "maxMemMB": 1500, "maxGroupMemMB": 6000, "maxAgeMin": 720, "idleMin": 120,
            "orphan": true, "hung": true, "killable": true,
            "note": "Dev builds of the projects (electron under X:/): a renderer over 1.5 GB, a build left running 12 h or idle 2 h, an orphan or a hung window is a leftover."
        }),
        json!({
            "name": "node-dev-server",
            "match": {
                "name": "^node([.]exe)?$",
                "cmd": r"(?<![.])\b(vite|electron-vite|storylite|pnpm|tsx|webpack|nodemon)\b|next\s+dev|astro\s+dev"
            },
            "ignore": dashboard_ignore(),
            "maxMemMB": 1500, "maxGroupMemMB": 4000, "maxAgeMin": 720, "idleMin": 180,
            "orphan": true, "killable": true,
            "note": "Dev servers and watchers (vite, storylite, pnpm, tsx): idle 3 h, older than 12 h or orphaned means nobody uses it."
        }),
        json!({
            "name": "build-helpers",
            "match": {"name": "^(esbuild|tsserver|rg|git)([.]exe)?$"},
            "onlyFlagged": true, "maxMemMB": 1000, "orphan": true, "killable": true,
            "note": "Helpers a build or an editor starts (esbuild, tsserver, rg, git): listed only when orphaned or over 1 GB."
        }),
        json!({
            "name": "node-other",
            "match": {"name": "^node([.]exe)?$"},
            "ignore": dashboard_ignore(),
            "maxMemMB": 1500, "maxGroupMemMB": 6000, "orphan": true, "killable": true,
            "note": "Any other node (MCP servers, scripts, tools): over 1.5 GB, or orphaned (its session died and left it running)."
        }),
    ]
}

fn other_rules() -> Vec<Value> {
    vec![
        json!({
            "name": "python",
            "match": {"name": "^(python[0-9.]*w?|py|pythonw)([.]exe)?$"},
            "maxMemMB": 2000, "maxAgeMin": 720, "idleMin": 360, "orphan": true, "killable": true,
            "note": "Python engines and servers: over 2 GB, older than 12 h, idle 6 h or orphaned."
        }),
        json!({
            "name": "game-engine",
            "match": {"name": "^godot"},
            "maxMemMB": 2500, "maxAgeMin": 720, "hung": true, "orphan": true, "killable": true,
            "note": "Godot (editor and games): the editor is the owner's app, ask before killing it; a hung or orphaned game run is a leftover."
        }),
        json!({
            "name": "project-build",
            "match": {"path": "^x:/"},
            "ignore": dashboard_ignore(),
            "maxMemMB": 2000, "maxAgeMin": 720, "hung": true, "killable": true,
            "note": "Any other program started from X:/ (a packaged build of a project): over 2 GB, older than 12 h or hung."
        }),
        json!({
            "name": "shell-leftovers",
            "match": {"name": "^(powershell|pwsh|cmd)([.]exe)?$"},
            "ignore": dashboard_ignore(),
            "onlyFlagged": true, "maxMemMB": 500, "orphan": true, "killable": true,
            "note": "PowerShell and cmd shells: listed only when orphaned (whatever started them is gone) or over 500 MB."
        }),
        json!({
            "name": "catch-all",
            "match": {},
            "minMemMB": 2000, "maxMemMB": 2000, "killable": true,
            "note": "Anything else over 2 GB. Killable, but never a protected or Windows process; an app of the owner needs the owner's word."
        }),
    ]
}

/// The default rules as JSON, in the order the first match wins.
pub(crate) fn default_rules_json() -> Vec<Value> {
    let mut all = claude_rules();
    all.extend(node_rules());
    all.extend(other_rules());
    all
}

/// The default rules, validated.
#[must_use]
pub fn default_rules() -> Vec<Rule> {
    default_rules_json()
        .iter()
        .filter_map(|r| normalize_rule(r).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_rule_is_valid_and_the_catch_all_is_last() {
        let json = default_rules_json();
        let rules = default_rules();
        assert_eq!(rules.len(), json.len());
        let names: std::collections::HashSet<_> = rules.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names.len(), rules.len());
        assert!(rules.last().unwrap().matcher.is_empty());
    }

    #[test]
    fn the_defaults_are_the_node_defaults_in_the_node_order() {
        let names: Vec<_> = default_rules().into_iter().map(|r| r.name).collect();
        assert_eq!(
            names,
            [
                "claude",
                "claude-code-node",
                "electron-dev",
                "node-dev-server",
                "build-helpers",
                "node-other",
                "python",
                "game-engine",
                "project-build",
                "shell-leftovers",
                "catch-all"
            ]
        );
    }

    #[test]
    fn a_dev_server_may_use_3_gb_before_it_is_flagged() {
        let rules = default_rules();
        let dev = rules.iter().find(|r| r.name == "node-dev-server").unwrap();
        assert_eq!(dev.max_mem_mb, Some(1500.0));
    }
}
