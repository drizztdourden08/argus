//! Where the project lives: the folder git and Brock work in.

use std::path::{Path, PathBuf};

use argus_board::store::read_board;
use argus_core::config::peek_config;
use argus_core::ids::PORTFOLIO;
use argus_core::remote::ci_root_of;

use crate::cli::resolve::resolve;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::{paths_of, project_of};
use crate::who::git_root;

fn configured(call: &Call<'_>) -> Option<String> {
    let config = peek_config(&paths_of(call), None)?;
    let root = config["projectRoots"][project_of(call)].as_str()?;
    Some(root.to_owned())
}

fn integration(call: &Call<'_>) -> Option<String> {
    ci_root_of(&paths_of(call).coord_home, project_of(call))
}

fn registered(call: &Call<'_>) -> Option<String> {
    let portfolio = read_board(&paths_of(call).coord_home, PORTFOLIO).ok()??;
    let root = portfolio["projects"][project_of(call)]["root"].as_str()?;
    Some(root.to_owned())
}

/// The root of the project: `--root`, else the integration checkout of `coord remote ci-root`,
/// else the project root of `config.json`, else the one the portfolio lists, else the git root
/// of the folder the call runs in.
pub(super) fn project_root(call: &Call<'_>) -> PathBuf {
    let named = call
        .options
        .text("root")
        .map(str::to_owned)
        .or_else(|| integration(call))
        .or_else(|| configured(call))
        .or_else(|| registered(call));
    match named.filter(|r| !r.is_empty()) {
        Some(root) => resolve(&call.inv.cwd, Path::new(&root)),
        None => git_root(&call.inv.cwd),
    }
}
