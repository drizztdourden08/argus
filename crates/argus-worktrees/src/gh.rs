//! The GitHub CLI, for one read-only question: is this branch's pull request merged?

use crate::exec::{GitOutput, Spec, run_command};
use std::time::Duration;

/// How long `gh` may take.
pub const GH_TIMEOUT: Duration = Duration::from_secs(5);

/// Runs `gh`. Never required: a missing or failing `gh` is a result with no exit code, read as "not merged".
pub trait GhRunner: Send + Sync {
    fn run(&self, cwd: &str, args: &[String]) -> GitOutput;
}

/// The real `gh`, and only `gh pr list`.
pub struct SystemGh;

impl GhRunner for SystemGh {
    fn run(&self, cwd: &str, args: &[String]) -> GitOutput {
        if !(args.first().map(String::as_str) == Some("pr")
            && args.get(1).map(String::as_str) == Some("list"))
        {
            return GitOutput::refused("only gh pr list may run here");
        }
        let env = [
            ("GH_PROMPT_DISABLED", "1"),
            ("GIT_TERMINAL_PROMPT", "0"),
            ("NO_COLOR", "1"),
        ];
        run_command(&Spec {
            program: "gh",
            args,
            cwd,
            env: &env,
            timeout: GH_TIMEOUT,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_runner_only_ever_runs_gh_pr_list() {
        let args: Vec<String> = ["repo", "delete", "x"].map(String::from).to_vec();
        let out = SystemGh.run(".", &args);
        assert_eq!(out.code, None);
        assert!(out.error.unwrap().contains("only gh pr list"));
    }
}
