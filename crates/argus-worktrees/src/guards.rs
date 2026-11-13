//! The guards every removal passes first: who asks, which rule covers the folder, what the folder is, who uses it.

use crate::board::{BoardFacts, Caller};
use crate::clock::{DAY_MS, HOUR_MS};
use crate::error::{ErrorCode, Result, WtError};
use crate::fact::Fact;
use crate::paths::{base_name, clean_path, is_below, parent_of, same_path};
use crate::ports::ProcInfo;
use crate::rule::{Rule, RuleKind};
use crate::service::Service;

/// How long one sample of the machine's processes serves a batch.
const PROCS_FRESH_MS: i64 = 10_000;

/// Brock locks every clone it makes with a reason that starts with this text. A lock with any other reason was set by
/// a person or another tool and keeps the worktree.
const BROCK_LOCK: &str = "cloned node_modules";

/// Is this lock reason Brock's?
pub(crate) fn is_brock_lock(reason: &str) -> bool {
    reason.starts_with(BROCK_LOCK)
}

/// A command line holding `worktree remove|prune` is the request itself (and the shell that runs it): not a user of the folder.
pub fn is_requester(cmd: &str) -> bool {
    let lower = cmd.to_lowercase();
    lower.match_indices("worktree").any(|(at, word)| {
        let before = lower[..at].chars().next_back();
        let rest = &lower[at + word.len()..];
        let spaced = rest.trim_start();
        let gap = spaced.len() < rest.len();
        before.is_none_or(|c| !is_word_char(c))
            && gap
            && ["remove", "prune"].iter().any(|w| starts_word(spaced, w))
    })
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn starts_word(text: &str, word: &str) -> bool {
    text.strip_prefix(word)
        .is_some_and(|after| !after.chars().next().is_some_and(is_word_char))
}

fn dotted_parent(path: &str) -> bool {
    let text = path.replace('\\', "/");
    text.match_indices("..")
        .any(|(at, _)| matches!(text[at + 2..].chars().next(), None | Some('/')))
}

/// Where a leftover may be: a direct child of a folder rule's folder, never deeper, never a rule's own folder, never a
/// short or odd path. The plain sentence of what is wrong, or none.
pub fn leftover_path_problem(rules: &[&Rule], rule: &Rule, path: &str, ci: bool) -> Option<String> {
    let clean = clean_path(path);
    let parts: Vec<&str> = clean.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() < 4 || parts.iter().any(|p| *p == "." || *p == "..") || dotted_parent(path) {
        return Some(format!(
            "{path} is too short or odd a path for Argus to delete anything (at least four path segments, no \".\" or \"..\"): refused"
        ));
    }
    if rule.kind != RuleKind::Folder {
        return Some(format!(
            "{path} is not under a folder rule: Argus only deletes leftovers directly inside a rule's folder"
        ));
    }
    if let Some(problem) = rule.problem() {
        return Some(format!(
            "rule {} ({}) is invalid: {problem}; a rule path is the parent folder. Nothing is deleted under it",
            rule.id, rule.path
        ));
    }
    if !same_path(&rule.path, &parent_of(path), ci) {
        return Some(format!(
            "{path} is not directly inside the folder of rule {} ({}): refused",
            rule.id, rule.path
        ));
    }
    if !rule.allows(&base_name(path), ci) {
        return Some(format!(
            "{} is excluded by rule {} (it is not in its --include list, or it is in its --exclude list): not a leftover of this rule",
            base_name(path),
            rule.id
        ));
    }
    if rules.iter().any(|r| same_path(&r.path, path, ci)) {
        return Some(format!(
            "{path} is itself the folder of a worktree rule: Argus never deletes a rule's folder"
        ));
    }
    rules.iter().find(|r| is_below(&r.path, path, ci)).map(|r| {
        format!(
            "{path} holds the folder of rule {} ({}): Argus never deletes it",
            r.id, r.path
        )
    })
}

fn forbidden(message: String) -> WtError {
    WtError::new(ErrorCode::Forbidden, message)
}

/// The thread that asks, checked against the board: the lead, or the thread of one of the rules.
pub fn check_caller(board: &BoardFacts, caller: &Caller, rules: &[&Rule]) -> Result<()> {
    let Caller::Thread(id) = caller else {
        return Ok(());
    };
    let Some(thread) = board.threads.get(id).filter(|t| t.live && !id.is_empty()) else {
        let who = if id.is_empty() {
            String::new()
        } else {
            format!(
                "; \"{}\" is not a live thread here",
                id.chars().take(40).collect::<String>()
            )
        };
        return Err(forbidden(format!(
            "only a registered thread of this project (its lead or the rule's thread) or the owner's page may remove a worktree{who}"
        )));
    };
    if thread.coordinator
        || rules
            .iter()
            .any(|r| r.thread.as_deref() == Some(thread.id.as_str()))
    {
        return Ok(());
    }
    let tells: Vec<&str> = rules.iter().filter_map(|r| r.thread.as_deref()).collect();
    let tells = if tells.is_empty() {
        "none".to_string()
    } else {
        tells.join(", ")
    };
    Err(forbidden(format!(
        "{id} may not remove it: only the project's lead or the thread its rule tells ({tells})"
    )))
}

/// The one-line form of a command for a sentence.
pub fn short_cmd(cmd: &str) -> String {
    cmd.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect()
}

/// Why a fact is outside its rule. A repo rule names only what git lists, so a folder git does not list gets the way to cover it.
fn outside_text(rule: &Rule, fact: &Fact) -> String {
    if rule.kind == RuleKind::Repo && fact.repo.is_none() {
        let parent = parent_of(&fact.path);
        return format!(
            "{} is not a worktree that git lists for {} (rule {} is a repo rule): a folder rule on its parent (coord worktree rule add {parent}/) covers leftovers",
            fact.path, rule.path, rule.id
        );
    }
    format!("{} is outside the project's worktree rules", fact.path)
}

impl Service {
    /// The rule a fact belongs to, and that it really is under it.
    pub(crate) fn rule_of(&self, board: &BoardFacts, fact: &Fact) -> Result<Rule> {
        if let Some(bad) = board
            .rules
            .iter()
            .find(|r| r.id == fact.rule && r.problem().is_some())
        {
            let problem = bad.problem().unwrap_or_default();
            return Err(forbidden(format!(
                "rule {} ({}) is invalid: {problem}. A rule path is the parent folder, never a wildcard; Argus ignores the rule and deletes nothing under it",
                bad.id, bad.path
            )));
        }
        let Some(rule) = board.rules.iter().find(|r| r.id == fact.rule) else {
            return Err(forbidden(format!(
                "{} is outside the project's worktree rules (its rule {} was removed)",
                fact.path, fact.rule
            )));
        };
        let ci = self.ci();
        let under = match rule.kind {
            RuleKind::Folder => {
                same_path(&rule.path, &parent_of(&fact.path), ci)
                    && rule.admits(&base_name(&fact.path), ci, fact.registered)
            }
            RuleKind::Repo => fact
                .repo
                .as_deref()
                .is_some_and(|repo| same_path(&rule.path, repo, ci)),
        };
        if under {
            Ok(rule.clone())
        } else {
            Err(forbidden(outside_text(rule, fact)))
        }
    }

    fn is_main_dir(&self, path: &str) -> bool {
        self.deps
            .fs
            .is_dir(&format!("{}/.git", path.trim_end_matches('/')))
    }

    /// Never a repository's main checkout.
    pub(crate) fn assert_not_main(&self, fact: &Fact) -> Result<()> {
        let is_repo = fact
            .repo
            .as_deref()
            .is_some_and(|repo| same_path(repo, &fact.path, self.ci()));
        if fact.main || self.is_main_dir(&fact.path) || is_repo {
            return Err(forbidden(format!(
                "{} is a repository's main checkout: Argus never removes one",
                fact.path
            )));
        }
        Ok(())
    }

    /// Never a locked worktree. The one lock Argus lifts itself is Brock's (see [`is_brock_lock`]), and only for a removal.
    pub(crate) fn assert_not_locked(fact: &Fact, removing: bool) -> Result<()> {
        match &fact.locked {
            Some(reason) if !(removing && is_brock_lock(reason)) => Err(forbidden(format!(
                "{} is locked ({reason}): unlock it first (git worktree unlock), Argus never removes a locked worktree",
                fact.path
            ))),
            _ => Ok(()),
        }
    }

    fn processes(&self, cache: bool) -> std::result::Result<Vec<ProcInfo>, String> {
        let now = self.now();
        if cache {
            if let Some((at, list)) = &self.st().proc_cache {
                if now - at < PROCS_FRESH_MS {
                    return Ok(list.clone());
                }
            }
        }
        let list = self.deps.procs.list()?;
        if cache {
            self.st().proc_cache = Some((now, list.clone()));
        }
        Ok(list)
    }

    /// A live process that has the folder in its command line, executable path or working directory.
    pub(crate) fn busy_by(
        &self,
        path: &str,
        cache: bool,
        nothing: &str,
    ) -> Result<Option<ProcInfo>> {
        let list = self.processes(cache).map_err(|err| {
            let said: String = err.chars().take(120).collect();
            WtError::new(ErrorCode::InUse, format!("could not list the running processes to check that nothing uses {path} ({said}): nothing was {nothing}"))
        })?;
        let ci = self.ci();
        let uses = |p: &ProcInfo| {
            let named = |text: &str| crate::approval::names_folder(text, path, ci);
            p.pid != std::process::id()
                && !is_requester(&p.cmd)
                && (named(&p.cmd)
                    || p.path.as_deref().is_some_and(named)
                    || p.cwd.as_deref().is_some_and(named))
        };
        Ok(list.into_iter().find(uses))
    }

    /// The refusal for a folder a process uses.
    pub(crate) fn in_use(path: &str, user: &ProcInfo, nothing: &str) -> WtError {
        let name = base_name(if user.name.is_empty() {
            "a process"
        } else {
            &user.name
        });
        let cmd = if user.cmd.is_empty() {
            String::new()
        } else {
            format!(" ({})", short_cmd(&user.cmd))
        };
        WtError::new(
            ErrorCode::InUse,
            format!(
                "{path} is in use by {name} {}{cmd}: close it first, nothing was {nothing}",
                user.pid
            ),
        )
    }

    /// Hours or days, as a sentence: 1 -> `1 day`, 3 -> `3 days`, 0.25 -> `6 hours`.
    pub(crate) fn duration_text(days: f64) -> String {
        if days >= 1.0 {
            let rounded = (days * 100.0).round() / 100.0;
            return format!(
                "{rounded} day{}",
                if (days - 1.0).abs() < f64::EPSILON {
                    ""
                } else {
                    "s"
                }
            );
        }
        #[allow(clippy::cast_possible_truncation)]
        let hours = (days * 24.0).round() as i64;
        format!("{hours} hour{}", if hours == 1 { "" } else { "s" })
    }

    /// `3 hours ago` / `12 days ago`, from a file time.
    pub(crate) fn ago_text(&self, ms: Option<i64>) -> String {
        let Some(ms) = ms else {
            return "at an unknown time".to_string();
        };
        let diff = (self.now() - ms).max(0);
        let hours = ((diff + HOUR_MS / 2) / HOUR_MS).max(1);
        if hours < 48 {
            format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" })
        } else {
            format!("{} days ago", diff / DAY_MS)
        }
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn r1() -> Rule {
        Rule::folder("W-1", "X:/Bxel/.claude/worktrees", "lead")
    }

    #[test]
    fn the_path_rules_on_their_own() {
        let r1 = r1();
        let problem = |rules: &[&Rule], rule: &Rule, path: &str| {
            leftover_path_problem(rules, rule, path, true)
        };
        assert_eq!(
            problem(&[&r1], &r1, "X:/Bxel/.claude/worktrees/agent-ab12"),
            None
        );
        assert!(
            problem(&[&r1], &r1, "X:/Bxel/other/agent")
                .unwrap()
                .contains("is not directly inside the folder of rule W-1")
        );
        assert!(
            problem(&[&r1], &r1, "X:/a/b")
                .unwrap()
                .contains("too short or odd a path")
        );
        assert!(
            problem(&[&r1], &r1, "X:/Bxel/.claude/worktrees/../agent")
                .unwrap()
                .contains("too short or odd")
        );
        assert!(
            problem(&[&r1], &r1, "X:/")
                .unwrap()
                .contains("too short or odd")
        );
        assert!(
            problem(&[&r1], &r1, "X:/Bxel/.claude/worktrees/agent-ab12/deeper")
                .unwrap()
                .contains("is not directly inside the folder of rule W-1")
        );
        let r2 = Rule::folder("W-2", "X:/Bxel/.claude/worktrees/agent-ab12", "lead");
        assert!(
            problem(&[&r1, &r2], &r1, "X:/Bxel/.claude/worktrees/agent-ab12")
                .unwrap()
                .contains("is itself the folder of a worktree rule")
        );
        let r3 = Rule::folder("W-3", "X:/Bxel/.claude/worktrees/agent-ab12/inner", "lead");
        assert!(
            problem(&[&r1, &r3], &r1, "X:/Bxel/.claude/worktrees/agent-ab12")
                .unwrap()
                .contains("holds the folder of rule W-3")
        );
        let repo = Rule::folder("W-4", "X:/Bxel", "lead").repo();
        assert!(
            problem(&[&repo], &repo, "X:/Bxel/.claude/worktrees/agent")
                .unwrap()
                .contains("not under a folder rule")
        );
    }

    #[test]
    fn a_folder_under_a_repo_rule_that_git_does_not_list_says_how_to_cover_it() {
        let repo = Rule::folder("W-4", "X:/Bxel", "lead").repo();
        let mut fact = crate::fact::blank_fact("X:/Bxel/leftovers/old", &repo, false);
        let message = outside_text(&repo, &fact);
        assert_eq!(
            message,
            "X:/Bxel/leftovers/old is not a worktree that git lists for X:/Bxel (rule W-4 is a repo rule): a folder rule on its parent (coord worktree rule add X:/Bxel/leftovers/) covers leftovers"
        );
        fact.repo = Some("X:/Other".to_string());
        assert!(outside_text(&repo, &fact).ends_with("is outside the project's worktree rules"));
        let folder = r1();
        fact.repo = None;
        assert!(outside_text(&folder, &fact).ends_with("is outside the project's worktree rules"));
    }

    #[test]
    fn a_wildcard_rule_and_the_name_filters_are_refused_by_the_path_rules() {
        let glob = Rule::folder("W-1", "X:/*/.claude/worktrees", "lead");
        let message =
            leftover_path_problem(&[&glob], &glob, "X:/Tessera/.claude/worktrees/a1", true)
                .unwrap();
        assert!(
            message.contains("rule W-1") && message.contains("is invalid: wildcard path"),
            "{message}"
        );
        let web = r1().filtered(&["web-*"], &[]);
        assert_eq!(
            leftover_path_problem(&[&web], &web, "X:/Bxel/.claude/worktrees/web-1", true),
            None
        );
        assert!(
            leftover_path_problem(&[&web], &web, "X:/Bxel/.claude/worktrees/agent-1", true)
                .unwrap()
                .contains("excluded by rule W-1")
        );
        let not_web = r1().filtered(&[], &["web-*"]);
        assert!(
            leftover_path_problem(
                &[&not_web],
                &not_web,
                "X:/Bxel/.claude/worktrees/WEB-2",
                true
            )
            .unwrap()
            .contains("excluded by rule W-1")
        );
        assert_eq!(
            leftover_path_problem(
                &[&not_web],
                &not_web,
                "X:/Bxel/.claude/worktrees/agent-1",
                true
            ),
            None
        );
    }

    #[test]
    fn the_requester_is_not_a_user_but_a_sibling_folder_is_not_the_requester() {
        assert!(is_requester("bash -c \"coord worktree remove X:/a/b\""));
        assert!(is_requester("coord WORKTREE   prune"));
        assert!(!is_requester("node X:/a/worktree-other/x.js"));
        assert!(!is_requester("git worktree list"));
    }

    #[test]
    fn who_may_ask() {
        let board = crate::testkit::board_with(Vec::new());
        let rule = Rule::folder("W-1", "X:/a", "worker");
        let ask = |caller: Caller| check_caller(&board, &caller, &[&rule]);
        assert!(
            ask(Caller::Page).is_ok()
                && ask(Caller::Thread("lead".into())).is_ok()
                && ask(Caller::Thread("worker".into())).is_ok()
        );
        let other = ask(Caller::Thread("other".into())).unwrap_err();
        assert!(other.message.contains(
            "other may not remove it: only the project's lead or the thread its rule tells (worker)"
        ));
        assert!(
            ask(Caller::Thread("ended".into()))
                .unwrap_err()
                .message
                .contains("\"ended\" is not a live thread here")
        );
        assert_eq!(
            ask(Caller::Thread(String::new())).unwrap_err().code,
            ErrorCode::Forbidden
        );
    }

    fn busy(case_insensitive: bool, path: &str, cwd: &str) -> Option<ProcInfo> {
        let dir = tempfile::tempdir().unwrap();
        let root = crate::testkit::root_of(&dir);
        let h = crate::testkit::Harness::new(&root, Vec::new(), |o| {
            o.case_insensitive = case_insensitive;
        });
        *h.procs.0.lock().unwrap() = vec![ProcInfo {
            pid: 4242,
            name: "pwsh.exe".into(),
            cwd: Some(cwd.into()),
            ..ProcInfo::default()
        }];
        h.svc.busy_by(path, false, "removed").unwrap()
    }

    #[test]
    fn a_working_folder_inside_the_folder_makes_it_busy_and_the_refusal_names_the_process() {
        let path = "X:/p/.claude/worktrees/agent-ab12";
        let user = busy(true, path, "x:\\P\\.claude\\Worktrees\\Agent-AB12\\src").unwrap();
        assert_eq!(user.pid, 4242);
        let text = Service::in_use(path, &user, "removed").message;
        assert!(text.contains("is in use by pwsh.exe 4242"), "{text}");
        assert!(busy(true, path, "X:\\p\\.claude\\worktrees\\agent-ab12").is_some());
        assert!(busy(true, path, "X:\\p\\.claude\\worktrees\\agent-ab12-other").is_none());
        assert!(busy(true, path, "X:\\p\\.claude\\worktrees").is_none());
        assert!(busy(false, path, "X:/P/.claude/worktrees/AGENT-AB12").is_none());
    }
}
