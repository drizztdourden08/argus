//! Running the rules of a column over the items a write touched.

use serde_json::Value;

use super::{assignee, column::column_of, field, limit, parent};
use crate::Board;
use crate::json::doc;
use crate::kanban_config::{Column, KanbanConfig, Rule};

const WRITE_KINDS: [&str; 4] = [
    "require-parent",
    "require-assignee",
    "limit",
    "require-field",
];

/// What the rules said of a write: the first refusal, and every warning.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Verdict {
    /// The refusal, starting with `rule-refused:`.
    pub refusal: Option<String>,
    /// One line per rule that only warns.
    pub warnings: Vec<String>,
}

fn rules_of<'a>(config: &'a KanbanConfig, column: &'a Column) -> impl Iterator<Item = &'a Rule> {
    column
        .rules
        .iter()
        .filter_map(|id| config.rules.iter().find(|r| &r.id == id))
}

fn acts_on_write(config: &KanbanConfig, column: &Column) -> bool {
    let limited = column.limit.as_ref().is_some_and(|l| l.mode != "alert");
    limited || rules_of(config, column).any(|r| WRITE_KINDS.contains(&r.kind.as_str()))
}

/// Whether the config has any rule that acts when an item is written.
#[must_use]
pub fn has_write_rules(config: &KanbanConfig) -> bool {
    config.columns.iter().any(|c| acts_on_write(config, c))
}

/// A break: the sentence, and whether its mode refuses.
struct Broken {
    refuses: bool,
    sentence: String,
}

/// A break in the mode given; a rule with no mode only warns.
fn broken(mode: Option<&str>, sentence: Option<String>) -> Option<Broken> {
    sentence.map(|sentence| Broken {
        refuses: mode == Some("refuse"),
        sentence,
    })
}

fn of_rule(
    (board, config): (&Board, &KanbanConfig),
    column: &Column,
    rule: &Rule,
    item: &Value,
) -> Option<Broken> {
    let sentence = match rule.kind.as_str() {
        "require-parent" => parent::breach(board, rule, item),
        "require-assignee" => assignee::breach(rule, item),
        "require-field" => field::breach(rule, item),
        "limit" => limit::breach(board, config, column),
        _ => None,
    };
    broken(rule.mode.as_deref(), sentence)
}

/// The column's own limit, unless a `limit` rule of the column already speaks for it.
fn of_column_limit((board, config): (&Board, &KanbanConfig), column: &Column) -> Option<Broken> {
    let mode = column.limit.as_ref().map(|l| l.mode.as_str())?;
    let ruled = rules_of(config, column).any(|r| r.kind == "limit");
    if ruled || mode == "alert" {
        return None;
    }
    broken(Some(mode), limit::breach(board, config, column))
}

fn breaks_of(at: (&Board, &KanbanConfig), column: &Column, item: &Value) -> Vec<Broken> {
    let mut found: Vec<Broken> = rules_of(at.1, column)
        .filter_map(|r| of_rule(at, column, r, item))
        .collect();
    found.extend(of_column_limit(at, column));
    found
}

fn line(id: &str, column: &str, broken: &Broken) -> String {
    if broken.refuses {
        format!(
            "rule-refused: {id} cannot enter {column}: {}",
            broken.sentence
        )
    } else {
        format!("{id} in {column}: {}", broken.sentence)
    }
}

/// The verdict of the rules on the items `ids`, read from the board as the write left it.
#[must_use]
pub fn evaluate(board: &Board, config: &KanbanConfig, ids: &[String]) -> Verdict {
    let mut verdict = Verdict::default();
    for id in ids {
        let Some(item) = doc(board, "items", id) else {
            continue;
        };
        let Some(column) = column_of(config, item) else {
            continue;
        };
        for found in breaks_of((board, config), column, item) {
            let said = line(id, &column.name, &found);
            if !found.refuses {
                verdict.warnings.push(said);
            } else if verdict.refusal.is_none() {
                verdict.refusal = Some(said);
            }
        }
    }
    verdict
}
