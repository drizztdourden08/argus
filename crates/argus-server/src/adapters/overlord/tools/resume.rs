//! `session_message`: a headless resume of a desktop session with the Overlord's text, through the same pieces
//! as the wake relay: the account login of the session, the archived skip, the CLI in the session's folder and
//! one run at a time per session. The tool waits for the outcome and says it in a few words.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use argus_accounts::session_map::{is_archived, seat_of};
use argus_accounts::{Account, CliCredentials, find_app_data, find_cli_exe};
use argus_core::clock::now_ms;
use argus_store::{ActivityLevel, NewActivity};
use tokio::sync::oneshot;

use crate::Services;
use crate::adapters::wake::account_pick::{Use, uses_now};
use crate::adapters::wake::backoff::{BASE_MS, clock_text};
use crate::adapters::wake::headless::{resume, session_running};
use crate::adapters::wake::login::login_of;
use crate::adapters::wake::outcome::Outcome;

/// The tool waits this long for the outcome; the resume goes on in the background after it.
const WAIT: Duration = Duration::from_secs(10 * 60);

const SIGN_IN: &str = "sign-in: the CLI login expired, the owner must run claude auth login";
const KILLED: &str = "timed out: the resume was killed after 20 minutes";
const STILL_RUNNING: &str =
    "timed out: still running after 10 minutes, it goes on in the background";

/// The answer for an account at its limit: the relay tries the lead again after its first backoff step.
fn limited(now: i64) -> String {
    format!(
        "rate limited, the relay will retry at {}",
        clock_text(now + BASE_MS)
    )
}

/// Where a session runs, checked.
#[derive(Debug, PartialEq, Eq)]
struct Plan {
    cli_id: String,
    cwd: String,
}

/// The CLI session id and folder of the desktop session `local_id`, or why it cannot be resumed.
fn plan(app: &Path, local_id: &str) -> Result<Plan, String> {
    let Some(seat) = seat_of(app, local_id) else {
        return Err(format!("failed: no desktop session \"{local_id}\""));
    };
    if is_archived(app, local_id) {
        return Err("skipped: the session is archived".to_owned());
    }
    let cli_id = seat
        .cli_session_id
        .filter(|id| !id.is_empty())
        .ok_or("failed: the session has no CLI id")?;
    let cwd = seat
        .cwd
        .filter(|cwd| !cwd.is_empty())
        .ok_or("failed: the session has no folder")?;
    Ok(Plan { cli_id, cwd })
}

/// What the headless run needs from the machine.
struct Desktop {
    app: PathBuf,
    exe: PathBuf,
    accounts: Vec<Account>,
    uses: Vec<Use>,
    creds: CliCredentials,
}

fn desktop(svc: &Services) -> Result<Desktop, String> {
    let accounts = svc
        .accounts
        .as_ref()
        .ok_or("failed: the accounts service is off")?;
    let config = accounts.config();
    let found = find_app_data(config.app_data.as_deref());
    let app = found
        .clone()
        .ok_or("failed: the desktop app data folder was not found")?;
    let exe = find_cli_exe(&app).ok_or("failed: the Claude CLI was not found")?;
    Ok(Desktop {
        app,
        exe,
        accounts: config.accounts,
        uses: uses_now(&svc.db),
        creds: CliCredentials::new(found),
    })
}

/// Starts the resume; `done` hears the outcome. Blocks for a moment: it samples the processes.
fn start(
    desk: &Desktop,
    local_id: &str,
    text: &str,
    done: oneshot::Sender<Outcome>,
) -> Result<(), String> {
    let plan = plan(&desk.app, local_id)?;
    if session_running(&plan.cli_id) || session_running(local_id) {
        return Err("skipped: the session is running now".to_owned());
    }
    let signed = login_of(
        &desk.creds,
        &desk.accounts,
        &desk.uses,
        &desk.app,
        &plan.cli_id,
    );
    let login = signed.ok().map(|s| s.login);
    let on_done = move |out| drop(done.send(out));
    let (exe, cwd) = (&desk.exe, Path::new(&plan.cwd));
    resume(exe, cwd, &plan.cli_id, text, login.as_ref(), on_done)
        .map_err(|e| format!("failed: not started ({})", e.kind()))
}

