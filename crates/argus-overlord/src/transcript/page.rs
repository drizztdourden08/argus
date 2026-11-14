//! Reading a transcript by line number.

/// Lines read from a transcript. `next` is the count of complete lines, the `after` of the
/// next read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub lines: Vec<String>,
    pub next: usize,
}

/// The complete lines of `content` after the first `after`. A last line still being written (no
/// newline yet) is left out.
#[must_use]
pub fn lines_after(content: &str, after: usize) -> Page {
    let complete = content.rfind('\n').map_or("", |i| &content[..=i]);
    let all: Vec<&str> = complete.lines().collect();
    Page {
        lines: all.iter().skip(after).map(|l| (*l).to_owned()).collect(),
        next: all.len(),
    }
}
