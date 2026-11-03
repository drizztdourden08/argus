//! `POST /api/p/:key/procs/:pid/kill {by, tree?, reason?}`: a thread of project `key` ends its
//! own process, and with `tree` the whole tree. The monitor checks the scope rule (every process
//! inside the project's folders, never a Claude session, the dashboard, another project or a
//! system process); this route checks that `by` is a live thread of that board. Answers
//! `{killed, pid, name, exe, tree, error?}`. A shadow server never kills.

use argus_core::{ApiError, ErrorCode};
use argus_procs::{SELF_KILL_KIND, SelfKillRequest};
use argus_store::{ActivityLevel, NewActivity};
use serde_json::Value;

use super::{ENDED, answer, blocking, reason_of, refusal, service};
use crate::router::Reply;
use crate::routes::params;
use crate::{Ctx, RouteOpts, Router};

const BY_MAX: usize = 60;

fn pid_param(ctx: &Ctx) -> Result<u32, ApiError> {
    let pid = ctx.param("pid").trim().parse::<u32>().ok();
    pid.filter(|p| *p > 0)
        .ok_or_else(|| ApiError::bad_request("pid must be a positive integer"))
}

/// The thread `by` of the body, when it is a live thread of the project's board.
fn thread_of(ctx: &Ctx, key: &str, body: &Value) -> Result<String, ApiError> {
    let by: String = body["by"]
        .as_str()
        .unwrap_or("")
        .trim()
        .chars()
        .take(BY_MAX)
        .collect();
    let board = ctx.svc.boards().board(key);
    let thread = board.as_ref().map(|b| &b["threads"][by.as_str()]);
    let live = thread
        .is_some_and(|t| t.is_object() && !ENDED.contains(&t["status"].as_str().unwrap_or("")));
    if !by.is_empty() && live {
        return Ok(by);
    }
    let who = if by.is_empty() {
        "no thread"
    } else {
        by.as_str()
    };
    let text = format!(
        "refused a process kill for project {key} from {who}: not a live thread of its board"
    );
    ctx.svc
        .activity
        .add(NewActivity::new(SELF_KILL_KIND, ActivityLevel::Warn, text));
    Err(ApiError::new(
        ErrorCode::Forbidden,
        format!(
            "only a live thread of project {key} may kill its own processes: send by (the thread id)"
        ),
    ))
}

async fn kill(ctx: Ctx) -> Reply {
    let procs = service(&ctx)?;
    let key = params::key(&ctx)?;
    let pid = pid_param(&ctx)?;
    let body = ctx.json().await?;
    if ctx.svc.shadow {
        return Err(ApiError::new(
            ErrorCode::Forbidden,
            "this Argus runs in shadow mode, which never kills: ask the live dashboard",
        ));
    }
    let thread = thread_of(&ctx, &key, &body)?;
    let request = SelfKillRequest {
        project: key,
        thread,
        pid,
        tree: body["tree"] == true,
        reason: reason_of(&body),
    };
    let done = blocking(move || procs.self_kill(&request)).await?;
    Ok(answer(&done.map_err(|e| refusal(&e))?))
}

/// Registers the route of a thread's own kill.
pub(super) fn register(r: &mut Router, opts: RouteOpts) {
    r.add("POST", "/api/p/:key/procs/:pid/kill", opts, kill);
}
