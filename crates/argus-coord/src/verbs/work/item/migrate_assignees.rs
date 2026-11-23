//! `coord item migrate-assignees [--dry-run]`: an open item that a live agent works on (the agent
//! lists it in `items`) gets `agent:A-n` as its assignee, so the board shows the one holder. A dry
//! run does all of it on a copy of the board and writes nothing.

use argus_board::Board;
use argus_board::agent_links::items_of;
use argus_board::agents::is_live;
use argus_board::json::{display, doc, section, truthy};
use serde_json::{Map, Value};

use super::migrate::edit;
use crate::cli::{Options, Outcome};
use crate::verbs::work::report_lines::say_report;
use crate::verbs::work::run::Run;
use crate::who::Caller;

/// The live agent that works on the item: the newest record when several do.
fn holder_of(board: &Board, item: &str) -> Option<String> {
    section(board, "agents")
        .iter()
        .filter(|(_, agent)| is_live(agent) && items_of(agent).iter().any(|i| i == item))
        .map(|(id, _)| id.clone())
        .next_back()
}

fn open(item: &Value) -> bool {
    !truthy(item, "deletedAt")
        && !truthy(item, "archivedAt")
        && !matches!(item["status"].as_str(), Some("done" | "dropped"))
}

/// The items to change: id and the `agent:A-n` they should name.
fn wanted(board: &Board) -> Vec<(String, String)> {
    section(board, "items")
        .iter()
        .filter(|(_, item)| open(item))
        .filter_map(|(id, item)| {
            let agent = format!("agent:{}", holder_of(board, id)?);
            (display(&item["assignee"]) != agent).then(|| (id.clone(), agent))
        })
        .collect()
}

fn migrate_all(board: &mut Board, by: &Caller, options: &Options) -> Vec<String> {
    let mut lines = Vec::new();
    for (id, agent) in wanted(board) {
        let before = board.clone();
        let from = doc(board, "items", &id).map_or_else(String::new, |i| display(&i["assignee"]));
        let set = Map::from_iter([("assignee".to_owned(), Value::from(agent.as_str()))]);
        match edit(board, by, (id.as_str(), set), options) {
            Ok(()) => lines.push(format!("{id}: assignee \"{from}\" -> {agent}")),
            Err(e) => {
                *board = before;
                lines.push(format!("{id}: kept ({e})"));
            }
        }
    }
    lines
}

fn summary(lines: &[String], dry: bool) -> String {
    let moved = lines.iter().filter(|l| !l.contains(": kept (")).count();
    let kept = lines.len() - moved;
    if lines.is_empty() {
        "no item needs an agent assignee.".to_owned()
    } else if dry {
        format!("dry run: {moved} assignee(s) would migrate, {kept} would stay; nothing written.")
    } else {
        format!("migrated {moved} assignee(s), kept {kept}.")
    }
}

fn print(r: &mut Run<'_>, lines: &[String], dry: bool) {
    say_report(r, lines, "[dry run]", dry, summary(lines, dry));
}

/// `coord item migrate-assignees [--dry-run]`.
pub(super) fn migrate(r: &mut Run<'_>) -> Outcome {
    if r.options.get("dry-run").is_some() {
        return r.read(|board, r| {
            let by = r.who(board)?;
            let lines = migrate_all(&mut board.clone(), &by, r.options);
            print(r, &lines, true);
            Ok(())
        });
    }
    r.write(|board, r| {
        let by = r.begin(board)?;
        let lines = migrate_all(board, &by, r.options);
        print(r, &lines, false);
        Ok(())
    })
}
