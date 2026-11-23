//! `coord item migrate-blockers [--dry-run]`: each free-text `blocker` becomes a link or a to-do
//! and the text is cleared, by the table of the design (4.4):
//!
//! | Text | Becomes |
//! |---|---|
//! | an item, test, to-do or question of this board | `blockedBy` holds its id |
//! | a decision `D-n` of this board | `blockedBy` holds its id |
//! | a `D-n` that is no decision here (a ledger dependency) | dropped, `waitsOn` already blocks |
//! | `external: owner...` or `owner...` | a to-do for the owner, linked |
//! | `external: <anything else>` or any other text | a to-do for the item's thread, linked |
//! | any text on an item that is done or dropped | cleared |
//!
//! A dry run does all of it on a copy of the board and writes nothing.

use argus_board::Board;
use argus_board::json::{display, doc, section, truthy};
use serde_json::{Map, Value};

use super::block::{is_id, linked, make_todo};
use crate::cli::{Options, Outcome};
use crate::verbs::work::crud::{apply, op};
use crate::verbs::work::report_lines::say_report;
use crate::verbs::work::run::{Res, Run};
use crate::who::Caller;

/// The longest to-do text; the text of a blocker is cut to it.
const TODO_TEXT: usize = 190;

/// What becomes of one blocker text.
enum Fate {
    Clear,
    Link(String),
    Drop,
    Todo { who: String, what: String },
}

/// What a run did, line by line.
struct Report {
    lines: Vec<String>,
    moved: usize,
    kept: usize,
}

fn blocker_of(item: &Value) -> Option<String> {
    item["blocker"]
        .as_str()
        .map(|t| argus_core::js::trim(t).to_owned())
        .filter(|t| !t.is_empty())
}

fn finished(item: &Value) -> bool {
    matches!(item["status"].as_str(), Some("done" | "dropped"))
}

/// Whether the id is a live record of this board of the kind its letter says.
fn exists(board: &Board, id: &str) -> bool {
    let (coll, kind) = match id.split_once('-').map(|(p, _)| p) {
        Some("I") => ("items", None),
        Some("T") => ("tests", None),
        Some("H") => ("actions", None),
        Some("Q") => ("entries", Some("question")),
        Some("D") => ("entries", Some("decision")),
        _ => return false,
    };
    doc(board, coll, id).is_some_and(|d| {
        !d.is_null() && !truthy(d, "deletedAt") && kind.is_none_or(|k| d["kind"] == k)
    })
}

/// The text after a leading `external:`, or all of it.
fn without_external(text: &str) -> &str {
    let head = text.get(..9).unwrap_or_default();
    if !head.eq_ignore_ascii_case("external:") {
        return text;
    }
    let rest = argus_core::js::trim(&text[9..]);
    if rest.is_empty() { text } else { rest }
}

/// Whether the text begins with the word `owner`.
fn for_owner(text: &str) -> bool {
    let head = text.get(..5).unwrap_or_default();
    head.eq_ignore_ascii_case("owner")
        && !text[5..].chars().next().is_some_and(char::is_alphanumeric)
}

/// A line of text that fits a to-do.
fn tidy(text: &str) -> String {
    let one = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > TODO_TEXT {
        let cut: String = one.chars().take(TODO_TEXT).collect();
        format!("{cut}...")
    } else {
        one
    }
}

/// Who the to-do is for and what it says: the owner when the text starts with `owner`, else the
/// thread of the item.
fn todo_parts(item: &Value, text: &str) -> (String, String) {
    let body = without_external(text);
    let who = if for_owner(body) {
        "owner".to_owned()
    } else {
        display(&item["thread"])
    };
    (who, tidy(body))
}

fn fate(board: &Board, item: &Value, text: &str) -> Fate {
    if finished(item) {
        return Fate::Clear;
    }
    if is_id(text) && text != display(&item["id"]) {
        if exists(board, text) {
            return Fate::Link(text.to_owned());
        }
        if text.starts_with("D-") {
            return Fate::Drop;
        }
    }
    let (who, what) = todo_parts(item, text);
    Fate::Todo { who, what }
}

fn cleared() -> Map<String, Value> {
    Map::from_iter([("blocker".to_owned(), Value::Null)])
}

pub(super) fn edit(
    board: &mut Board,
    by: &Caller,
    (id, set): (&str, Map<String, Value>),
    options: &Options,
) -> Res<()> {
    let rest = Map::from_iter([("set".to_owned(), Value::Object(set))]);
    apply(board, by, &op(("edit", "items", id), rest, options)?)
}

fn link_and_clear(item: &Value, link: &str) -> Map<String, Value> {
    let mut set = cleared();
    set.insert("blockedBy".to_owned(), linked(item, link));
    set
}

