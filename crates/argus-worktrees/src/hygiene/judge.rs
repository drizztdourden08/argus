//! Is a worktree released? Who owns it, what links it, and whether removing it is safe.

use super::memory::Refused;
use crate::approval::{mentions, names_folder};
use crate::board::{AgentFact, BoardFacts, Caller, Link};
use crate::fact::Fact;
use crate::guards::{check_caller, is_brock_lock};
use crate::paths::{base_name, clean_path, is_below, same_path};
use crate::remove::{Safety, plural};
use crate::service::Service;

/// What the hygiene makes of one worktree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not the hygiene's: no sub-agent owns it, or one still works in it.
    Quiet,
    /// Every condition holds.
    Released,
    /// The thread keeps it.
    Kept(Option<String>),
    /// Something keeps Argus from removing it.
    Review(String),
}

/// Who a verdict is about: the agent that owned the worktree and the thread that is told.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Owner {
    pub agent: Option<String>,
    pub thread: Option<String>,
}

/// What a judging reads besides the fact.
pub(crate) struct Look<'a> {
    pub board: &'a BoardFacts,
    pub runs: &'a Result<Vec<Link>, String>,
    /// When the last scan read the facts.
    pub scanned_ms: i64,
    /// Right before a removal: the processes are listed afresh, not from the short cache.
    pub fresh: bool,
    pub refused: Option<&'a Refused>,
}

fn absolute(path: &str) -> bool {
    let clean = clean_path(path);
    let bytes = clean.as_bytes();
    clean.starts_with('/') || (bytes.len() >= 3 && bytes[1] == b':' && bytes[2] == b'/')
}

/// Does this agent record own the worktree: its `worktree` names it, or it is the agent's own folder?
fn owns(agent: &AgentFact, fact: &Fact, ci: bool) -> bool {
    let name = base_name(&fact.path);
    let same = |a: &str, b: &str| {
        if ci {
            a.eq_ignore_ascii_case(b)
        } else {
            a == b
        }
    };
    let own = agent
        .claude_id
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .is_some_and(|c| same(&name, &format!("agent-{c}")));
    let named = agent
        .worktree
        .as_deref()
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .is_some_and(|w| {
            if absolute(w) {
                same_path(w, &fact.path, ci)
            } else {
                same(&clean_path(w), &name)
            }
        });
    own || named
}

/// Does this open thing use the worktree: its folder, its branch, or its text names it?
fn links_to(link: &Link, fact: &Fact, ci: bool) -> bool {
    let folder = link
        .path
        .as_deref()
        .is_some_and(|p| same_path(p, &fact.path, ci) || is_below(p, &fact.path, ci));
    let branch = link.branch.is_some() && link.branch == fact.branch;
    let named = link.text.as_deref().is_some_and(|t| {
        names_folder(t, &fact.path, ci)
            || mentions(t, &base_name(&fact.path))
            || fact.branch.as_deref().is_some_and(|b| mentions(t, b))
    });
    folder || branch || named
}

/// Why removing it is never automatic, or none.
fn unsafe_why(fact: &Fact) -> Option<String> {
    if let Some(reason) = fact.locked.as_deref().filter(|r| !is_brock_lock(r)) {
        return Some(format!("it is locked ({reason})"));
    }
    if fact.dirty > 0 {
        return Some(format!(
            "it has {}",
            plural(fact.dirty, "uncommitted change")
        ));
    }
    (!Safety::of(fact).kept).then(|| "its HEAD has commits no branch or ref keeps".to_string())
}

impl Service {
    /// The thread told about a worktree: the owning agent's when it may remove it, else the rule's, else the lead.
    fn told_thread(board: &BoardFacts, fact: &Fact, agent: Option<&AgentFact>) -> Option<String> {
        let rule = board.rules.iter().find(|r| r.id == fact.rule);
        let rules = Self::caller_rules(board, rule);
        let may = |t: &String| check_caller(board, &Caller::Thread(t.clone()), &rules).is_ok();
        agent
            .and_then(|a| a.thread.clone())
            .filter(may)
            .or_else(|| {
                rule.and_then(|r| r.thread.as_deref())
                    .and_then(|t| board.live_thread(t))
            })
            .or_else(|| board.coordinator())
    }

