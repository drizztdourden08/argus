//! Reading server-sent events from a byte stream: blocks end at a blank line, each has an `event:` name
//! and `data:` lines holding one JSON value.

use serde_json::Value;

/// One frame of the stream.
#[derive(Debug, Clone, PartialEq)]
pub struct SseFrame {
    /// The `event:` name (`message` when the block has none).
    pub name: String,
    /// The `data:` lines joined, parsed as JSON.
    pub data: Value,
}

/// Collects chunks and yields the frames they complete.
#[derive(Debug, Default)]
pub struct SseParser {
    buffer: String,
}

fn frame_of(block: &str) -> Option<SseFrame> {
    let mut name = "message";
    let mut data = String::new();
    for line in block.split('\n') {
        if let Some(value) = line.strip_prefix("event: ") {
            name = value.trim();
        } else if let Some(value) = line.strip_prefix("data: ") {
            data.push_str(value);
        }
    }
    if data.is_empty() {
        return None;
    }
    serde_json::from_str(&data).ok().map(|data| SseFrame {
        name: name.to_owned(),
        data,
    })
}

impl SseParser {
    /// A parser with nothing buffered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chunk; returns the frames it completed. A block with no data or with data that is not JSON is skipped.
    pub fn push(&mut self, chunk: &str) -> Vec<SseFrame> {
        self.buffer.push_str(chunk);
        let mut frames = Vec::new();
        while let Some(at) = self.buffer.find("\n\n") {
            let block: String = self.buffer.drain(..at + 2).collect();
            frames.extend(frame_of(&block[..at]));
        }
        frames
    }
}
