//! What a launcher's command line is, exactly: ONE line, handed to the platform shell,
//! from the launcher's folder, with no console window and no terminal. cmd.exe is not bash
//! and not PowerShell, so habits from another shell get a warning (never a refusal).

use std::sync::LazyLock;

use regex::Regex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Other,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Other
        }
    }
}

/// `cmd.exe /d /s /c` on Windows, `sh -c` elsewhere.
pub fn shell_of(platform: Platform) -> &'static str {
    match platform {
        Platform::Windows => "cmd.exe /d /s /c",
        Platform::Other => "sh -c",
    }
}

pub fn runs_note(platform: Platform) -> String {
    format!(
        "Play runs the command line with {} from the launcher's folder (--cwd), hidden: no console window, output goes to the dashboard's Terminal, stdin is a pipe (nothing can ask a question), PATH is the dashboard's.",
        shell_of(platform)
    )
}

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}

static DOLLAR: LazyLock<Regex> = LazyLock::new(|| re(r"\$env:|\$\{?[A-Za-z_]"));
static EXPORT: LazyLock<Regex> = LazyLock::new(|| re(r"(^|&&|\|\||&|;)\s*(export|source)\s"));
static SINGLE: LazyLock<Regex> = LazyLock::new(|| re(r"(^|\s)'[^']*'"));
static QUOTED: LazyLock<Regex> = LazyLock::new(|| re(r#""[^"]*""#));
static SEMICOLON: LazyLock<Regex> = LazyLock::new(|| re(r";\s*\S"));
static CD: LazyLock<Regex> = LazyLock::new(|| re(r"(^|&&|\|\||&|;)\s*cd\s"));
static BASH: LazyLock<Regex> = LazyLock::new(|| re(r"(^|\s)\./\S+\.sh\b|\bbash\s+-c\b"));
static PERCENT: LazyLock<Regex> = LazyLock::new(|| re(r"(^|\s)%[A-Za-z_]+%"));

/// Habits from another shell that do not mean the same thing here. The command is stored
/// as given.
pub fn command_warnings(cmd: &str, platform: Platform) -> Vec<&'static str> {
    if platform == Platform::Other {
        return if PERCENT.is_match(cmd) {
            vec!["sh does not expand %NAME%: write $NAME."]
        } else {
            Vec::new()
        };
    }
    let unquoted = QUOTED.replace_all(cmd, "");
    [
        (DOLLAR.is_match(cmd), "cmd.exe does not expand $VAR or $env:NAME: write %NAME%."),
        (EXPORT.is_match(cmd), "export and source are bash: in cmd.exe use set NAME=value && command."),
        (SINGLE.is_match(cmd), "single quotes do not quote in cmd.exe: use double quotes."),
        (SEMICOLON.is_match(&unquoted), "\";\" does not chain commands in cmd.exe: use && (or &)."),
        (CD.is_match(cmd), "do not cd: set the folder with --cwd."),
        (BASH.is_match(cmd), "this is not a bash shell: .sh scripts need bash on PATH and are not what the owner has by default."),
    ]
    .into_iter()
    .filter_map(|(hit, warning)| hit.then_some(warning))
    .collect()
}
