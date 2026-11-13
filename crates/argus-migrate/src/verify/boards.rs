//! The board side of the verification: the table, the folders, the boards and the portfolio.

use std::fs;
use std::path::Path;

use argus_core::boards;
use argus_core::ids::PORTFOLIO;
use serde_json::Value;

use super::{Check, proofs};
use crate::error::Result;
use crate::jsonio;
use crate::plan::board_file;
use crate::rewrite::{board, portfolio};

fn table(check: &Check, problems: &mut Vec<String>) -> Result<()> {
    let table = boards::load(check.home)?;
    for e in check.entries {
        let aliased = table
            .boards
            .get(&e.id)
            .is_some_and(|b| b.aliases.contains(&e.key));
        if !aliased {
            problems.push(format!("{} is not an alias of {}", e.key, e.id));
        }
    }
    Ok(())
}

fn same_as_rewritten(file: &Path, backup: &Path, f: impl FnOnce(&mut Value)) -> Result<bool> {
    let mut expected = jsonio::read(backup)?;
    f(&mut expected);
    Ok(expected == jsonio::read(file)?)
}

fn one_board(check: &Check, e: &crate::journal::Entry, problems: &mut Vec<String>) -> Result<()> {
    if check.home.join(&e.key).exists() {
        problems.push(format!("the old folder {} still exists", e.key));
    }
    let file = board_file(check.home, &e.id);
    if !file.is_file() {
        problems.push(format!("{}/board.json is missing", e.id));
        return Ok(());
    }
    if jsonio::read(&file)?["project"].as_str() == Some(e.key.as_str()) {
        problems.push(format!("{}/board.json still names the key as its project", e.id));
    }
    let Some(backup) = check.backup else {
        return Ok(());
    };
    let before = board_file(backup, &e.key);
    let equal = same_as_rewritten(&file, &before, |v| {
        board::rewrite(v, check.map);
    })?;
    if !equal {
        problems.push(format!("{} differs from its backup outside the whitelist", e.id));
    }
    proofs::check(check, e, &before, problems)
}

fn portfolio_file(check: &Check, problems: &mut Vec<String>) -> Result<()> {
    let file = board_file(check.home, PORTFOLIO);
    let Some(backup) = check.backup.filter(|_| file.is_file()) else {
        return Ok(());
    };
    let equal = same_as_rewritten(&file, &board_file(backup, PORTFOLIO), |v| {
        portfolio::rewrite(v, check.map);
    })?;
    if !equal {
        problems.push("the portfolio differs from its backup outside the whitelist".to_owned());
    }
    Ok(())
}

fn stamps(check: &Check, problems: &mut Vec<String>) {
    let Some(backup) = check.backup else {
        return;
    };
    let count = |root: &Path| {
        fs::read_dir(root.join("dashboard")).map_or(0, |d| {
            d.filter_map(std::result::Result::ok)
                .filter(|e| e.file_name().to_string_lossy().starts_with("listen-"))
                .count()
        })
    };
    if count(backup) != count(check.home) {
        problems.push("the number of listen stamps changed".to_owned());
    }
}

/// The board checks.
pub fn check(check: &Check, problems: &mut Vec<String>) -> Result<()> {
    table(check, problems)?;
    for e in check.entries {
        one_board(check, e, problems)?;
    }
    portfolio_file(check, problems)?;
    stamps(check, problems);
    Ok(())
}
