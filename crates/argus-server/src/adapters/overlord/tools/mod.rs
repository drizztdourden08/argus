//! The Overlord's own tools on `POST /mcp/overlord` (I-192): reach a thread, list the Desktop sessions and
//! resume one. The shadow Overlord only reads, so it may list sessions and nothing else.

mod board;
mod resume;

use std::sync::Arc;

use argus_accounts::find_app_data;
use argus_accounts::session_list::sessions;
use argus_overlord::tools::Call;

use crate::Services;
use crate::services::overlord::config::is_shadow;

/// How many sessions the list returns at most.
const SESSION_LIMIT: usize = 200;

const SHADOW_REFUSAL: &str = "the shadow Overlord only reads";

/// What a tool says back, and whether it failed.
pub struct Answer {
    pub text: String,
    pub is_error: bool,
}

impl Answer {
    fn of(result: Result<String, String>) -> Self {
        match result {
            Ok(text) => Self {
                text,
                is_error: false,
            },
            Err(text) => Self {
                text,
                is_error: true,
            },
        }
    }
}

fn list(svc: &Services) -> Result<String, String> {
    let accounts = svc.accounts.as_ref().ok_or("the accounts service is off")?;
    let app = find_app_data(accounts.config().app_data.as_deref())
        .ok_or("the desktop app data folder was not found")?;
    serde_json::to_string(&sessions(&app, SESSION_LIMIT)).map_err(|e| e.to_string())
}

/// Runs a file or board tool off the async threads.
async fn blocking<F>(svc: &Arc<Services>, work: F) -> Result<String, String>
where
    F: FnOnce(&Services) -> Result<String, String> + Send + 'static,
{
    let svc = Arc::clone(svc);
    tokio::task::spawn_blocking(move || work(&svc))
        .await
        .map_err(|_| "the tool stopped before it answered".to_owned())?
}

async fn dispatch(svc: &Arc<Services>, call: Call) -> Result<String, String> {
    match call {
        Call::SessionsList => blocking(svc, list).await,
        Call::ThreadMessage {
            project,
            thread,
            text,
        } => blocking(svc, move |s| board::message(s, &project, &thread, &text)).await,
        Call::ThreadWake {
            project,
            thread,
            reason,
        } => blocking(svc, move |s| board::wake_only(s, &project, &thread, &reason)).await,
        Call::SessionMessage { local_id, text } => resume::run(svc, &local_id, &text).await,
    }
}

/// Runs one tool call. The shadow is refused every tool that writes.
pub async fn run(svc: &Arc<Services>, call: Call) -> Answer {
    if is_shadow(svc) && !matches!(call, Call::SessionsList) {
        return Answer::of(Err(SHADOW_REFUSAL.to_owned()));
    }
    Answer::of(dispatch(svc, call).await)
}
