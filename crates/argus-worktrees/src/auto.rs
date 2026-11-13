//! Automatic repo rules: every project root that no hand repo rule covers is scanned as a repository, so the sibling
//! worktrees git registers outside `.claude/worktrees` are seen. The rules are derived, never written into a board.

use crate::paths::same_path;
use crate::rule::{Rule, RuleKind};

/// Whether the derived rules exist and whether the hygiene acts on them (`worktrees.autoRules`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AutoMode {
    /// Scanned, shown and classified; the hygiene removes, prunes and asks nothing, and the plan lists its verdicts.
    #[default]
    Report,
    /// They behave like hand rules.
    On,
    /// They are not made.
    Off,
}

impl AutoMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Report => "report",
            Self::On => "on",
            Self::Off => "off",
        }
    }

    /// `report`, `on` or `off`; anything else is the default.
    pub fn parse(text: &str) -> Self {
        match text {
            "on" => Self::On,
            "off" => Self::Off,
            _ => Self::Report,
        }
    }
}

/// A board as far as the derivation cares.
pub struct AutoBoard<'a> {
    pub key: &'a str,
    /// The project root, when it is a git repository.
    pub root: Option<&'a str>,
    pub rules: &'a [Rule],
}

/// A repository with no board (`worktrees.extraRepos`): the board whose lead gets the asks.
pub struct ExtraRepo {
    pub path: String,
    pub owner: String,
}

/// A derived rule and the board that owns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutoRule {
    pub owner: String,
    pub rule: Rule,
}

fn repo_rule(id: String, path: &str, owner: &str) -> Rule {
    Rule {
        auto: true,
        note: Some("automatic".to_string()),
        ..Rule::folder(&id, path, owner).repo()
    }
}

fn covered(made: &[AutoRule], hand: &[&Rule], path: &str, ci: bool) -> bool {
    hand.iter().any(|r| same_path(&r.path, path, ci))
        || made.iter().any(|a| same_path(&a.rule.path, path, ci))
}

/// One repo rule per root that has none: a board with a repo rule of its own gets none, two boards sharing a root get
/// one (the first owns it), and the listed extra repos are added the same way.
pub fn auto_rules(
    boards: &[AutoBoard<'_>],
    extras: &[ExtraRepo],
    mode: AutoMode,
    ci: bool,
) -> Vec<AutoRule> {
    if mode == AutoMode::Off {
        return Vec::new();
    }
    let hand: Vec<&Rule> = boards
        .iter()
        .flat_map(|b| b.rules)
        .filter(|r| r.kind == RuleKind::Repo)
        .collect();
    let mut made: Vec<AutoRule> = Vec::new();
    for board in boards {
        let own = board.rules.iter().any(|r| r.kind == RuleKind::Repo);
        let Some(root) = board.root.filter(|r| !own && !r.trim().is_empty()) else {
            continue;
        };
        if !covered(&made, &hand, root, ci) {
            let rule = repo_rule(format!("auto-{}", board.key), root, board.key);
            made.push(AutoRule {
                owner: board.key.to_string(),
                rule,
            });
        }
    }
    for (n, extra) in extras.iter().enumerate() {
        if !covered(&made, &hand, &extra.path, ci) {
            let rule = repo_rule(format!("auto-extra-{}", n + 1), &extra.path, &extra.owner);
            made.push(AutoRule {
                owner: extra.owner.clone(),
                rule,
            });
        }
    }
    made
}
