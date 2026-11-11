//! The CLI logins in the account folders. The access token is read from
//! `<cliDir>/.credentials.json`. A refresh first asks the CLI to refresh its own login; when the
//! token did not change, the OAuth `refresh_token` grant is done here and the new tokens are
//! written back into the file. A login is renewed only when its token is expired or the usage
//! endpoint answered 401, never ahead of time, so the CLI keeps the lead on its own login. No
//! token is ever put in an error text.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use argus_core::clock::now_ms;
use argus_core::process::hidden_command;
use serde_json::{Value, json};

use crate::catalog::read::write_atomic;
use crate::ports::Credentials;
use crate::registry::find_cli_exe;

const FILE: &str = ".credentials.json";
const TOKEN_URL: &str = "https://platform.claude.com/v1/oauth/token";
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const SCOPE: &str = "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload user:plugins";
const TIMEOUT: Duration = Duration::from_secs(20);

/// What the credentials file holds that the poller needs.
struct Login {
    token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
}

fn read_login(cli_dir: &Path) -> Result<Login, String> {
    let text = fs::read_to_string(cli_dir.join(FILE))
        .map_err(|e| format!("no CLI login file in the account folder: {}", e.kind()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|_| "the CLI login file is not JSON".to_owned())?;
    let oauth = value.get("claudeAiOauth");
    let text_of = |key: &str| {
        oauth
            .and_then(|o| o.get(key))
            .and_then(Value::as_str)
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
    };
    Ok(Login {
        token: text_of("accessToken")
            .ok_or_else(|| "the CLI login file has no access token".to_owned())?,
        refresh_token: text_of("refreshToken"),
        expires_at: oauth
            .and_then(|o| o.get("expiresAt"))
            .and_then(Value::as_i64),
    })
}

/// The answer of a successful `refresh_token` grant.
pub struct Grant {
    /// The new access token.
    pub access_token: String,
    /// The new refresh token, when the endpoint rotated it.
    pub refresh_token: Option<String>,
    /// Seconds the access token lives.
    pub expires_in: i64,
}

/// The OAuth token endpoint, behind a seam so tests answer without a network.
pub trait TokenEndpoint: Send + Sync {
    /// Trades a refresh token for new tokens.
    ///
    /// # Errors
    /// Why the grant failed; the text never contains a token.
    fn exchange(&self, refresh_token: &str) -> Result<Grant, String>;
}

/// The real endpoint: one JSON POST with a 20 second timeout.
pub struct HttpEndpoint;

impl TokenEndpoint for HttpEndpoint {
    fn exchange(&self, refresh_token: &str) -> Result<Grant, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .map_err(|_| "the token request could not be set up".to_owned())?;
        let response = client
            .post(TOKEN_URL)
            .json(&json!({
                "grant_type": "refresh_token",
                "refresh_token": refresh_token,
                "client_id": CLIENT_ID,
                "scope": SCOPE,
            }))
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    "the token endpoint did not answer in time".to_owned()
                } else {
                    "the token endpoint could not be reached".to_owned()
                }
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("the token endpoint answered {}", status.as_u16()));
        }
        let body: Value = response
            .json()
            .map_err(|_| "the token endpoint answered something unreadable".to_owned())?;
        grant_of(&body)
    }
}

fn grant_of(body: &Value) -> Result<Grant, String> {
    let text = |key: &str| {
        body.get(key)
            .and_then(Value::as_str)
            .filter(|t| !t.is_empty())
    };
    Ok(Grant {
        access_token: text("access_token")
            .ok_or_else(|| "the token endpoint sent no access token".to_owned())?
            .to_owned(),
        refresh_token: text("refresh_token").map(str::to_owned),
        expires_in: body
            .get("expires_in")
            .and_then(Value::as_i64)
            .ok_or_else(|| "the token endpoint sent no lifetime".to_owned())?,
    })
}

/// Writes a grant into the login file: `accessToken`, `refreshToken` (when sent) and
/// `expiresAt`. Every other field and the order of the keys stay. The file is read again first:
/// when its `refreshToken` is no longer `started_with` (the refresh token the exchange began
/// with) the CLI renewed in the meantime, and nothing is written.
fn store_grant(
    cli_dir: &Path,
    grant: &Grant,
    started_with: &str,
    now_ms: i64,
) -> Result<(), String> {
    let path = cli_dir.join(FILE);
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("no CLI login file in the account folder: {}", e.kind()))?;
    let mut value: Value =
        serde_json::from_str(&text).map_err(|_| "the CLI login file is not JSON".to_owned())?;
    let oauth = value
        .get_mut("claudeAiOauth")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "the CLI login file has no login".to_owned())?;
    if oauth.get("refreshToken").and_then(Value::as_str) != Some(started_with) {
        return Err("the CLI login changed during the renewal; nothing was written".to_owned());
    }
    oauth.insert("accessToken".to_owned(), json!(grant.access_token));
    if let Some(refresh) = &grant.refresh_token {
        oauth.insert("refreshToken".to_owned(), json!(refresh));
    }
    let expires_at = now_ms.saturating_add(grant.expires_in.saturating_mul(1000));
    oauth.insert("expiresAt".to_owned(), json!(expires_at));
    let bytes = if text.trim_end().contains('\n') {
        serde_json::to_vec_pretty(&value)
    } else {
        serde_json::to_vec(&value)
    }
    .map_err(|_| "the CLI login file could not be written".to_owned())?;
    write_atomic(&path, &bytes)
}