struct Job<'a> {
    by: &'a Caller,
    options: &'a Options,
    item: &'a Value,
    id: &'a str,
}

fn make_link(board: &mut Board, job: &Job<'_>, link: &str) -> Res<String> {
    edit(
        board,
        job.by,
        (job.id, link_and_clear(job.item, link)),
        job.options,
    )?;
    Ok(format!("blockedBy {link}"))
}

fn make_todo_link(board: &mut Board, job: &Job<'_>, (who, what): (&str, &str)) -> Res<String> {
    let todo = make_todo(board, job.by, job.item, (what, who))?;
    edit(
        board,
        job.by,
        (job.id, link_and_clear(job.item, &todo)),
        job.options,
    )?;
    Ok(format!("to-do {todo} for {who}: {what}"))
}

fn clear_only(board: &mut Board, job: &Job<'_>, said: &str) -> Res<String> {
    edit(board, job.by, (job.id, cleared()), job.options)?;
    Ok(said.to_owned())
}

/// A link that the board refuses (a cycle) becomes a to-do for the item's thread.
fn try_link(board: &mut Board, job: &Job<'_>, link: &str, text: &str) -> Res<String> {
    let before = board.clone();
    make_link(board, job, link).or_else(|_| {
        *board = before;
        let (who, what) = todo_parts(job.item, text);
        make_todo_link(board, job, (who.as_str(), what.as_str()))
    })
}

fn carry_out(board: &mut Board, job: &Job<'_>, text: &str) -> Res<String> {
    match fate(board, job.item, text) {
        Fate::Clear => clear_only(board, job, "cleared (the item is done or dropped)"),
        Fate::Drop => clear_only(
            board,
            job,
            "dropped (a dependency: the waitsOn mirror already blocks)",
        ),
        Fate::Link(link) => try_link(board, job, &link, text),
        Fate::Todo { who, what } => make_todo_link(board, job, (who.as_str(), what.as_str())),
    }
}

fn shown(text: &str) -> String {
    let one = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = one.chars().take(60).collect();
    if one.chars().count() > 60 {
        format!("{cut}...")
    } else {
        cut
    }
}

/// Migrates one item; an item that cannot be migrated keeps its text and says why.
fn migrate_item(board: &mut Board, by: &Caller, options: &Options, id: &str, report: &mut Report) {
    let item = doc(board, "items", id).cloned().unwrap_or(Value::Null);
    let Some(text) = blocker_of(&item) else {
        return;
    };
    let before = board.clone();
    let job = Job {
        by,
        options,
        item: &item,
        id,
    };
    match carry_out(board, &job, &text) {
        Ok(result) => {
            report.moved += 1;
            report
                .lines
                .push(format!("{id}: \"{}\" -> {result}", shown(&text)));
        }
        Err(e) => {
            *board = before;
            report.kept += 1;
            report
                .lines
                .push(format!("{id}: \"{}\" kept ({e})", shown(&text)));
        }
    }
}

fn migrate_all(board: &mut Board, by: &Caller, options: &Options) -> Report {
    let ids: Vec<String> = section(board, "items")
        .iter()
        .filter(|(_, item)| !truthy(item, "deletedAt") && blocker_of(item).is_some())
        .map(|(id, _)| id.clone())
        .collect();
    let mut report = Report {
        lines: Vec::new(),
        moved: 0,
        kept: 0,
    };
    for id in ids {
        migrate_item(board, by, options, &id, &mut report);
    }
    report
}

fn summary(report: &Report, dry: bool) -> String {
    if report.moved + report.kept == 0 {
        return "no free-text blockers.".to_owned();
    }
    let (moved, kept) = (report.moved, report.kept);
    if dry {
        format!("dry run: {moved} blocker(s) would migrate, {kept} would stay; nothing written.")
    } else {
        format!("migrated {moved} blocker(s), kept {kept}.")
    }
}

fn print(r: &mut Run<'_>, report: &Report, dry: bool) {
    say_report(r, &report.lines, "[dry run]", dry, summary(report, dry));
}

/// `coord item migrate-blockers [--dry-run]`.
pub(super) fn migrate(r: &mut Run<'_>) -> Outcome {
    if r.options.get("dry-run").is_some() {
        return r.read(|board, r| {
            let by = r.who(board)?;
            let report = migrate_all(&mut board.clone(), &by, r.options);
            print(r, &report, true);
            Ok(())
        });
    }
    r.write(|board, r| {
        let by = r.begin(board)?;
        let report = migrate_all(board, &by, r.options);
        print(r, &report, false);
        Ok(())
    })
}