    /// The agents that own the worktree, and the one the verdict names (the last to end).
    fn owners<'a>(&self, board: &'a BoardFacts, fact: &Fact) -> Vec<&'a AgentFact> {
        let mut found: Vec<&AgentFact> = board
            .agents
            .iter()
            .filter(|a| owns(a, fact, self.ci()))
            .collect();
        found.sort_by_key(|a| a.ended_ms.unwrap_or(i64::MAX));
        found
    }

    /// Is the worktree a git work tree on disk the hygiene may look at (not a leftover, readable)?
    pub(crate) fn judged(fact: &Fact) -> bool {
        fact.exists && fact.is_git && fact.leftover.is_none() && fact.error.is_none()
    }

    /// Condition (a): its agents all ended, and the scan looked after that. `None` when the hygiene says nothing.
    fn not_needed(
        &self,
        look: &Look<'_>,
        project: &str,
        owners: &[&AgentFact],
    ) -> Option<Result<(), String>> {
        if owners.is_empty() || owners.iter().any(|a| a.live()) {
            return None;
        }
        if let Some(other) = owners.iter().find(|a| !a.ended()) {
            return Some(Err(format!(
                "its agent {} is {}: Argus removes the worktree of a done or stopped agent only",
                other.id, other.status
            )));
        }
        let ended = owners.iter().filter_map(|a| a.ended_ms).max().unwrap_or(0);
        if ended > look.scanned_ms {
            self.request_rescan(project);
            return Some(Err(
                "its agent ended after the last scan: Argus looks at it again first".to_string(),
            ));
        }
        Some(Ok(()))
    }

    /// Condition (b): no open thing names it and no process uses it.
    fn linked_why(&self, look: &Look<'_>, fact: &Fact) -> Option<String> {
        let runs = match look.runs {
            Ok(runs) => runs,
            Err(err) => return Some(format!("the runs could not be read ({err})")),
        };
        let ci = self.ci();
        if let Some(link) = look
            .board
            .links
            .iter()
            .chain(runs)
            .find(|l| links_to(l, fact, ci))
        {
            return Some(format!("{} uses it", link.what));
        }
        match self.busy_by(&fact.path, !look.fresh, "removed") {
            Err(err) => Some(err.message),
            Ok(Some(user)) => Some(format!("{} {} uses it", base_name(&user.name), user.pid)),
            Ok(None) => None,
        }
    }

    /// Condition (c) and the never-automatic cases.
    fn safety_why(&self, board: &BoardFacts, fact: &Fact) -> Option<String> {
        if let Err(err) = self.assert_not_main(fact) {
            return Some(err.message);
        }
        if !fact.registered {
            return Some("it is not a linked worktree its repository lists".to_string());
        }
        if let Err(err) = self.rule_of(board, fact) {
            return Some(err.message);
        }
        unsafe_why(fact)
    }

    /// Conditions (b) and (c): what a thread's own release is checked against too.
    pub(crate) fn blocked_why(&self, look: &Look<'_>, fact: &Fact) -> Option<String> {
        self.safety_why(look.board, fact)
            .or_else(|| self.linked_why(look, fact))
    }

    /// The verdict on one worktree and whom it is about.
    pub(crate) fn judge(&self, project: &str, look: &Look<'_>, fact: &Fact) -> (Verdict, Owner) {
        if !Self::judged(fact) {
            return (Verdict::Quiet, Owner::default());
        }
        let owners = self.owners(look.board, fact);
        let Some(needed) = self.not_needed(look, project, &owners) else {
            return (Verdict::Quiet, Owner::default());
        };
        let last = owners.last().copied();
        let owner = Owner {
            agent: last.map(|a| a.id.clone()),
            thread: Self::told_thread(look.board, fact, last),
        };
        if let Err(why) = needed {
            return (Verdict::Review(why), owner);
        }
        let verdict = self.verdict_of(look, fact);
        (verdict, owner)
    }

    fn verdict_of(&self, look: &Look<'_>, fact: &Fact) -> Verdict {
        if let Some(refused) = look
            .refused
            .filter(|r| r.head == fact.head && r.dirty == fact.dirty)
        {
            return Verdict::Review(format!("Argus could not remove it: {}", refused.why));
        }
        if let Some(keep) = look
            .board
            .keeps
            .get(&fact.id)
            .filter(|k| k.until_ms > self.now())
        {
            return Verdict::Kept(keep.note.clone());
        }
        match self.blocked_why(look, fact) {
            Some(why) => Verdict::Review(why),
            None => Verdict::Released,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::blank_fact;
    use crate::rule::Rule;

    fn fact_at(path: &str, branch: Option<&str>) -> Fact {
        let mut fact = blank_fact(
            path,
            &Rule::folder("W-1", "X:/p/.claude/worktrees", "t"),
            true,
        );
        fact.branch = branch.map(String::from);
        fact
    }

    fn agent(claude: Option<&str>, worktree: Option<&str>) -> AgentFact {
        AgentFact {
            id: "A-1".into(),
            claude_id: claude.map(String::from),
            worktree: worktree.map(String::from),
            status: "done".into(),
            ..AgentFact::default()
        }
    }

    #[test]
    fn an_agent_owns_its_own_folder_and_the_one_it_names() {
        let fact = fact_at("X:/p/.claude/worktrees/agent-a12", None);
        assert!(owns(&agent(Some("a12"), None), &fact, true));
        assert!(owns(&agent(None, Some("agent-a12")), &fact, true));
        assert!(owns(
            &agent(None, Some("X:\\p\\.claude\\worktrees\\agent-a12")),
            &fact,
            true
        ));
        assert!(!owns(&agent(Some("a1"), None), &fact, true));
        assert!(!owns(&agent(None, Some("X:/q/agent-a12")), &fact, true));
        assert!(!owns(&agent(None, None), &fact, true));
    }

    #[test]
    fn a_link_names_the_folder_the_branch_or_the_name() {
        let fact = fact_at(
            "X:/p/.claude/worktrees/agent-a12",
            Some("worktree-agent-a12"),
        );
        let link = |path: Option<&str>, branch: Option<&str>, text: Option<&str>| Link {
            what: "x".into(),
            path: path.map(String::from),
            branch: branch.map(String::from),
            text: text.map(String::from),
        };
        assert!(links_to(
            &link(Some("X:/p/.claude/worktrees/agent-a12/web"), None, None),
            &fact,
            true
        ));
        assert!(links_to(
            &link(None, Some("worktree-agent-a12"), None),
            &fact,
            true
        ));
        assert!(links_to(
            &link(None, None, Some("land agent-a12 first")),
            &fact,
            true
        ));
        assert!(!links_to(
            &link(Some("X:/p"), Some("main"), Some("agent-a123")),
            &fact,
            true
        ));
    }
}
