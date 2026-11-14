//! The two answers the CLI documents for `--permission-prompt-tool`.

use serde_json::{Value, json};

use super::model::{Approval, Status};

/// The message when nobody answered in time.
pub const TIMED_OUT: &str = "the owner did not answer";

/// The message of a denial that carries none.
pub const DENIED_DEFAULT: &str = "the owner denied it";

/// `{"behavior":"allow","updatedInput":<input>}`.
#[must_use]
pub fn allow(input: &Value) -> Value {
    json!({ "behavior": "allow", "updatedInput": input })
}

/// `{"behavior":"deny","message":"..."}`.
#[must_use]
pub fn deny(message: &str) -> Value {
    json!({ "behavior": "deny", "message": message })
}

/// The answer for a settled approval; a pending one is denied, since nobody allowed it.
#[must_use]
pub fn reply_for(approval: &Approval, input: &Value) -> Value {
    match approval.status {
        Status::Allowed => allow(input),
        Status::Denied => deny(approval.message.as_deref().unwrap_or(DENIED_DEFAULT)),
        Status::Expired | Status::Pending => deny(TIMED_OUT),
    }
}
