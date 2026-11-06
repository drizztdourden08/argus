//! Is a worktree attached? Who owns it, what holds it, and what removing it would lose.

use super::memory::{Exception, Refused};
use crate::approval::{mentions, names_folder};
use crate::board::{AgentFact, BoardFacts, Link};
use crate::fact::Fact;
use crate::guards::is_brock_lock;
use crate::linking::{agent_item, resolve};
use crate::paths::{base_name, clean_path, is_below, same_path};
use crate::remove::{Safety, plural};
use crate::service::Service;

/// What the hygiene makes of one worktree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not the hygiene's: a main checkout, a folder its repository does not list, one outside its rule, a leftover
    /// that holds files.
    Quiet,
    /// A running agent owns it, or work in progress, a launcher, a bundle, a run or a live process is in it (or Argus
    /// cannot tell, which counts the same).
    Attached(String),
    /// On the Overlord's exception list, with its reason.
    Exception(String),
    /// Nothing is attached and removing it loses nothing.
    Clean,
    /// Nothing is attached but something would be lost: the sentence says what.
    Left(String),
}

/// Who a verdict is about: the agent that owned the worktree and the lead who is asked.
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
    pub exception: Option<&'a Exception>,
}

/// What the hygiene judges: a linked git worktree, or a leftover folder with no file in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Worktree,
    EmptyLeftover,
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

/// Does this open thing work in the worktree: its folder, its branch, or its text names it?
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

/// A lock that is not Brock's: nothing removes the worktree while it holds.
fn lock_why(fact: &Fact) -> Option<String> {
    fact.locked
        .as_deref()
        .filter(|r| !is_brock_lock(r))
        .map(|reason| format!("it is locked ({reason})"))
}

/// What removing it would lose or what stops a plain removal, or none.
fn left_why(fact: &Fact) -> Option<String> {
    if let Some(why) = lock_why(fact) {
        return Some(why);
    }
    if fact.dirty > 0 {
        return Some(format!(
            "it has {}",
            plural(fact.dirty, "uncommitted change")
        ));
    }
    (!Safety::of(fact).kept).then(|| "its HEAD has commits no branch or ref keeps".to_string())
}

/// What the hygiene looks at: a readable git worktree that is not a main checkout, or an empty leftover.
fn shape_of(fact: &Fact) -> Option<Shape> {
    if !fact.exists || fact.error.is_some() {
        return None;
    }
    if fact.is_git {
        return (fact.leftover.is_none() && !fact.main).then_some(Shape::Worktree);
    }
    let empty = fact.leftover.as_ref().is_some_and(|l| {
        l.files == Some(0) && !l.pending && !l.partial && !l.is_link && !l.has_nested_git
    });
    empty.then_some(Shape::EmptyLeftover)
}

impl Service {
    /// The lead asked about a worktree: the thread of its last agent while that thread lives, else the thread of the
    /// item it belongs to, else the thread of its rule, else the board's lead.
    pub(crate) fn owning_lead(
        &self,
        board: &BoardFacts,
        fact: &Fact,
        owners: &[&AgentFact],
    ) -> Option<String> {
        let live = |t: Option<&str>| t.and_then(|t| board.live_thread(t));
        let item_thread = || {
            let link = resolve(board, fact, agent_item(owners), self.ci())?;
            live(board.items.get(&link.item)?.thread.as_deref())
        };
        let rule_thread = || {
            let rule = board.rules.iter().find(|r| r.id == fact.rule)?;
            live(rule.thread.as_deref())
        };
        live(owners.last().and_then(|a| a.thread.as_deref()))
            .or_else(item_thread)
            .or_else(rule_thread)
            .or_else(|| board.coordinator())
    }

    /// The agents that own the worktree, the last to end last (a running one after all).
    pub(crate) fn owners<'a>(&self, board: &'a BoardFacts, fact: &Fact) -> Vec<&'a AgentFact> {
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

    /// A running agent owns it, or an agent that ended after the last scan has not been looked at since (it counts as
    /// attached until the rescan it asks for).
    fn held_by_agent(&self, look: &Look<'_>, project: &str, owners: &[&AgentFact]) -> Option<String> {
        if let Some(running) = owners.iter().find(|a| a.live()) {
            return Some(format!("its agent {} is {}", running.id, running.status));
        }
        let ended = owners.iter().filter_map(|a| a.ended_ms).max()?;
        (ended > look.scanned_ms).then(|| {
            self.request_rescan(project);
            "its agent ended after the last scan: Argus looks at it again first".to_string()
        })
    }

    /// What is attached besides its agents: an open thing that works in it, a run in it, a process that uses it. A
    /// list Argus cannot read counts as attached.
    fn attached_why(&self, look: &Look<'_>, fact: &Fact) -> Option<String> {
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
            return Some(format!("{} works in it", link.what));
        }
        match self.busy_by(&fact.path, !look.fresh, "removed") {
            Err(err) => Some(err.message),
            Ok(Some(user)) => Some(format!("{} {} uses it", base_name(&user.name), user.pid)),
            Ok(None) => None,
        }
    }

    /// Not the hygiene's at all: a main checkout, a folder its repository does not list, one outside its rule.
    fn outside(&self, board: &BoardFacts, fact: &Fact) -> bool {
        self.assert_not_main(fact).is_err() || !fact.registered || self.rule_of(board, fact).is_err()
    }

    /// The verdict on one worktree and whom it is about, whatever its age.
    pub(crate) fn judge(&self, project: &str, look: &Look<'_>, fact: &Fact) -> (Verdict, Owner) {
        let Some(shape) = shape_of(fact) else {
            return (Verdict::Quiet, Owner::default());
        };
        if shape == Shape::Worktree && self.outside(look.board, fact) {
            return (Verdict::Quiet, Owner::default());
        }
        let owners = self.owners(look.board, fact);
        let owner = Owner {
            agent: owners.last().map(|a| a.id.clone()),
            thread: self.owning_lead(look.board, fact, &owners),
        };
        if let Some(exception) = look.exception {
            return (Verdict::Exception(exception.reason.clone()), owner);
        }
        let attached = self
            .held_by_agent(look, project, &owners)
            .or_else(|| self.attached_why(look, fact));
        if let Some(why) = attached {
            return (Verdict::Attached(why), owner);
        }
        (Self::free_verdict(look, fact, shape), owner)
    }

    /// Nothing is attached: clean, or what would be lost (what Argus could not remove before counts while the
    /// worktree stays as it was).
    fn free_verdict(look: &Look<'_>, fact: &Fact, shape: Shape) -> Verdict {
        if let Some(refused) = look
            .refused
            .filter(|r| r.head == fact.head && r.dirty == fact.dirty)
        {
            return Verdict::Left(format!("Argus could not remove it: {}", refused.why));
        }
        if shape == Shape::EmptyLeftover {
            return Verdict::Clean;
        }
        left_why(fact).map_or(Verdict::Clean, Verdict::Left)
    }
}

// test-tag: critical
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
