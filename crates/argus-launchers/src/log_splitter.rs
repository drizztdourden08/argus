//! From decoded output chunks to lines: `\n` and `\r\n` end a line, a carriage return keeps
//! the last non-empty segment (terminal semantics), a line longer than [`MAX_LINE`]
//! characters is cut into pieces, and a partial line (a prompt without a newline) is
//! released after a pause.

use crate::record::Stream;

/// The longest stored line, in characters.
pub const MAX_LINE: usize = 16 * 1024;
const PARTIAL_MS: i64 = 1000;

#[derive(Debug, Default)]
struct Partial {
    text: String,
    at_ms: i64,
}

/// Terminal semantics for a carriage return: the last non-empty segment stays visible.
fn visible(line: &str) -> String {
    if !line.contains('\r') {
        return line.to_owned();
    }
    line.split('\r')
        .rfind(|p| !p.is_empty())
        .unwrap_or_default()
        .to_owned()
}

/// A line as pieces of at most [`MAX_LINE`] characters; an empty line is one empty piece.
fn pieces(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return vec![String::new()];
    }
    chars.chunks(MAX_LINE).map(|c| c.iter().collect()).collect()
}

#[derive(Debug, Default)]
pub struct LineSplitter {
    out: Partial,
    err: Partial,
}

impl LineSplitter {
    pub fn new() -> Self {
        Self::default()
    }

    fn partial(&mut self, stream: Stream) -> &mut Partial {
        match stream {
            Stream::Err => &mut self.err,
            Stream::Out | Stream::Sys => &mut self.out,
        }
    }

    /// Adds a chunk and returns the lines it completed. A partial line that reaches
    /// [`MAX_LINE`] characters is released in full pieces.
    pub fn push(&mut self, stream: Stream, text: &str, now_ms: i64) -> Vec<String> {
        let partial = self.partial(stream);
        partial.text.push_str(text);
        partial.at_ms = now_ms;
        let mut parts: Vec<String> = partial.text.split('\n').map(str::to_owned).collect();
        partial.text = parts.pop().unwrap_or_default();
        let mut lines: Vec<String> = parts
            .iter()
            .flat_map(|line| pieces(&visible(line.strip_suffix('\r').unwrap_or(line))))
            .collect();
        lines.extend(drain_full_pieces(&mut partial.text));
        lines
    }

    /// The partial lines that have been idle for a second (all of them when `force`).
    pub fn release(&mut self, now_ms: i64, force: bool) -> Vec<(Stream, Vec<String>)> {
        let mut released = Vec::new();
        for stream in [Stream::Out, Stream::Err] {
            let partial = self.partial(stream);
            if partial.text.is_empty() || !(force || now_ms - partial.at_ms >= PARTIAL_MS) {
                continue;
            }
            let text = std::mem::take(&mut partial.text);
            released.push((stream, pieces(&visible(&text))));
        }
        released
    }

    /// True while a partial line waits for more output or its pause.
    pub fn has_partial(&self) -> bool {
        !self.out.text.is_empty() || !self.err.text.is_empty()
    }
}

fn drain_full_pieces(text: &mut String) -> Vec<String> {
    let mut lines = Vec::new();
    while text.chars().count() >= MAX_LINE {
        let cut = text
            .char_indices()
            .nth(MAX_LINE)
            .map_or(text.len(), |(index, _)| index);
        let rest = text.split_off(cut);
        lines.push(visible(text));
        *text = rest;
    }
    lines
}
