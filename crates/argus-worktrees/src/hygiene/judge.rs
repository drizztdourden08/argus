//! Is a worktree released? Who owns it, what links it, and whether removing it is safe.

use super::memory::Refused;
use crate::approval::{mentions, names_folder};
use crate::board::{AgentFact, BoardFacts, Caller, Link};
use crate::fact::Fact;
use crate::guards::{check_caller, is_brock_lock};
use crate::linking::{agent_item, resolve};
use crate::paths::{base_name, clean_path, is_below, same_path};
use crate::remove::{Safety, plural};
use crate::service::Service;

/// What the hygiene makes of one worktree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not the hygiene's: a main checkout, a folder that is not a work tree, a leftover that holds files.
    Quiet,
    /// A running agent owns it, or a process, a run, a launcher, a pull request or work in progress holds it.
    InUse(String),
    /// Nothing holds it and removing it loses nothing.
    Released,
    /// Its item closed and nothing holds it: it goes after the grace, its full backup first.
    Closing,
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
    /// The item it belongs to, when that item is closed.
    pub item: Option<String>,
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

/// What the hygiene judges: a linked git worktree, or a leftover folder with no file in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Worktree,
    EmptyLeftover,
}

/// What holds a worktree besides its agents.
enum Linked {
    Free,
    /// Something uses it: the sentence says what.
    Uses(String),
    /// Argus could not tell (the runs or the processes could not be read).
    Unknown(String),
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

/// A lock that is not Brock's: nothing removes the worktree while it holds.
fn lock_why(fact: &Fact) -> Option<String> {
    fact.locked
        .as_deref()
        .filter(|r| !is_brock_lock(r))
        .map(|reason| format!("it is locked ({reason})"))
}

/// Why removing it is never automatic, or none.
fn unsafe_why(fact: &Fact) -> Option<String> {
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
    /// The thread told about a worktree: the owning agent's when it may remove it, else the rule's, else the lead.
    pub(crate) fn told_thread(
        board: &BoardFacts,
        fact: &Fact,
        agent: Option<&AgentFact>,
    ) -> Option<String> {
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

    /// The agents that own the worktree, and the one the verdict names (the last to end, a running one after all).
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

    /// A running agent owns it, or an agent that ended after the last scan has not been looked at since. `None` when
    /// no agent holds it.
    fn held_by_agent(
        &self,
        look: &Look<'_>,
        project: &str,
        owners: &[&AgentFact],
    ) -> Option<Verdict> {
        if let Some(running) = owners.iter().find(|a| a.live()) {
            return Some(Verdict::InUse(format!(
                "its agent {} is {}",
                running.id, running.status
            )));
        }
        let ended = owners.iter().filter_map(|a| a.ended_ms).max()?;
        if ended > look.scanned_ms {
            self.request_rescan(project);
            return Some(Verdict::Review(
                "its agent ended after the last scan: Argus looks at it again first".to_string(),
            ));
        }
        None
    }

    /// What holds it: an open thing that names it, a run in it, a process that uses it.
    fn linked(&self, look: &Look<'_>, fact: &Fact) -> Linked {
        let runs = match look.runs {
            Ok(runs) => runs,
            Err(err) => return Linked::Unknown(format!("the runs could not be read ({err})")),
        };
        let ci = self.ci();
        if let Some(link) = look
            .board
            .links
            .iter()
            .chain(runs)
            .find(|l| links_to(l, fact, ci))
        {
            return Linked::Uses(format!("{} uses it", link.what));
        }
        match self.busy_by(&fact.path, !look.fresh, "removed") {
            Err(err) => Linked::Unknown(err.message),
            Ok(Some(user)) => {
                Linked::Uses(format!("{} {} uses it", base_name(&user.name), user.pid))
            }
            Ok(None) => Linked::Free,
        }
    }

    /// Condition (c) and the never-automatic cases. For a worktree whose item closed the work is backed up first, so
    /// only what the backup cannot undo stops it: a main checkout, a worktree its repository does not list, a rule
    /// that does not cover it, a lock that is not Brock's.
    fn safety_why(&self, board: &BoardFacts, fact: &Fact, closing: bool) -> Option<String> {
        if let Err(err) = self.assert_not_main(fact) {
            return Some(err.message);
        }
        if !fact.registered {
            return Some("it is not a linked worktree its repository lists".to_string());
        }
        if let Err(err) = self.rule_of(board, fact) {
            return Some(err.message);
        }
        if closing {
            lock_why(fact)
        } else {
            unsafe_why(fact)
        }
    }

    /// What a thread's own release is checked against: the safety, then what holds it.
    pub(crate) fn blocked_why(&self, look: &Look<'_>, fact: &Fact) -> Option<String> {
        if let Some(why) = self.safety_why(look.board, fact, false) {
            return Some(why);
        }
        match self.linked(look, fact) {
            Linked::Free => None,
            Linked::Uses(why) | Linked::Unknown(why) => Some(why),
        }
    }

    /// The item of the worktree when that item is closed and the worktree closes with it.
    fn closed_item(&self, look: &Look<'_>, fact: &Fact, owners: &[&AgentFact]) -> Option<String> {
        if !self.opts.close_with_item {
            return None;
        }
        let link = resolve(look.board, fact, agent_item(owners), self.ci())?;
        look.board
            .items
            .get(&link.item)
            .filter(|item| item.closed())
            .map(|item| item.id.clone())
    }

    /// The verdict on one worktree and whom it is about. Whatever its age: a worktree no running agent owns, nothing
    /// holds, that is clean and whose HEAD is kept is released; one whose item closed goes the same way with a backup.
    pub(crate) fn judge(&self, project: &str, look: &Look<'_>, fact: &Fact) -> (Verdict, Owner) {
        let Some(shape) = shape_of(fact) else {
            return (Verdict::Quiet, Owner::default());
        };
        let owners = self.owners(look.board, fact);
        let last = owners.last().copied();
        let closed = (shape == Shape::Worktree)
            .then(|| self.closed_item(look, fact, &owners))
            .flatten();
        let owner = Owner {
            agent: last.map(|a| a.id.clone()),
            thread: Self::told_thread(look.board, fact, last),
            item: closed.clone(),
        };
        let verdict = self
            .held_by_agent(look, project, &owners)
            .unwrap_or_else(|| self.verdict_of(look, fact, shape, closed.as_deref()));
        (verdict, owner)
    }

    /// What stops the worktree before anything else is asked: what Argus could not remove, the thread's keep (not
    /// for a closed item: a keep does not hold it, link it to another open item instead), the safety.
    fn stopped_by(
        &self,
        look: &Look<'_>,
        fact: &Fact,
        shape: Shape,
        closed: Option<&str>,
    ) -> Option<Verdict> {
        if let Some(refused) = look
            .refused
            .filter(|r| r.head == fact.head && r.dirty == fact.dirty)
        {
            let why = format!("Argus could not remove it: {}", refused.why);
            return Some(Verdict::Review(because(closed, why)));
        }
        let keep = look
            .board
            .keeps
            .get(&fact.id)
            .filter(|k| k.until_ms > self.now() && closed.is_none());
        if let Some(keep) = keep {
            return Some(Verdict::Kept(keep.note.clone()));
        }
        if shape != Shape::Worktree {
            return None;
        }
        self.safety_why(look.board, fact, closed.is_some())
            .map(|why| Verdict::Review(because(closed, why)))
    }

    fn verdict_of(
        &self,
        look: &Look<'_>,
        fact: &Fact,
        shape: Shape,
        closed: Option<&str>,
    ) -> Verdict {
        if let Some(stopped) = self.stopped_by(look, fact, shape, closed) {
            return stopped;
        }
        match self.linked(look, fact) {
            Linked::Free if closed.is_some() => Verdict::Closing,
            Linked::Free => Verdict::Released,
            Linked::Uses(why) => Verdict::InUse(why),
            Linked::Unknown(why) => Verdict::Review(because(closed, why)),
        }
    }
}

/// `its item I-4 closed, but <why>` for a worktree of a closed item, else the reason as it is.
fn because(closed: Option<&str>, why: String) -> String {
    match closed {
        Some(item) => format!("its item {item} closed, but {why}"),
        None => why,
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
