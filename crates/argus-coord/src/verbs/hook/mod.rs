//! `coord hook <event>`: how the board reaches a thread with no agent in between (`hook.mjs`).
//!
//! | Event | Claude Code hook | Does |
//! |---|---|---|
//! | `session-start` | `SessionStart` | tells a thread who it is, hands its inbox |
//! | `prompt` | `UserPromptSubmit` | hands new inbox messages over with the prompt, and a digest line of the board notices since the last turn |
//! | `tool` | `PostToolUse` | heartbeat, and new messages mid-turn |
//! | `stop` | Stop | marks the thread idle; a waiting message keeps it going |
//! | `guard` | `PreToolUse` `tabs_close` | refuses closing a tab in a registered thread |
//! | `agent-dispatch` | `PreToolUse` `Agent or Task` | remembers what is being dispatched |
//! | `agent-result` | `PostToolUse` `Agent or Task` | binds the agent id Claude printed |
//! | `subagent-start` | `SubagentStart` | records the sub-agent on the board |
//! | `subagent-stop` | `SubagentStop` | closes it, with the model its transcript shows |
//! | `listen` | async, `asyncRewake` | waits for a wake and rewakes the session with it |
//!
//! Every hook reads one JSON object on stdin, returns at once when its folder does not exist,
//! and never breaks a session: the router turns any error or panic into exit 0 with no output,
//! and a hook writes its output only once everything else succeeded.

mod adopt;
mod agent_dispatch;
mod agent_result;
mod alive;
mod autostart;
mod binding;
mod context;
mod ctx;
mod dispatches;
mod events;
mod guard;
mod input;
mod kind;
mod listen;
mod notices;
mod subagent_start;
mod subagent_stop;
mod thread;
mod transcript;
mod visit;
mod waiting;
mod wake_line;

use std::io::Write;

use crate::cli::{Call, CoordError, Outcome};

use autostart::ensure_dashboard;
use ctx::Hook;

/// What a hook has to say once it is done.
pub enum Said {
    /// Nothing: exit 0, no output.
    Nothing,
    /// Text for stdout, written with no trailing newline: exit 0.
    Out(String),
    /// A heard wake: the line on stderr and exit 2, which rewakes the session.
    Wake(String),
}

fn serve(event: &str, hook: &Hook<'_>) -> Result<Said, CoordError> {
    match event {
        "session-start" | "prompt" | "tool" | "stop" => events::run(hook, event),
        "guard" => guard::run(hook),
        "listen" => listen::run(hook),
        "agent-dispatch" => agent_dispatch::run(hook).map(|()| Said::Nothing),
        "agent-result" => agent_result::run(hook).map(|()| Said::Nothing),
        "subagent-start" => subagent_start::run(hook).map(|()| Said::Nothing),
        "subagent-stop" => subagent_stop::run(hook).map(|()| Said::Nothing),
        _ => Ok(Said::Nothing),
    }
}

/// The `hook` verb group: reads the input, starts the dashboard for a session start or a
/// prompt, serves the event and writes what it said.
///
/// # Errors
/// [`CoordError::Exit`] with 2 for a heard wake; any other error is swallowed by the router.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let event = call.positional.first().cloned().unwrap_or_default();
    let Some(input) = input::read(&mut call.inv.stdin) else {
        return Ok(());
    };
    let Some(cwd) = input::folder(&input, &call.inv.cwd) else {
        return Ok(());
    };
    let hook = Hook::new(&call.inv.env, &call.inv.cwd, cwd, input);
    if matches!(event.as_str(), "session-start" | "prompt") {
        ensure_dashboard(hook.env, &hook.paths);
    }
    match serve(&event, &hook)? {
        Said::Nothing => Ok(()),
        Said::Out(text) => {
            call.inv.stdout.write_all(text.as_bytes())?;
            Ok(())
        }
        Said::Wake(line) => {
            writeln!(call.inv.stderr, "{line}")?;
            Err(CoordError::Exit(2))
        }
    }
}
