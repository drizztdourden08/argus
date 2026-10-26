//! `coord portfolio show` and the retired verbs `page`, `url` and `pushed`.

use argus_board::portfolio::{project_keys, summarize};
use argus_board::store::read_board;
use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::scope::coord_home;
use crate::verbs::remote::support::text::cell;
use crate::verbs::remote::support::url::dashboard_url;

fn row(card: &Value, now: i64) -> Vec<String> {
    let counts = card.get("counts");
    let count = |field: &str| cell(counts.and_then(|c| c.get(field)));
    let tests = counts.map_or_else(String::new, |c| {
        format!("{}/{}", cell(c.get("testsPass")), cell(c.get("tests")))
    });
    vec![
        cell(card.get("name")),
        cell(card.get("state")),
        cell(card["coordinator"].get("session")),
        count("threads"),
        count("agents"),
        count("needsYou"),
        count("itemsOpen"),
        tests,
        out::ago(card["coordinator"]["lastSeen"].as_str(), now),
    ]
}

/// `coord portfolio show [--json]`.
pub(super) fn show(call: &mut Call<'_>) -> Outcome {
    let home = coord_home(call);
    let Some(board) = read_board(&home, PORTFOLIO)? else {
        return Err(CoordError::refuse(
            "no portfolio yet: coord portfolio init.",
        ));
    };
    let registry = json!({ "projects": board["projects"] });
    let cards: Vec<Value> = project_keys(&home, &registry)
        .iter()
        .map(|key| summarize(&home, key, &registry, None))
        .collect();
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &json!(cards));
        return Ok(());
    }
    let now = call.inv.clock.now_ms();
    let head = format!("# Argus ({})", dashboard_url(call, "/"));
    call.inv.say(&[&head, ""]);
    let mut rows = vec![
        [
            "project",
            "state",
            "coordinator",
            "threads",
            "agents",
            "needs you",
            "open items",
            "tests",
            "seen",
        ]
        .map(str::to_owned)
        .to_vec(),
    ];
    rows.extend(cards.iter().map(|c| row(c, now)));
    call.inv.say(&[&out::table(&rows)]);
    Ok(())
}

/// `page`, `url` and `pushed` belonged to the hand-published page; the dashboard serves it now.
pub(super) fn retired(call: &mut Call<'_>) {
    let line = format!(
        "retired: the dashboard serves this page at {}",
        dashboard_url(call, "/")
    );
    call.inv.say(&[&line]);
}
