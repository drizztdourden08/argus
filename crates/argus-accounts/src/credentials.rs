//! The CLI logins in the account folders. The access token is read from
//! `<cliDir>/.credentials.json`; a refresh asks the CLI to refresh its own login. No token is
//! ever put in an error text.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::process::hidden_command;
use serde_json::Value;

use crate::ports::Credentials;
use crate::registry::find_cli_exe;

const FILE: &str = ".credentials.json";

/// What the credentials file holds that the poller needs.
struct Login {
    token: String,
    expires_at: Option<i64>,
}

fn read_login(cli_dir: &Path) -> Result<Login, String> {
    let text = fs::read_to_string(cli_dir.join(FILE))
        .map_err(|e| format!("no CLI login file in the account folder: {}", e.kind()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|_| "the CLI login file is not JSON".to_owned())?;
    let oauth = value.get("claudeAiOauth");
    let token = oauth
        .and_then(|o| o.get("accessToken"))
        .and_then(Value::as_str)
        .filter(|t| !t.is_empty())
        .ok_or_else(|| "the CLI login file has no access token".to_owned())?;
    Ok(Login {
        token: token.to_owned(),
        expires_at: oauth
            .and_then(|o| o.get("expiresAt"))
            .and_then(Value::as_i64),
    })
}

/// The CLI logins, refreshed by running the CLI that the Desktop app carries.
pub struct CliCredentials {
    app: Option<PathBuf>,
}

impl CliCredentials {
    /// Logins refreshed with the CLI under the Desktop app folder `app`, when there is one.
    #[must_use]
    pub fn new(app: Option<PathBuf>) -> Self {
        CliCredentials { app }
    }

    fn ask_cli(&self, cli_dir: &Path) -> Result<(), String> {
        let exe = self
            .app
            .as_deref()
            .and_then(find_cli_exe)
            .ok_or_else(|| "the Claude CLI was not found to refresh the login".to_owned())?;
        hidden_command(exe)
            .args(["auth", "status", "--json"])
            .env("CLAUDE_CONFIG_DIR", cli_dir)
            .output()
            .map(|_| ())
            .map_err(|e| format!("the Claude CLI did not run: {}", e.kind()))
    }
}

impl Credentials for CliCredentials {
    fn access_token(&self, cli_dir: &Path) -> Result<String, String> {
        read_login(cli_dir).map(|l| l.token)
    }

    fn expires_at(&self, cli_dir: &Path) -> Option<i64> {
        read_login(cli_dir).ok().and_then(|l| l.expires_at)
    }

    fn refresh(&self, cli_dir: &Path) -> Result<String, String> {
        let before = read_login(cli_dir)?.token;
        self.ask_cli(cli_dir)?;
        let after = read_login(cli_dir)?.token;
        if after == before {
            return Err(
                "the CLI login did not refresh: it needs a fresh `claude auth login`".to_owned(),
            );
        }
        Ok(after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder_with(json: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(FILE), json).unwrap();
        dir
    }

    #[test]
    fn reads_the_token_and_expiry() {
        let dir =
            folder_with(r#"{"claudeAiOauth":{"accessToken":"t0k","expiresAt":1790000000000}}"#);
        let creds = CliCredentials::new(None);
        assert_eq!(creds.access_token(dir.path()).unwrap(), "t0k");
        assert_eq!(creds.expires_at(dir.path()), Some(1_790_000_000_000));
    }

    #[test]
    fn errors_never_carry_the_token() {
        let dir = folder_with(r#"{"claudeAiOauth":{"accessToken":"secret-value"}}"#);
        let creds = CliCredentials::new(None);
        let err = creds.refresh(dir.path()).unwrap_err();
        assert!(!err.contains("secret-value"));
    }

    #[test]
    fn a_missing_file_or_token_is_an_error() {
        let empty = tempfile::tempdir().unwrap();
        assert!(read_login(empty.path()).is_err());
        let none = folder_with(r#"{"claudeAiOauth":{}}"#);
        assert!(read_login(none.path()).is_err());
    }
}
