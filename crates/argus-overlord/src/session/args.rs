//! The child command line, built by one pure function.

/// The permission tool the child asks before a gated tool call.
const PERMISSION_TOOL: &str = "mcp__argus-overlord__permission_prompt";

/// What the command line is built from.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionArgs {
    pub model: String,
    /// The effort setting. Adds nothing yet: the story 4 probe decides whether it is a flag or
    /// a setting, and this field then gains its line in [`build_argv`].
    pub effort: Option<String>,
    /// A session id to resume.
    pub resume: Option<String>,
    /// With `resume`: continue in a new session instead of the old one.
    pub fork: bool,
    /// The temp JSON naming the private MCP endpoint and its bearer.
    pub mcp_config: String,
    /// What is left of the month's budget, in USD.
    pub budget_left_usd: f64,
}

/// The arguments after the program name. Never `--bare`: it would skip the hooks and the
/// `CLAUDE.md` the Overlord depends on.
#[must_use]
pub fn build_argv(config: &SessionArgs) -> Vec<String> {
    let mut argv = to_strings(&[
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--model",
        &config.model,
    ]);
    if let Some(id) = &config.resume {
        argv.extend(["--resume".to_owned(), id.clone()]);
        if config.fork {
            argv.push("--fork-session".to_owned());
        }
    }
    argv.extend(to_strings(&[
        "--permission-prompt-tool",
        PERMISSION_TOOL,
        "--mcp-config",
        &config.mcp_config,
        "--max-budget-usd",
        &format!("{:.2}", config.budget_left_usd),
    ]));
    argv
}

fn to_strings(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> SessionArgs {
        SessionArgs {
            model: "sonnet".to_owned(),
            effort: Some("medium".to_owned()),
            resume: None,
            fork: false,
            mcp_config: "C:/tmp/mcp.json".to_owned(),
            budget_left_usd: 42.5,
        }
    }

    #[test]
    fn the_argv_never_has_bare_and_names_the_permission_tool() {
        let argv = build_argv(&config());
        assert!(!argv.iter().any(|arg| arg == "--bare"));
        let at = argv
            .iter()
            .position(|arg| arg == "--permission-prompt-tool")
            .unwrap();
        assert_eq!(argv[at + 1], "mcp__argus-overlord__permission_prompt");
    }

    #[test]
    fn the_argv_carries_the_budget_and_the_mcp_config() {
        let argv = build_argv(&config());
        assert!(argv.windows(2).any(|w| w == ["--max-budget-usd", "42.50"]));
        assert!(
            argv.windows(2)
                .any(|w| w == ["--mcp-config", "C:/tmp/mcp.json"])
        );
    }

    #[test]
    fn effort_adds_nothing_yet() {
        let mut other = config();
        other.effort = None;
        assert_eq!(build_argv(&config()), build_argv(&other));
    }

    #[test]
    fn resume_adds_the_id_and_fork_adds_its_flag() {
        let mut resumed = config();
        resumed.resume = Some("abc".to_owned());
        let plain = build_argv(&resumed);
        assert!(plain.windows(2).any(|w| w == ["--resume", "abc"]));
        assert!(!plain.iter().any(|arg| arg == "--fork-session"));
        resumed.fork = true;
        assert!(
            build_argv(&resumed)
                .iter()
                .any(|arg| arg == "--fork-session")
        );
    }

    #[test]
    fn fork_without_resume_adds_nothing() {
        let mut forked = config();
        forked.fork = true;
        assert!(
            !build_argv(&forked)
                .iter()
                .any(|arg| arg == "--fork-session")
        );
    }
}