/// Renews the login with the refresh token and returns the new access token.
fn renew(cli_dir: &Path, endpoint: &dyn TokenEndpoint, now_ms: i64) -> Result<String, String> {
    let refresh = read_login(cli_dir)?
        .refresh_token
        .ok_or_else(|| "the CLI login has no refresh token".to_owned())?;
    let grant = endpoint.exchange(&refresh)?;
    store_grant(cli_dir, &grant, &refresh, now_ms)?;
    Ok(grant.access_token)
}

/// The CLI logins, refreshed by running the CLI that the Desktop app carries and, when that
/// changes nothing, by the OAuth refresh grant.
pub struct CliCredentials {
    app: Option<PathBuf>,
    endpoint: Box<dyn TokenEndpoint>,
}

impl CliCredentials {
    /// Logins refreshed with the CLI under the Desktop app folder `app`, when there is one.
    #[must_use]
    pub fn new(app: Option<PathBuf>) -> Self {
        Self::with_endpoint(app, Box::new(HttpEndpoint))
    }

    /// [`CliCredentials::new`] with another token endpoint.
    #[must_use]
    pub fn with_endpoint(app: Option<PathBuf>, endpoint: Box<dyn TokenEndpoint>) -> Self {
        CliCredentials { app, endpoint }
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

    fn refresh_at(&self, cli_dir: &Path, now_ms: i64) -> Result<String, String> {
        let before = read_login(cli_dir)?;
        let asked = self.ask_cli(cli_dir);
        let after = read_login(cli_dir)?;
        if after.token != before.token {
            return Ok(after.token);
        }
        if after.refresh_token.is_some() {
            return renew(cli_dir, self.endpoint.as_ref(), now_ms);
        }
        asked.and(Err(
            "the CLI login did not refresh: it needs a fresh `claude auth login`".to_owned(),
        ))
    }

    /// The access token as the file holds it. Only an expired token is renewed first (through
    /// [`Self::refresh_at`], which asks the CLI before the grant); a renewal that fails leaves
    /// the token in the file.
    fn token_at(&self, cli_dir: &Path, now_ms: i64) -> Result<String, String> {
        let login = read_login(cli_dir)?;
        let expired = login.expires_at.is_some_and(|at| at <= now_ms);
        if expired {
            if let Ok(fresh) = self.refresh_at(cli_dir, now_ms) {
                return Ok(fresh);
            }
        }
        Ok(login.token)
    }
}

impl Credentials for CliCredentials {
    fn access_token(&self, cli_dir: &Path) -> Result<String, String> {
        self.token_at(cli_dir, now_ms())
    }

    fn expires_at(&self, cli_dir: &Path) -> Option<i64> {
        read_login(cli_dir).ok().and_then(|l| l.expires_at)
    }

    fn refresh(&self, cli_dir: &Path) -> Result<String, String> {
        self.refresh_at(cli_dir, now_ms())
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    const NOW: i64 = 1_800_000_000_000;

    fn folder_with(json: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(FILE), json).unwrap();
        dir
    }

    /// An endpoint that answers one fixed thing and counts the calls.
    struct Fake {
        answer: Result<(String, Option<String>, i64), String>,
        calls: AtomicU32,
    }

    impl Fake {
        fn granting(access: &str, refresh: Option<&str>, expires_in: i64) -> Self {
            Fake {
                answer: Ok((access.to_owned(), refresh.map(str::to_owned), expires_in)),
                calls: AtomicU32::new(0),
            }
        }

        fn failing(why: &str) -> Self {
            Fake {
                answer: Err(why.to_owned()),
                calls: AtomicU32::new(0),
            }
        }
    }

    impl TokenEndpoint for &'static Fake {
        fn exchange(&self, refresh_token: &str) -> Result<Grant, String> {
            assert_eq!(refresh_token, "r-old");
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.answer
                .clone()
                .map(|(access_token, refresh_token, expires_in)| Grant {
                    access_token,
                    refresh_token,
                    expires_in,
                })
        }
    }

    fn creds(fake: Fake) -> (CliCredentials, &'static Fake) {
        let fake: &'static Fake = Box::leak(Box::new(fake));
        (CliCredentials::with_endpoint(None, Box::new(fake)), fake)
    }

    const LOGIN: &str = r#"{"claudeAiOauth":{"accessToken":"a-old","refreshToken":"r-old","expiresAt":5,"scopes":["user:profile"],"subscriptionType":"max"},"other":{"keep":true}}"#;

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

