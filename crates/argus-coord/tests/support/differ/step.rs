//! One command of a differential run: the arguments after `coord`, the stdin, the session id
//! and the extra environment. Both engines get the same step.

/// One call of `coord`.
#[derive(Clone, Debug, Default)]
pub struct Step {
    /// The words after `coord`.
    pub argv: Vec<String>,
    /// What the call reads on stdin (a hook's JSON); empty for none.
    pub stdin: String,
    /// `CLAUDE_CODE_SESSION_ID`; the variable is removed from the call when `None`.
    pub session: Option<String>,
    /// Extra environment variables, set after the harness's own.
    pub env: Vec<(String, String)>,
    /// A folder under the work folder to run in; the work folder itself when `None`.
    pub cwd: Option<String>,
    /// Output lines (by their start) that only the Argus side prints, because Argus is ahead of
    /// Node; they are left out of the Argus output before it is compared. See `docs/coord-port.md`.
    pub argus_ahead: Vec<String>,
}

impl Step {
    /// A step that runs `coord <args...>`.
    #[must_use]
    pub fn new(args: &[&str]) -> Step {
        Step {
            argv: args.iter().map(|a| (*a).to_owned()).collect(),
            ..Step::default()
        }
    }

    /// The same step reading `text` on stdin.
    #[must_use]
    pub fn stdin(mut self, text: &str) -> Step {
        text.clone_into(&mut self.stdin);
        self
    }

    /// The same step run as the Claude Code session `id`.
    #[must_use]
    pub fn session(mut self, id: &str) -> Step {
        self.session = Some(id.to_owned());
        self
    }

    /// The same step with one more environment variable.
    #[must_use]
    pub fn env(mut self, name: &str, value: &str) -> Step {
        self.env.push((name.to_owned(), value.to_owned()));
        self
    }

    /// The same step run in `folder` under the work folder (created when absent).
    #[must_use]
    pub fn cwd(mut self, folder: &str) -> Step {
        self.cwd = Some(folder.to_owned());
        self
    }

    /// The same step, comparing without the Argus lines that start with one of `starts`.
    #[must_use]
    pub fn argus_ahead(mut self, starts: &[&str]) -> Step {
        self.argus_ahead = starts.iter().map(|s| (*s).to_owned()).collect();
        self
    }

    /// The step as one line for a report.
    #[must_use]
    pub fn line(&self) -> String {
        format!("coord {}", self.argv.join(" "))
    }
}
