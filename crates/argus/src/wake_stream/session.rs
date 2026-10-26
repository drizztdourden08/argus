//! One connection's life: bytes in, wake frames out, until the server ends the stream or the
//! command has had enough.

use std::io;
use std::time::Duration;

use argus_wake::sse::SseParser;
use argus_wake::wake_stream::{SILENT_MS, StreamAction, action_of};
use tokio::time::timeout;

use super::connection::Connection;

/// What the caller says after an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// Keep reading.
    Go,
    /// Stop reading; the command is done with this stream.
    Stop,
}

/// How the stream ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    /// The server closed it.
    ByServer,
    /// The caller stopped it.
    ByCaller,
}

/// Decodes UTF-8 that arrives cut anywhere, keeping an unfinished character for the next piece.
#[derive(Default)]
struct Decoder {
    pending: Vec<u8>,
}

impl Decoder {
    fn push(&mut self, bytes: &[u8]) -> String {
        self.pending.extend_from_slice(bytes);
        let valid = match std::str::from_utf8(&self.pending) {
            Ok(_) => self.pending.len(),
            Err(e) => e.valid_up_to(),
        };
        let text = String::from_utf8_lossy(&self.pending[..valid]).into_owned();
        self.pending.drain(..valid);
        text
    }
}

/// Reads the connection and hands each action of the stream to `on_action`.
///
/// # Errors
/// The connection fails, or stays silent for longer than the server's heartbeat allows.
pub async fn pump(
    conn: &mut Connection,
    mut on_action: impl FnMut(StreamAction) -> Flow,
) -> io::Result<Ended> {
    let mut parser = SseParser::new();
    let mut decoder = Decoder::default();
    let silence = Duration::from_millis(SILENT_MS);
    loop {
        let piece = timeout(silence, conn.next())
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "silent for too long"))??;
        let Some(bytes) = piece else {
            return Ok(Ended::ByServer);
        };
        for frame in parser.push(&decoder.push(&bytes)) {
            let Some(action) = action_of(&frame.name, &frame.data) else {
                continue;
            };
            if on_action(action) == Flow::Stop {
                return Ok(Ended::ByCaller);
            }
        }
    }
}
