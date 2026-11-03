//! `coord build migrate-links [--dry-run]`: every build gets its place in the item model, once
//! per board and idempotent.
//!
//! | Launcher today | Becomes |
//! |---|---|
//! | the board's master build (the launcher named `master`) | flagged `master: true` |
//! | a bundle that exactly one test plan names | `for` that plan |
//! | a bundle whose tests all verify one item | `for` that item |
//! | a live build with no clear match | `relink: true`, to link with `coord launcher link` |
//! | a retired build with no clear match | left as it is |
//!
//! A dry run works on the board it read and writes nothing.

use argus_board::Board;
use argus_board::json::{doc, doc_mut, section, set, truthy, unset};
use argus_board::ops::Role;
use argus_board::registry::registry;
use argus_board::served::{closure, for_of};
use argus_board::store::touch;
use serde_json::Value;

use super::support::{audit, carrying};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::serves::plans_naming;
use crate::verbs::agent::base::{read, write};
use crate::who::Caller;

/// The launcher a board that was never migrated calls its master build.
const LEGACY_MASTER: &str = "master";

/// What the migration does to the launchers, in board order.
#[derive(Default)]
struct Plan {
    /// The launcher that gets the master flag.
    master: Option<String>,
    /// `(launcher, item, why)`: builds that get a `for`.
    links: Vec<(String, String, String)>,
    /// Live builds with no clear item, not yet marked.
    relinks: Vec<String>,
    /// Live builds with no clear item that were marked already.
    waiting: Vec<String>,
    /// Retired builds with no clear item.
    left: Vec<String>,
}

impl Plan {
    fn changes(&self) -> usize {
        usize::from(self.master.is_some()) + self.links.len() + self.relinks.len()
    }
}

fn type_of(item: &Value) -> &str {
    item["type"].as_str().unwrap_or("task")
}

/// Whether a build may serve the item: a type the registry lets a build serve, and an item that is
/// still open when the build is live.
fn servable(board: &Board, id: &str, live: bool) -> bool {
    doc(board, "items", id).is_some_and(|item| {
        registry().can_serve(type_of(item))
            && !truthy(item, "deletedAt")
            && (!live || closure(item).is_none())
    })
}

/// The bundles a launcher goes with: the one it was made for, else those that link it.
fn bundles_of(board: &Board, name: &str, rec: &Value) -> Vec<String> {
    if let Some(tag) = rec["bundle"].as_str().filter(|t| !t.is_empty()) {
        return vec![tag.to_owned()];
    }
    section(board, "builds")
        .iter()
        .filter(|(_, bundle)| bundle["launcher"].as_str() == Some(name))
        .map(|(tag, _)| tag.clone())
        .collect()
}

/// The item the tests of a bundle all verify, `None` when they verify none, several or an item
/// that does not exist.
fn tested_item(board: &Board, tag: &str) -> Option<String> {
    let tests = carrying(board, tag);
    let mut items: Vec<&str> = tests
        .iter()
        .map(|test| test["item"].as_str())
        .collect::<Option<Vec<_>>>()?;
    items.sort_unstable();
    items.dedup();
    match items.as_slice() {
        [one] => Some((*one).to_owned()),
        _ => None,
    }
}

/// The item a bundle points at, and why: the one test plan that names it, else the one item its
/// tests verify.
fn bundle_item(board: &Board, tag: &str) -> Option<(String, String)> {
    let plans = plans_naming(board, tag);
    match plans.as_slice() {
        [plan] => Some((
            plan.clone(),
            format!("the one test plan that names bundle {tag}"),
        )),
        [] => tested_item(board, tag)
            .map(|item| (item, format!("every test of bundle {tag} verifies it"))),
        _ => None,
    }
}

/// The one item all the bundles of a launcher point at.
fn item_of(board: &Board, name: &str, rec: &Value, live: bool) -> Option<(String, String)> {
    let mut found: Vec<(String, String)> = bundles_of(board, name, rec)
        .iter()
        .filter_map(|tag| bundle_item(board, tag))
        .filter(|(item, _)| servable(board, item, live))
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found.dedup_by(|a, b| a.0 == b.0);
    (found.len() == 1).then(|| found.remove(0))
}

/// What the migration does to one launcher that is not the master build.
fn plan_one(board: &Board, plan: &mut Plan, (name, rec): (&str, &Value)) {
    if for_of(rec).is_some() {
        return;
    }
    let live = !truthy(rec, "retiredAt");
    match (item_of(board, name, rec, live), live) {
        (Some((item, why)), _) => plan.links.push((name.to_owned(), item, why)),
        (None, true) if truthy(rec, "relink") => plan.waiting.push(name.to_owned()),
        (None, true) => plan.relinks.push(name.to_owned()),
        (None, false) => plan.left.push(name.to_owned()),
    }
}