    #[test]
    fn a_grant_rewrites_three_fields_and_keeps_everything_else_in_order() {
        let dir = folder_with(LOGIN);
        let grant = Grant {
            access_token: "a-new".to_owned(),
            refresh_token: Some("r-new".to_owned()),
            expires_in: 3600,
        };
        store_grant(dir.path(), &grant, "r-old", NOW).unwrap();
        let text = fs::read_to_string(dir.path().join(FILE)).unwrap();
        let expected = format!(
            r#"{{"claudeAiOauth":{{"accessToken":"a-new","refreshToken":"r-new","expiresAt":{},"scopes":["user:profile"],"subscriptionType":"max"}},"other":{{"keep":true}}}}"#,
            NOW + 3_600_000
        );
        assert_eq!(text, expected);
        let left: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(left.len(), 1, "no temp file stays");
    }

    #[test]
    fn a_grant_without_a_new_refresh_token_keeps_the_old_one() {
        let dir = folder_with(LOGIN);
        let grant = Grant {
            access_token: "a-new".to_owned(),
            refresh_token: None,
            expires_in: 60,
        };
        store_grant(dir.path(), &grant, "r-old", NOW).unwrap();
        let login = read_login(dir.path()).unwrap();
        assert_eq!(login.token, "a-new");
        assert_eq!(login.refresh_token.as_deref(), Some("r-old"));
        assert_eq!(login.expires_at, Some(NOW + 60_000));
    }

    #[test]
    fn a_pretty_file_stays_pretty() {
        let dir = folder_with(
            "{\n  \"claudeAiOauth\": {\n    \"accessToken\": \"a-old\",\n    \"refreshToken\": \"r-old\"\n  }\n}\n",
        );
        let grant = Grant {
            access_token: "a-new".to_owned(),
            refresh_token: None,
            expires_in: 1,
        };
        store_grant(dir.path(), &grant, "r-old", NOW).unwrap();
        let text = fs::read_to_string(dir.path().join(FILE)).unwrap();
        assert!(text.contains("\n  \"claudeAiOauth\""));
    }

    #[test]
    fn a_refresh_that_changes_nothing_falls_back_to_the_grant() {
        let dir = folder_with(LOGIN);
        let (creds, fake) = creds(Fake::granting("a-new", Some("r-new"), 3600));
        let token = creds.refresh_at(dir.path(), NOW).unwrap();
        assert_eq!(token, "a-new");
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
        assert_eq!(read_login(dir.path()).unwrap().token, "a-new");
    }

    #[test]
    fn a_failed_grant_is_an_error_without_tokens() {
        let dir = folder_with(LOGIN);
        let (creds, _) = creds(Fake::failing("the token endpoint answered 400"));
        let err = creds.refresh_at(dir.path(), NOW).unwrap_err();
        assert_eq!(err, "the token endpoint answered 400");
        assert!(!err.contains("a-old") && !err.contains("r-old"));
        assert_eq!(read_login(dir.path()).unwrap().token, "a-old");
    }

    #[test]
    fn no_refresh_token_means_no_grant() {
        let dir = folder_with(r#"{"claudeAiOauth":{"accessToken":"a-old"}}"#);
        let (creds, fake) = creds(Fake::granting("a-new", None, 1));
        assert!(creds.refresh_at(dir.path(), NOW).is_err());
        assert_eq!(fake.calls.load(Ordering::SeqCst), 0);
    }

    fn with_expiry(at: i64) -> String {
        LOGIN.replace("\"expiresAt\":5", &format!("\"expiresAt\":{at}"))
    }

    #[test]
    fn an_expired_login_is_renewed_before_use() {
        let dir = folder_with(&with_expiry(NOW - 1));
        let (creds, fake) = creds(Fake::granting("a-new", None, 3600));
        assert_eq!(creds.token_at(dir.path(), NOW).unwrap(), "a-new");
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_login_that_is_still_valid_is_never_renewed_ahead_of_time() {
        for at in [NOW + 1, NOW + 60_000, NOW + 3_600_000] {
            let dir = folder_with(&with_expiry(at));
            let (creds, fake) = creds(Fake::granting("a-new", None, 3600));
            assert_eq!(creds.token_at(dir.path(), NOW).unwrap(), "a-old");
            assert_eq!(fake.calls.load(Ordering::SeqCst), 0);
        }
    }

    #[test]
    fn a_failed_renewal_of_an_expired_login_keeps_the_token_in_the_file() {
        let dir = folder_with(&with_expiry(NOW - 1));
        let (creds, fake) = creds(Fake::failing("down"));
        assert_eq!(creds.token_at(dir.path(), NOW).unwrap(), "a-old");
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_grant_is_not_written_when_the_refresh_token_changed_meanwhile() {
        let dir = folder_with(&LOGIN.replace("r-old", "r-by-cli"));
        let before = fs::read_to_string(dir.path().join(FILE)).unwrap();
        let grant = Grant {
            access_token: "a-new".to_owned(),
            refresh_token: Some("r-new".to_owned()),
            expires_in: 60,
        };
        let err = store_grant(dir.path(), &grant, "r-old", NOW).unwrap_err();
        assert!(!err.contains("r-by-cli") && !err.contains("r-old"));
        assert_eq!(fs::read_to_string(dir.path().join(FILE)).unwrap(), before);
    }
}
