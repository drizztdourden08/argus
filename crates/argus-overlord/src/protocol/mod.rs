//! The stream-json codec: one line Claude Code prints with `--output-format stream-json
//! --verbose` becomes a typed [`Event`], and the user message written to stdin with
//! `--input-format stream-json` is built by [`encode_user_message`].
//!
//! Unknown event types, unknown content blocks and unknown fields never fail a session: they are
//! kept as raw JSON.
//!
//! Unverified shapes: the plan (`i159-hosted-overlord.md`, story 3) lists these from the docs,
//! not from a recorded session, until the story 4 probe replaces the fixtures with real lines:
//! the `system/init` fields (`apiKeySource`, `mcp_servers`, `tools`), the `stream_event` wrapper
//! used when partial messages are on, the `usage` counters of `result`, and a user message whose
//! `content` is a plain string.

mod block;
mod decode;
mod encode;
mod event;
mod lines;

pub use block::Block;
pub use decode::{DecodeError, decode_line};
pub use encode::encode_user_message;
pub use event::{Event, Init, Message, Outcome, Raw, Usage};
pub use lines::LineSplitter;
