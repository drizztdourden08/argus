//! Who is calling. Nothing is carried between two Bash calls, so the thread is derived each
//! time: `--as <name>` (see [`caller`] for who may use it), the thread holding this session id,
//! else the thread registered from this checkout that no other session has claimed.

use std::path::Path;

use argus_board::Board;
use argus_board::ops::Role;
use argus_board::store::touch;
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Value, json};

use super::path::{contains, norm};
use super::session::is_ended;
use crate::cli::Options;
use crate::cli::env::Env;
use crate::cli::exit::CoordError;

/// Who is calling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller {
    /// The thread the verb acts as (the `--as` thread, else the session's own).
    pub thread: String,
    /// The authority: coordinator or thread.
    pub role: Role,
    /// The id recorded as the creator or updater.
    pub actor: String,
    /// The thread this session actually is, `None` for a call with no session.
    pub real: Option<String>,
}

fn id_of(thread: &Value) -> String {
    thread["id"].as_str().unwrap_or_default().to_owned()
}

fn role_of(thread: &Value) -> Role {
    if thread["role"] == "coordinator" {
        Role::Coordinator
    } else {
        Role::Thread
    }
}

fn live_threads(board: &Board) -> Vec<&Value> {
    board["threads"]
        .as_object()
        .map(|m| m.values().filter(|t| !is_ended(&t["status"])).collect())
        .unwrap_or_default()
}

fn named_thread<'a>(board: &'a Board, name: &str) -> Result<&'a Value, CoordError> {
    let named = &board["threads"][name];
    if named.is_null() {
        return Err(CoordError::refuse(format!(
            "no thread named \"{name}\" on this board."
        )));
    }
    Ok(named)
}

/// The threads registered from a folder holding `here`, no other session having claimed them.
fn by_root<'a>(live: &[&'a Value], cwd: &Path, sid: Option<&str>) -> Vec<&'a Value> {
    let here = norm(cwd, cwd);
    live.iter()
        .copied()
        .filter(|t| {
            let root = t["root"].as_str().filter(|r| !r.is_empty());
            let claimed = t["sessionId"].as_str().filter(|s| !s.is_empty());
            root.is_some_and(|r| contains(cwd, r, &here)) && (claimed.is_none() || claimed == sid)
        })
        .collect()
}

fn root_key(cwd: &Path, thread: &Value) -> String {
    norm(cwd, Path::new(thread["root"].as_str().unwrap_or_default()))
}

fn deepest_root(mut found: Vec<&Value>, cwd: &Path) -> Result<String, CoordError> {
    found.sort_by_key(|t| std::cmp::Reverse(js::len(&root_key(cwd, t))));
    let top = found[0];
    let top_root = root_key(cwd, top);
    let ties: Vec<String> = found
        .iter()
        .filter(|t| root_key(cwd, t) == top_root)
        .map(|t| id_of(t))
        .collect();
    if ties.len() > 1 {
        return Err(CoordError::refuse(format!(
            "several threads share this root ({}); pass --as <thread>.",
            ties.join(", ")
        )));
    }
    Ok(id_of(top))
}

/// The thread of this call: the one `--as` names, else the live thread holding the session id,
/// else the live thread registered from a folder holding `cwd` that no other session claimed
/// (the deepest root wins, a tie is refused).
///
/// # Errors
/// Refused for an unknown `--as` name, for no thread of this session, and for a tie.
pub fn resolve_thread(
    board: &Board,
    cwd: &Path,
    options: &Options,
    sid: Option<&str>,
) -> Result<String, CoordError> {
    if let Some(name) = options.text("as") {
        return named_thread(board, name).map(id_of);
    }
    let live = live_threads(board);
    if let Some(mine) = sid.and_then(|s| live.iter().find(|t| t["sessionId"].as_str() == Some(s))) {
        return Ok(id_of(mine));
    }
    let found = by_root(&live, cwd, sid);
    if found.is_empty() {
        return Err(CoordError::refuse(
            "no thread registered for this session. Run `coord thread register <name>` first, or pass --as <thread>.",
        ));
    }
    deepest_root(found, cwd)
}

/// The thread this session already is, ignoring `--as`: by session id, else by checkout.
fn session_thread(board: &Board, cwd: &Path, sid: &str) -> Option<String> {
    resolve_thread(board, cwd, &Options::default(), Some(sid)).ok()
}

/// Who is calling, as a [`Caller`]. `--as X` is refused when this session already is a live
/// thread other than X and not the coordinator: a thread never acts as another thread. The
/// coordinator acting as X keeps coordinator authority and is recorded as the actor. A call with
/// no session id (the owner's own terminal) may act as any thread, with that thread's authority.
///
/// # Errors
/// Refused for an unknown `--as` name, for no thread of this session, for a tie, and for a
/// thread acting as another.
pub fn caller(
    board: &Board,
    cwd: &Path,
    env: &Env,
    options: &Options,
) -> Result<Caller, CoordError> {
    let sid = env.session_id();
    let Some(name) = options.text("as") else {
        let id = resolve_thread(board, cwd, &Options::default(), sid)?;
        let role = role_of(&board["threads"][id.as_str()]);
        return Ok(Caller {
            actor: id.clone(),
            real: Some(id.clone()),
            thread: id,
            role,
        });
    };
    let named = named_thread(board, name)?;
    let named_id = id_of(named);
    let real = sid.and_then(|s| session_thread(board, cwd, s));
    let Some(real_id) = real.clone().filter(|r| *r != named_id) else {
        return Ok(Caller {
            actor: named_id.clone(),
            role: role_of(named),
            thread: named_id,
            real,
        });
    };
    if role_of(&board["threads"][real_id.as_str()]) == Role::Coordinator {
        return Ok(Caller {
            thread: named_id,
            role: Role::Coordinator,
            actor: real_id,
            real,
        });
    }
    Err(CoordError::refuse(format!(
        "--as {named_id} refused: this session is thread {real_id}, and a thread never acts as another thread."
    )))
}

/// The thread a verb acts as, with the real caller's heartbeat refreshed (never the `--as`
/// thread's). Every verb a thread runs counts as a sign of life, which is what the stale-thread
/// report reads. A heartbeat is not a change: it does not bump the doc's revision.
///
/// # Errors
/// As [`caller`].
pub fn me(
    board: &mut Board,
    cwd: &Path,
    env: &Env,
    options: &Options,
) -> Result<String, CoordError> {
    let who = caller(board, cwd, env, options)?;
    if let Some(real) = &who.real {
        board["threads"][real.as_str()]["lastSeen"] = json!(now_iso());
        touch(board, "threads", real);
    }
    Ok(who.thread)
}