/// The tool's answer for an outcome: the text, and whether the message was delivered.
fn describe(outcome: &Outcome, now: i64) -> Result<String, String> {
    match outcome {
        Outcome::Ok => Ok("ok".to_owned()),
        Outcome::SignIn => Err(SIGN_IN.to_owned()),
        Outcome::RateLimited => Err(limited(now)),
        Outcome::Failed(reason) => Err(format!("failed: {reason}")),
        Outcome::TimedOut => Err(KILLED.to_owned()),
    }
}

fn log(svc: &Services, local_id: &str, answer: &Result<String, String>) {
    let (level, what) = match answer {
        Ok(text) => (ActivityLevel::Info, text),
        Err(text) => (ActivityLevel::Warn, text),
    };
    let text = format!("Overlord resumed session {local_id}: {what}");
    svc.activity
        .add(NewActivity::new("overlord-tool", level, text));
}

async fn resumed(svc: &Arc<Services>, local_id: &str, text: &str) -> Result<String, String> {
    let desk = desktop(svc)?;
    let (done, outcome) = oneshot::channel();
    let (id, line) = (local_id.to_owned(), text.to_owned());
    let started = tokio::task::spawn_blocking(move || start(&desk, &id, &line, done))
        .await
        .map_err(|_| "failed: the resume could not be started".to_owned())?;
    started?;
    match tokio::time::timeout(WAIT, outcome).await {
        Ok(Ok(outcome)) => describe(&outcome, now_ms()),
        Ok(Err(_)) => Err("failed: the resume ended without an outcome".to_owned()),
        Err(_) => Err(STILL_RUNNING.to_owned()),
    }
}

/// Resumes `local_id` with `text` and waits for the outcome.
pub(super) async fn run(svc: &Arc<Services>, local_id: &str, text: &str) -> Result<String, String> {
    let answer = resumed(svc, local_id, text).await;
    log(svc, local_id, &answer);
    answer
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn write(dir: &Path, name: &str, text: &str) {
        let org = dir.join("claude-code-sessions/a/o");
        fs::create_dir_all(&org).expect("org");
        fs::write(org.join(name), text).expect("record");
    }

    #[test]
    fn a_session_with_an_id_and_a_folder_is_planned() {
        let dir = tempfile::tempdir().expect("dir");
        let record = r#"{"cliSessionId":"c1","cwd":"X:/w"}"#;
        write(dir.path(), "local_a.json", record);
        let planned = plan(dir.path(), "local_a").expect("plan");
        let expected = Plan {
            cli_id: "c1".to_owned(),
            cwd: "X:/w".to_owned(),
        };
        assert_eq!(planned, expected);
    }

    #[test]
    fn an_archived_or_unknown_or_incomplete_session_says_why() {
        let dir = tempfile::tempdir().expect("dir");
        let archived = r#"{"isArchived":true,"cliSessionId":"c","cwd":"w"}"#;
        write(dir.path(), "local_arch.json", archived);
        write(dir.path(), "local_nocwd.json", r#"{"cliSessionId":"c"}"#);
        write(dir.path(), "local_noid.json", r#"{"cwd":"w"}"#);
        let why = |id: &str| plan(dir.path(), id).unwrap_err();
        assert_eq!(why("local_arch"), "skipped: the session is archived");
        assert_eq!(why("local_nocwd"), "failed: the session has no folder");
        assert_eq!(why("local_noid"), "failed: the session has no CLI id");
        assert_eq!(
            why("local_none"),
            "failed: no desktop session \"local_none\""
        );
    }

    #[test]
    fn only_a_clean_finish_is_delivered() {
        assert_eq!(describe(&Outcome::Ok, 0), Ok("ok".to_owned()));
        assert_eq!(describe(&Outcome::SignIn, 0), Err(SIGN_IN.to_owned()));
        let failed = describe(&Outcome::Failed("status 529".to_owned()), 0);
        assert_eq!(failed, Err("failed: status 529".to_owned()));
        assert_eq!(describe(&Outcome::TimedOut, 0), Err(KILLED.to_owned()));
        let limit = describe(&Outcome::RateLimited, 0);
        assert_eq!(
            limit,
            Err("rate limited, the relay will retry at 00:02 UTC".to_owned())
        );
    }
}
