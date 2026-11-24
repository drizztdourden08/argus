//! The login of a headless wake: the access token of the Claude account the thread's session belongs to,
//! so the wake runs with the login Argus already manages and not with the CLI's default login, which can lapse.
//! The token never reaches a log, a notice or a message: only fixed reasons leave this module.

use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use argus_accounts::session_map::{account_for_scopes, local_ids, scopes_of};
use argus_accounts::{Account, Credentials};

use super::account_pick::{Use, pick};

/// An access token held for one headless resume. Its `Debug` hides it.
pub struct Login(String);

impl Login {
    /// Holds `token`.
    #[must_use]
    pub fn new(token: String) -> Self {
        Self(token)
    }

    /// The token, for the child's environment only.
    pub(super) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Login {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<hidden>")
    }
}

/// The access token of the login in `cli_dir`: the stored one, or a refreshed one when it expired at or
/// before `now_ms`. The error is a fixed reason, never the text of the failure.
///
/// # Errors
/// When no token could be had.
pub fn token_of(
    creds: &dyn Credentials,
    cli_dir: &Path,
    now_ms: i64,
) -> Result<Login, &'static str> {
    let expired = creds.expires_at(cli_dir).is_some_and(|at| at <= now_ms);
    let token = if expired {
        creds.refresh(cli_dir)
    } else {
        creds.access_token(cli_dir)
    };
    token.map(Login::new).map_err(|_| "token unavailable")
}

fn now_ms() -> i64 {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
}

/// A login with the account it belongs to.
pub struct Signed {
    /// The token.
    pub login: Login,
    /// The key of the account that signed in.
    pub account: String,
    /// Why the owning account was passed over, when it was.
    pub why: Option<String>,
}

/// The login for the desktop session `session` (a hook session id), or why none. The owning account signs
/// in, unless it is at its limit and another account has room.
pub(crate) fn login_of(
    creds: &dyn Credentials,
    accounts: &[Account],
    uses: &[Use],
    app: &Path,
    session: &str,
) -> Result<Signed, &'static str> {
    let wanted = [session.to_owned()];
    let local = local_ids(app, &wanted);
    let scopes = local
        .get(session)
        .map_or_else(Vec::new, |id| scopes_of(app, id));
    let owner = account_for_scopes(accounts, &scopes).ok_or("no matching account")?;
    let chosen = pick(accounts, owner, uses);
    let sign = |account: &Account, why: Option<String>| {
        token_of(creds, Path::new(&account.cli_dir), now_ms()).map(|login| Signed {
            login,
            account: account.key.clone(),
            why,
        })
    };
    match sign(chosen.account, chosen.why.clone()) {
        Err(_) if chosen.why.is_some() => sign(owner, None),
        other => other,
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        expires: Option<i64>,
    }

    impl Credentials for Fake {
        fn access_token(&self, _cli_dir: &Path) -> Result<String, String> {
            Ok("fake-stored".to_owned())
        }
        fn expires_at(&self, _cli_dir: &Path) -> Option<i64> {
            self.expires
        }
        fn refresh(&self, _cli_dir: &Path) -> Result<String, String> {
            Ok("fake-refreshed".to_owned())
        }
    }

    struct Broken;

    impl Credentials for Broken {
        fn access_token(&self, _cli_dir: &Path) -> Result<String, String> {
            Err("secret-in-error".to_owned())
        }
        fn expires_at(&self, _cli_dir: &Path) -> Option<i64> {
            None
        }
        fn refresh(&self, _cli_dir: &Path) -> Result<String, String> {
            Err("secret-in-error".to_owned())
        }
    }

    fn token(creds: &dyn Credentials) -> Result<String, &'static str> {
        token_of(creds, Path::new("dir"), 1000).map(|l| l.expose().to_owned())
    }

    #[test]
    fn a_live_token_is_used_as_stored() {
        assert_eq!(
            token(&Fake {
                expires: Some(1001)
            })
            .ok()
            .as_deref(),
            Some("fake-stored")
        );
        assert_eq!(
            token(&Fake { expires: None }).ok().as_deref(),
            Some("fake-stored")
        );
    }

    #[test]
    fn a_token_that_expires_now_or_earlier_is_refreshed() {
        assert_eq!(
            token(&Fake {
                expires: Some(1000)
            })
            .ok()
            .as_deref(),
            Some("fake-refreshed")
        );
        assert_eq!(
            token(&Fake { expires: Some(5) }).ok().as_deref(),
            Some("fake-refreshed")
        );
    }

    #[test]
    fn a_failure_gives_a_fixed_reason_and_never_its_text() {
        assert_eq!(token(&Broken), Err("token unavailable"));
    }

    #[test]
    fn a_login_hides_its_token_in_debug_output() {
        let login = Login::new("fake-stored".to_owned());
        assert_eq!(format!("{login:?}"), "<hidden>");
    }
}
