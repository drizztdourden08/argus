//! Cuts a byte stream read in arbitrary chunks into whole lines.

/// Holds the unfinished tail of the last read until its newline arrives.
#[derive(Debug, Default)]
pub struct LineSplitter {
    tail: String,
}

impl LineSplitter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chunk; returns every line it completed, without the newline.
    pub fn push(&mut self, chunk: &str) -> Vec<String> {
        self.tail.push_str(chunk);
        let mut lines = Vec::new();
        while let Some(at) = self.tail.find('\n') {
            let rest = self.tail.split_off(at + 1);
            let line = std::mem::replace(&mut self.tail, rest);
            lines.push(line.trim_end_matches(['\n', '\r']).to_string());
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn partial_line_waits_for_its_newline() {
        let mut splitter = LineSplitter::new();
        assert_eq!(splitter.push(r#"{"type":"res"#).len(), 0);
        let lines = splitter.push("ult\"}\r\n{\"a\":1}\n{\"b\"");
        assert_eq!(lines, vec![r#"{"type":"result"}"#, r#"{"a":1}"#]);
        assert_eq!(splitter.push(":2}\n"), vec![r#"{"b":2}"#]);
    }
}
