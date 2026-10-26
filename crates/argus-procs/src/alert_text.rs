//! The words of an alert wake: `3 processes need a look: node 4812 (vite) 1.9 GB stale 3h, ...`.

use argus_core::clock::age_text;

use crate::alerts::Due;
use crate::text::{base_name, mem_text};
use crate::types::{Flag, ProcDoc};

/// The tail every alert wake carries; the activity line leaves it off.
pub const ALERT_TAIL: &str = ": Argus > Processes";
const MAX_TEXT: usize = 300;
const MINUTE_MS: i64 = 60_000;

struct Cell {
    key: String,
    mem_mb: i64,
    body: Body,
}

enum Body {
    Group(String),
    Proc {
        doc: Box<ProcDoc>,
        kinds: Vec<String>,
    },
}

fn kind_text(doc: &ProcDoc, kind: Flag) -> String {
    match kind {
        Flag::Stale => format!("stale {}", age_text(doc.idle_min * MINUTE_MS)),
        Flag::Old | Flag::Orphan => {
            format!("{} {}", kind.as_str(), age_text(doc.age_min * MINUTE_MS))
        }
        other => other.as_str().to_string(),
    }
}

fn cells_of(due: &[Due]) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    for item in due {
        match item {
            Due::Group {
                name,
                mem_mb,
                limit,
            } => cells.push(Cell {
                key: format!("group:{name}"),
                mem_mb: *mem_mb,
                body: Body::Group(format!(
                    "{name} group {} over {}",
                    mem_text(*mem_mb),
                    mem_text(crate::num::round_to_i64(*limit))
                )),
            }),
            Due::Proc { doc, kind } => {
                let text = kind_text(doc, *kind);
                match cells.iter_mut().find(|c| c.key == doc.id) {
                    Some(Cell {
                        body: Body::Proc { kinds, .. },
                        ..
                    }) => kinds.push(text),
                    _ => cells.push(Cell {
                        key: doc.id.clone(),
                        mem_mb: doc.mem_mb,
                        body: Body::Proc {
                            doc: doc.clone(),
                            kinds: vec![text],
                        },
                    }),
                }
            }
        }
    }
    cells.sort_by_key(|c| std::cmp::Reverse(c.mem_mb));
    cells
}

fn cell_text(cell: &Cell) -> String {
    match &cell.body {
        Body::Group(text) => text.clone(),
        Body::Proc { doc, kinds } => {
            let hint = doc
                .hint
                .as_ref()
                .map_or(String::new(), |h| format!(" ({h})"));
            let guard = if doc.protected { " (protected)" } else { "" };
            format!(
                "{} {}{hint} {} {}{guard}",
                base_name(&doc.name),
                doc.pid,
                mem_text(doc.mem_mb),
                kinds.join(", ")
            )
        }
    }
}

fn clip(text: &str) -> String {
    text.chars().take(MAX_TEXT).collect()
}

/// The alert text for the due items: the biggest first, cut with `(+N more)` until it fits 300 characters.
#[must_use]
pub fn alert_text(due: &[Due]) -> String {
    let list: Vec<String> = cells_of(due).iter().map(cell_text).collect();
    let head = format!(
        "{} process{} a look: ",
        list.len(),
        if list.len() == 1 { " needs" } else { "es need" }
    );
    for n in (1..=list.len()).rev() {
        let more = if list.len() > n {
            format!(" (+{} more)", list.len() - n)
        } else {
            String::new()
        };
        let text = format!("{head}{}{more}{ALERT_TAIL}", list[..n].join(", "));
        if text.chars().count() <= MAX_TEXT || n == 1 {
            return clip(&text);
        }
    }
    head
}
