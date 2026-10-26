//! `coord portfolio init | add | remove`: the registry of the projects Argus lists.

use std::path::Path;

use argus_board::json::{map_mut, obj};
use argus_board::limits::require_text;
use argus_board::store::board_path;
use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};

use super::with_portfolio;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::resolve::resolve;
use crate::cli::route::Call;
use crate::verbs::remote::support::scope::coord_home;
use crate::verbs::remote::support::text::is_key;
use crate::verbs::remote::support::url::dashboard_url;

/// `coord portfolio init`.
pub(super) fn init(call: &mut Call<'_>) -> Outcome {
    let line = format!(
        "portfolio (Argus) at {} (page: {})",
        board_path(&coord_home(call), PORTFOLIO).display(),
        dashboard_url(call, "/")
    );
    with_portfolio(call, true, |p, scope| {
        p["name"] = json!("Argus");
        scope.say(&[&line]);
        Ok(())
    })
}

fn key_of(call: &Call<'_>, refusal: &str) -> Result<String, CoordError> {
    call.positional
        .first()
        .filter(|k| is_key(k))
        .cloned()
        .ok_or_else(|| CoordError::refuse(refusal))
}

/// `coord portfolio add <key> --name "<Name>" [--root <dir>]`.
pub(super) fn add(call: &mut Call<'_>) -> Outcome {
    let key = key_of(call, "pass the project key (the ai-config key, e.g. rotp).");
    with_portfolio(call, false, |p, scope| {
        let key = key?;
        let name_option = scope.options.json("name").unwrap_or(Value::Null);
        let name = require_text("name", &name_option)?;
        let root = match scope.options.text("root") {
            Some(root) => json!(resolve(scope.cwd, Path::new(root)).to_string_lossy()),
            None => p["projects"][key.as_str()]
                .get("root")
                .cloned()
                .unwrap_or(Value::Null),
        };
        map_mut(p, "projects").insert(
            key.clone(),
            json!({ "key": key, "name": name, "root": root }),
        );
        map_mut(p, "pushedHash").shift_remove(&key);
        scope.say(&[&format!("{key} listed as \"{name}\"")]);
        Ok(())
    })
}

/// `coord portfolio remove <key>`: a retired project drops off the portfolio, its board folder
/// stays on disk.
pub(super) fn remove(call: &mut Call<'_>) -> Outcome {
    let key = key_of(call, "pass the project key to retire (e.g. foundation).");
    with_portfolio(call, false, |p, scope| {
        let key = key?;
        let mut record = match obj(&p["projects"]).get(&key) {
            Some(known) => known.clone(),
            None => json!({ "key": key, "name": key, "root": null }),
        };
        record["retired"] = json!(true);
        map_mut(p, "projects").insert(key.clone(), record);
        map_mut(p, "pushedHash").shift_remove(&key);
        scope.say(&[&format!(
            "{key} retired from the portfolio (its board folder is kept)."
        )]);
        Ok(())
    })
}