/// The launcher that is, or will be, the master build.
fn master_of(board: &Board) -> Option<String> {
    let launchers = section(board, "launchers");
    let flagged = launchers.iter().find(|(_, rec)| truthy(rec, "master"));
    if let Some((name, _)) = flagged {
        return Some(name.clone());
    }
    launchers
        .get(LEGACY_MASTER)
        .map(|_| LEGACY_MASTER.to_owned())
}

fn plan_board(board: &Board) -> Plan {
    let mut plan = Plan::default();
    let master = master_of(board);
    let flagged = master
        .as_ref()
        .and_then(|name| doc(board, "launchers", name))
        .is_some_and(|rec| truthy(rec, "master"));
    if !flagged && master.as_deref().is_some_and(|m| live_launcher(board, m)) {
        plan.master.clone_from(&master);
    }
    for (name, rec) in section(board, "launchers") {
        if master.as_deref() != Some(name.as_str()) {
            plan_one(board, &mut plan, (name, rec));
        }
    }
    plan
}

fn live_launcher(board: &Board, name: &str) -> bool {
    doc(board, "launchers", name).is_some_and(|rec| !truthy(rec, "retiredAt"))
}

/// Writes the plan onto the board.
fn apply(board: &mut Board, who: &Caller, plan: &Plan) {
    if let Some(name) = &plan.master {
        stamp(board, name, |rec| set(rec, "master", true));
    }
    for (name, item, _) in &plan.links {
        stamp(board, name, |rec| {
            set(rec, "for", item.as_str());
            unset(rec, "relink");
        });
    }
    for name in &plan.relinks {
        stamp(board, name, |rec| set(rec, "relink", true));
    }
    if plan.changes() > 0 {
        audit(board, who, &summary(plan, false));
    }
}

/// Changes one launcher record and marks it for the sync. The time it was last set stays.
fn stamp(board: &mut Board, name: &str, change: impl FnOnce(&mut Value)) {
    if let Some(rec) = doc_mut(board, "launchers", name) {
        change(rec);
    }
    touch(board, "launchers", name);
}

fn summary(plan: &Plan, dry: bool) -> String {
    let counts = format!(
        "{} master, {} linked, {} need a link, {} retired left unlinked",
        usize::from(plan.master.is_some()),
        plan.links.len(),
        plan.relinks.len() + plan.waiting.len(),
        plan.left.len()
    );
    if dry {
        format!("dry run: {counts}; nothing written.")
    } else {
        format!("build links migrated: {counts}.")
    }
}

fn lines(plan: &Plan) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(name) = &plan.master {
        lines.push(format!("{name}: the master build (flagged master)."));
    }
    for (name, item, why) in &plan.links {
        lines.push(format!("{name}: serves {item} ({why})."));
    }
    for name in plan.relinks.iter().chain(&plan.waiting) {
        lines.push(format!(
            "{name}: no clear item, needs a link: coord launcher link {name} I-n."
        ));
    }
    for name in &plan.left {
        lines.push(format!("{name}: retired, no clear item, left as it is."));
    }
    lines
}

fn print(say: &mut dyn FnMut(&str), plan: &Plan, dry: bool) {
    if plan.changes() + plan.waiting.len() + plan.left.len() == 0 {
        say("no builds to link: every build is linked.");
        return;
    }
    for line in lines(plan) {
        say(&if dry {
            format!("[dry run] {line}")
        } else {
            line
        });
    }
    say(&summary(plan, dry));
}

/// `coord build migrate-links [--dry-run]`.
///
/// # Errors
/// A thread that does not lead the project.
pub fn run(call: &mut Call<'_>) -> Outcome {
    if call.options.get("dry-run").is_some() {
        return read(call, |board, run| {
            let plan = plan_board(board);
            print(&mut |text: &str| run.say(text), &plan, true);
            Ok(())
        });
    }
    write(call, |board, who, run| {
        if who.role == Role::Thread {
            return Err(CoordError::refuse(
                "coord build migrate-links belongs to the coordinator: try --dry-run to see what it would do.",
            ));
        }
        let plan = plan_board(board);
        apply(board, who, &plan);
        print(&mut |text: &str| run.say(text), &plan, false);
        Ok(())
    })
}
