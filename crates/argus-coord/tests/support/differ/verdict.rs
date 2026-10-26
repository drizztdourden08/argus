//! Judging one step: what each side did (normalised) and whether it is the same. "Rust: not
//! ported (120)" is its own verdict, never a mismatch: exit 120 means the shim would run Node,
//! so there is nothing to compare yet.

use std::fmt::{self, Write};

use super::snapshot::Snapshot;

/// The exit code that says "not served here, run Node" (`argus_coord::NOT_PORTED`).
pub const NOT_PORTED: i32 = 120;

/// What one side did in one step, normalised.
#[derive(Clone, Debug)]
pub struct Seen {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    /// The home after the step.
    pub files: Snapshot,
}

/// The outcome of one step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Same exit code, streams and files.
    Same,
    /// The Argus side answered exit 120; the step ran on Node for both homes (as the shim does).
    NotPorted,
    /// The sides differ; one line per difference.
    Mismatch(Vec<String>),
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Same => f.write_str("same"),
            Verdict::NotPorted => write!(f, "rust: not ported ({NOT_PORTED})"),
            Verdict::Mismatch(lines) => write!(f, "MISMATCH\n{}", lines.join("\n")),
        }
    }
}

/// Compares the Node side with the Argus side.
#[must_use]
pub fn judge(node: &Seen, rust: &Seen) -> Verdict {
    if rust.code == Some(NOT_PORTED) && node.code != Some(NOT_PORTED) {
        return Verdict::NotPorted;
    }
    let mut lines = Vec::new();
    if node.code != rust.code {
        lines.push(format!(
            "  exit code: node {:?}, rust {:?}",
            node.code, rust.code
        ));
    }
    stream_line(&mut lines, "stdout", &node.stdout, &rust.stdout);
    stream_line(&mut lines, "stderr", &node.stderr, &rust.stderr);
    file_lines(&mut lines, &node.files, &rust.files);
    if lines.is_empty() {
        Verdict::Same
    } else {
        Verdict::Mismatch(lines)
    }
}

fn stream_line(lines: &mut Vec<String>, what: &str, node: &str, rust: &str) {
    if node != rust {
        lines.push(format!(
            "  {what} differs ({})",
            first_difference(node, rust)
        ));
    }
}

fn file_lines(lines: &mut Vec<String>, node: &Snapshot, rust: &Snapshot) {
    for (path, text) in node {
        match rust.get(path) {
            None => lines.push(format!("  file {path}: only in node")),
            Some(other) if other != text => {
                lines.push(format!(
                    "  file {path} differs ({})",
                    first_difference(text, other)
                ));
            }
            Some(_) => {}
        }
    }
    for path in rust.keys().filter(|p| !node.contains_key(*p)) {
        lines.push(format!("  file {path}: only in rust"));
    }
}

/// The first line where two texts part, with both lines (clipped).
#[must_use]
pub fn first_difference(node: &str, rust: &str) -> String {
    let (mut node_lines, mut rust_lines) = (node.split('\n'), rust.split('\n'));
    let mut line = 1;
    loop {
        match (node_lines.next(), rust_lines.next()) {
            (Some(left), Some(right)) if left == right => line += 1,
            (None, None) => return "same lines, different ending".to_owned(),
            (left, right) => {
                let mut out = format!("line {line}\n");
                writeln!(out, "      node: {}", clip(left)).ok();
                write!(out, "      rust: {}", clip(right)).ok();
                return out;
            }
        }
    }
}

fn clip(line: Option<&str>) -> String {
    match line {
        None => "(no line)".to_owned(),
        Some(text) if text.chars().count() > 160 => {
            format!("{}...", text.chars().take(160).collect::<String>())
        }
        Some(text) => text.to_owned(),
    }
}
