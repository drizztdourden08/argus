//! A reader of server-sent events, as the Node listener reads them: a block ends at a blank
//! line (`\n\n`), `event: ` sets the name (default `message`) and `data: ` lines are joined
//! with nothing between them; every other line is ignored.

use std::io::{self, Read};

/// One event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// The event name, `message` when the block names none.
    pub name: String,
    /// The data lines, joined.
    pub data: String,
}

/// Reads events from a stream.
pub struct SseReader<R: Read> {
    source: R,
    buf: Vec<u8>,
}

fn parse_block(block: &str) -> Event {
    let mut event = Event {
        name: "message".to_owned(),
        data: String::new(),
    };
    for line in block.split('\n') {
        if let Some(name) = line.strip_prefix("event: ") {
            name.clone_into(&mut event.name);
        } else if let Some(data) = line.strip_prefix("data: ") {
            event.data.push_str(data);
        }
    }
    event
}

impl<R: Read> SseReader<R> {
    /// A reader of this stream.
    pub fn new(source: R) -> Self {
        SseReader {
            source,
            buf: Vec::new(),
        }
    }

    fn take_block(&mut self) -> Option<String> {
        let at = memchr::memmem::find(&self.buf, b"\n\n")?;
        let block = String::from_utf8_lossy(&self.buf[..at]).into_owned();
        self.buf.drain(..at + 2);
        Some(block)
    }

    /// The next event; `None` when the stream ended.
    ///
    /// # Errors
    /// The stream fails or times out.
    pub fn next_event(&mut self) -> io::Result<Option<Event>> {
        loop {
            if let Some(block) = self.take_block() {
                return Ok(Some(parse_block(&block)));
            }
            let mut chunk = [0_u8; 4096];
            let n = self.source.read(&mut chunk)?;
            if n == 0 {
                return Ok(None);
            }
            self.buf.extend_from_slice(&chunk[..n]);
        }
    }
}
