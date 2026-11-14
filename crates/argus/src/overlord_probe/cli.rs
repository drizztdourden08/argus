//! The two one-shot calls: `claude --version` and `claude --help`.

use std::path::{Path, PathBuf};

use argus_core::process::hidden_command;
use tokio::process::Command;

use super::probe::Probe;

/// The flags the help text is checked for.
const FLAGS: [&str; 5] = [
    "--effort",
    "--max-budget-usd",
    "--permission-prompt-tool",
    "--fork-session",
    "--include-partial-messages",
];

/// The newest CLI the Desktop app carries, else `claude` from the path.
pub fn find_exe() -> PathBuf {
    argus_accounts::find_app_data(None)
        .and_then(|app| argus_accounts::find_cli_exe(&app))
        .unwrap_or_else(|| PathBuf::from("claude"))
}

async fn capture(exe: &Path, arg: &str) -> Result<String, String> {
    let mut command = Command::from(hidden_command(exe));
    command.arg(arg).kill_on_drop(true);
    let out = command
        .output()
        .await
        .map_err(|e| format!("cannot run claude {arg}: {e}"))?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok(text)
}

/// Prints the `claude --version` fact.
pub async fn version(probe: &mut Probe, exe: &Path) -> Result<(), String> {
    probe.step = "version";
    probe.fact("exe", &exe.display().to_string());
    let text = capture(exe, "--version").await?;
    probe.fact("cli-version", text.trim());
    Ok(())
}

/// Prints one fact per flag of [`FLAGS`] and the scrub or subprocess variables the help names.
pub async fn help(probe: &mut Probe, exe: &Path) -> Result<(), String> {
    probe.step = "help";
    let text = capture(exe, "--help").await?;
    for flag in FLAGS {
        let seen = if text.contains(flag) { "yes" } else { "no" };
        probe.fact(&format!("help-mentions {flag}"), seen);
    }
    let names = scrub_names(&text);
    let shown = if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    };
    probe.fact("help-env-scrub", &shown);
    Ok(())
}

/// Upper case words of the text that contain SCRUB or SUBPROCESS, sorted and unique.
fn scrub_names(text: &str) -> Vec<String> {
    let mut names: Vec<String> = text
        .split(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
        .filter(|w| w.len() > 5 && (w.contains("SCRUB") || w.contains("SUBPROCESS")))
        .map(str::to_owned)
        .collect();
    names.sort();
    names.dedup();
    names
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::scrub_names;

    #[test]
    fn scrub_names_finds_the_variables() {
        let text = "set CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1 or ENV_SCRUB, not subprocess";
        assert_eq!(
            scrub_names(text),
            vec!["CLAUDE_CODE_SUBPROCESS_ENV_SCRUB", "ENV_SCRUB"]
        );
    }
}
