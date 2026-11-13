//! The automatic repo rules of the registry: one per project root with no repo rule, and one per `worktrees.extraRepos`
//! path. Computed from the boards on demand and kept a few seconds; never written into a board.

use std::path::Path;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use argus_worktrees::{AutoBoard, AutoMode, AutoRule, ExtraRepo, Rule, auto_rules};
use serde_json::Value;

use super::facts::rules_of;
use crate::boards::BoardReader;

const KEEP: Duration = Duration::from_secs(5);

/// The settings of the automatic rules (`worktrees.autoRules` and `worktrees.extraRepos`).
#[derive(Default)]
pub struct AutoSetting {
    pub mode: AutoMode,
    pub extras: Vec<ExtraRepo>,
}

/// What the derivation reads: the boards, the settings and the last result.
pub(super) struct AutoRepos {
    setting: AutoSetting,
    kept: Mutex<Option<(Instant, Vec<AutoRule>)>>,
}

fn is_repo(root: &str) -> bool {
    Path::new(root).join(".git").exists()
}

impl AutoRepos {
    pub(super) fn new(setting: AutoSetting) -> Self {
        AutoRepos {
            setting,
            kept: Mutex::new(None),
        }
    }

    /// Each board with a board file: its key, its root when it is a live git repository, its rules.
    fn seen(boards: &BoardReader) -> Vec<(String, Option<String>, Vec<Rule>)> {
        boards
            .projects()
            .into_iter()
            .filter(|p| p.has_board)
            .map(|p| {
                let board = boards.board(&p.key);
                let archived = board.as_ref().is_some_and(|b| b["archived"] == true);
                let rules = board.map_or_else(Vec::new, |b| rules_of(&b["worktreeRules"]));
                let root = p.root.filter(|r| !archived && is_repo(r));
                (p.key, root, rules)
            })
            .collect()
    }

    fn derive(&self, boards: &BoardReader) -> Vec<AutoRule> {
        let seen = Self::seen(boards);
        let view: Vec<AutoBoard<'_>> = seen
            .iter()
            .map(|(key, root, rules)| AutoBoard {
                key,
                root: root.as_deref(),
                rules,
            })
            .collect();
        auto_rules(
            &view,
            &self.setting.extras,
            self.setting.mode,
            cfg!(windows),
        )
    }

    /// The automatic rules the project owns.
    pub(super) fn of(&self, boards: &BoardReader, project: &str) -> Vec<Rule> {
        let mut kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
        if kept.as_ref().is_none_or(|(at, _)| at.elapsed() >= KEEP) {
            *kept = Some((Instant::now(), self.derive(boards)));
        }
        let all = kept.as_ref().map_or(&[][..], |(_, rules)| rules.as_slice());
        all.iter()
            .filter(|a| a.owner == project)
            .map(|a| a.rule.clone())
            .collect()
    }
}

/// The text of a setting value as a list of repos with their owner.
#[must_use]
pub fn extras_of(raw: &Value, default_owner: &str) -> Vec<ExtraRepo> {
    raw.as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let path = e["path"]
                .as_str()
                .map(str::trim)
                .filter(|p| !p.is_empty())?;
            let owner = e["owner"].as_str().filter(|o| !o.is_empty());
            Some(ExtraRepo {
                path: path.to_owned(),
                owner: owner.unwrap_or(default_owner).to_owned(),
            })
        })
        .collect()
}
