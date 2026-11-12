//! A sub-agent's own transcript: where it is written (next to the parent session's
//! transcript) and the model its first assistant line shows.

use std::path::Path;

use argus_agents::fs::StdFs;
use argus_agents::hook_logic::{subagent_transcript, transcript_first_prompt, transcript_model};
use serde_json::Value;

use super::ctx::Hook;
use super::input::clip;
use crate::cli::resolve::resolve;

/// `<dir>/<sessionId>/subagents/agent-<agentId>.jsonl` beside the parent transcript, in the
/// form `path.join` gives it; `null` when the payload lacks a piece or an id holds odd
/// characters.
pub fn file_of(hook: &Hook<'_>, agent_id: &str) -> Value {
    let parent = clip(&hook.input["transcript_path"], 1000);
    let sid = clip(&hook.input["session_id"], 120);
    subagent_transcript(Some(&parent), Some(&sid), agent_id).map_or(Value::Null, |file| {
        Value::String(resolve(Path::new(""), &file).to_string_lossy().into_owned())
    })
}

/// The first prompt of the sub-agent transcript at `file`, from its head only.
pub fn prompt_of(file: &Value) -> Option<String> {
    let file = file.as_str().filter(|f| !f.is_empty())?;
    transcript_first_prompt(&StdFs, Path::new(file))
}

/// The model of the first assistant line of the transcript at `file`, from its head only.
pub fn model_of(file: &Value) -> Option<String> {
    let file = file.as_str().filter(|f| !f.is_empty())?;
    transcript_model(&StdFs, Path::new(file))
}
