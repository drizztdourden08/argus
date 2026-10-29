//! The cache as the page shows it: one row per cached version, and the totals.

use crate::entry::{Entry, Kind};
use crate::places::{Usage, User};
use crate::plan::is_prunable;
use crate::tmp::Tmp;
use argus_core::clock::iso_from_ms;
use serde::Serialize;
use std::collections::BTreeMap;

/// One cached version: every build key of it together.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub kind: Kind,
    pub name: String,
    pub version: String,
    pub build_keys: usize,
    pub users: Vec<User>,
    pub user_count: usize,
    pub size_bytes: u64,
    /// The newest last use of its build keys.
    pub last_used_at: String,
    pub in_use: bool,
    /// An unused row: when its newest build key has passed the grace period and all of them may go.
    pub delete_after: Option<String>,
    /// How many build keys a prune deletes now.
    pub prunable: usize,
    pub prunable_bytes: u64,
}

/// The sums of the cache.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub versions: usize,
    pub entries: usize,
    pub size_bytes: u64,
    pub in_use_bytes: u64,
    pub unused_bytes: u64,
    pub prunable_entries: usize,
    pub prunable_bytes: u64,
    pub abandoned_folders: usize,
    pub abandoned_bytes: u64,
}

/// What `GET /api/cache` answers with, apart from the last prune.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub root: String,
    pub exists: bool,
    pub grace_days: u32,
    pub rows: Vec<Row>,
    pub totals: Totals,
    /// Lockfiles that could not be read; a prune refuses to run while there are any.
    pub problems: Vec<String>,
}

/// The inputs of a report.
pub struct Survey<'a> {
    pub entries: &'a [Entry],
    pub usage: &'a Usage,
    pub stale_tmp: &'a [Tmp],
    pub now_ms: i64,
    pub grace_ms: i64,
}

fn row_of(group: &[&Entry], survey: &Survey) -> Row {
    let first = group[0];
    let users = survey.usage.users_of(&first.name, &first.version).to_vec();
    let newest = group.iter().map(|e| e.last_used_ms).max().unwrap_or(0);
    let prunable: Vec<&&Entry> = group
        .iter()
        .filter(|e| is_prunable(e, survey.usage, survey.now_ms, survey.grace_ms))
        .collect();
    Row {
        kind: first.kind,
        name: first.name.clone(),
        version: first.version.clone(),
        build_keys: group.len(),
        user_count: users.len(),
        in_use: !users.is_empty(),
        delete_after: users
            .is_empty()
            .then(|| iso_from_ms(newest + survey.grace_ms)),
        users,
        size_bytes: group.iter().map(|e| e.size_bytes).sum(),
        last_used_at: iso_from_ms(newest),
        prunable: prunable.len(),
        prunable_bytes: prunable.iter().map(|e| e.size_bytes).sum(),
    }
}

fn totals_of(rows: &[Row], survey: &Survey) -> Totals {
    let used = |in_use: bool| -> u64 {
        rows.iter()
            .filter(|r| r.in_use == in_use)
            .map(|r| r.size_bytes)
            .sum()
    };
    Totals {
        versions: rows.len(),
        entries: survey.entries.len(),
        size_bytes: rows.iter().map(|r| r.size_bytes).sum(),
        in_use_bytes: used(true),
        unused_bytes: used(false),
        prunable_entries: rows.iter().map(|r| r.prunable).sum(),
        prunable_bytes: rows.iter().map(|r| r.prunable_bytes).sum(),
        abandoned_folders: survey.stale_tmp.len(),
        abandoned_bytes: survey.stale_tmp.iter().map(|t| t.size_bytes).sum(),
    }
}

/// The rows (kind, name, version order) and totals of a survey.
pub fn rows_and_totals(survey: &Survey) -> (Vec<Row>, Totals) {
    let mut groups: BTreeMap<(Kind, &str, &str), Vec<&Entry>> = BTreeMap::new();
    for entry in survey.entries {
        groups
            .entry((entry.kind, &entry.name, &entry.version))
            .or_default()
            .push(entry);
    }
    let rows: Vec<Row> = groups.values().map(|g| row_of(g, survey)).collect();
    let totals = totals_of(&rows, survey);
    (rows, totals)
}
