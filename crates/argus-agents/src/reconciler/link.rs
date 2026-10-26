//! Step 2: a live record with no id and no transcript is matched to a transcript of its own
//! thread's session by the meta's `description` against the record's name, task, branch and
//! worktree: at least two shared words, a model that agrees when both know it, a transcript
//! created from an hour before to ten minutes after the record, and a clear winner or no link.

use super::pass::Pass;
use super::report::Class;
use super::session::{Candidate, agent_id_of_path};
use crate::config::MINUTE_MS;
use crate::iso::format_iso;
use crate::record::{Patch, Record, claude_id_of, filled, is_set, ms_of, text};
use crate::text::{clip, overlap, same_model};
use std::collections::HashSet;

const MIN_LINK: usize = 2;
const RIVAL_GAP_MS: i64 = 2 * MINUTE_MS;

struct Pair {
    rec: usize,
    cand: usize,
    score: usize,
    gap: i64,
}

fn describe(agent: &Record) -> String {
    let part = |key: &str| text(agent, key).unwrap_or("").to_owned();
    format!(
        "{} {} {} {}",
        part("name"),
        part("task"),
        part("branch"),
        part("worktree")
    )
}

fn meta_text<'a>(candidate: &'a Candidate, key: &str) -> Option<&'a str> {
    candidate.meta.as_ref().and_then(|m| filled(m, key))
}

fn models_differ(agent: &Record, candidate: &Candidate) -> bool {
    match (filled(agent, "model"), meta_text(candidate, "model")) {
        (Some(asked), Some(used)) => !same_model(asked, used),
        _ => false,
    }
}

fn in_window(start: Option<i64>, candidate: &Candidate, before: i64, after: i64) -> bool {
    start.is_none_or(|s| candidate.birth_ms >= s - before && candidate.birth_ms <= s + after)
}

fn rivals(
    pairs: &[Pair],
    at: usize,
    rec_done: &HashSet<usize>,
    cand_done: &HashSet<usize>,
) -> bool {
    let p = &pairs[at];
    pairs.iter().enumerate().any(|(i, q)| {
        let shares_one = (q.rec == p.rec) != (q.cand == p.cand);
        i != at
            && !rec_done.contains(&q.rec)
            && !cand_done.contains(&q.cand)
            && shares_one
            && q.score == p.score
            && (q.gap - p.gap).abs() < RIVAL_GAP_MS
    })
}

impl<'a> Pass<'a> {
    pub(super) fn link(&mut self) {
        let mut claimed = self.claimed_ids();
        for (thread, records) in self.unlinked_by_thread() {
            let session = self.session_of(thread.as_deref());
            if !session.known {
                continue;
            }
            let pool: Vec<Candidate> = session
                .candidates()
                .into_iter()
                .filter(|c| !claimed.contains(&c.id))
                .collect();
            let pairs = self.score_pairs(&records, &pool);
            self.take_pairs(&records, &pool, &pairs, &mut claimed);
        }
    }

    fn claimed_ids(&self) -> HashSet<String> {
        let mut claimed = HashSet::new();
        for agent in self
            .all
            .iter()
            .filter(|a| text(a, "source") != Some("hook") || is_set(a, "leadBound"))
        {
            let current = self.cur(agent);
            let from_file = filled(&current, "transcriptFile").and_then(agent_id_of_path);
            claimed.extend(claude_id_of(&current).into_iter().chain(from_file));
        }
        claimed
    }

    fn unlinked_by_thread(&self) -> Vec<(Option<String>, Vec<&'a Record>)> {
        let mut groups: Vec<(Option<String>, Vec<&'a Record>)> = Vec::new();
        for agent in self.all.iter().copied() {
            let current = self.cur(agent);
            let unlinked =
                claude_id_of(&current).is_none() && !self.exists(text(agent, "transcriptFile"));
            if !(Self::is_open(agent) && unlinked) {
                continue;
            }
            let thread = text(agent, "thread").map(str::to_owned);
            match groups.iter_mut().find(|(t, _)| *t == thread) {
                Some((_, list)) => list.push(agent),
                None => groups.push((thread, vec![agent])),
            }
        }
        groups
    }

    fn score_pairs(&self, records: &[&Record], pool: &[Candidate]) -> Vec<Pair> {
        let opts = &self.env.opts;
        let mut pairs = Vec::new();
        for (rec, agent) in records.iter().enumerate() {
            let start = ms_of(agent, "startedAt");
            let words = describe(agent);
            for (cand, candidate) in pool.iter().enumerate() {
                if !in_window(start, candidate, opts.link_before_ms, opts.link_after_ms)
                    || models_differ(agent, candidate)
                {
                    continue;
                }
                let score = overlap(&words, meta_text(candidate, "description").unwrap_or(""));
                if score >= MIN_LINK {
                    let gap = start.map_or(0, |s| (candidate.birth_ms - s).abs());
                    pairs.push(Pair {
                        rec,
                        cand,
                        score,
                        gap,
                    });
                }
            }
        }
        pairs.sort_by(|x, y| y.score.cmp(&x.score).then(x.gap.cmp(&y.gap)));
        pairs
    }

    fn take_pairs(
        &mut self,
        records: &[&Record],
        pool: &[Candidate],
        pairs: &[Pair],
        claimed: &mut HashSet<String>,
    ) {
        let mut rec_done = HashSet::new();
        let mut cand_done = HashSet::new();
        for (at, pair) in pairs.iter().enumerate() {
            if rec_done.contains(&pair.rec) || cand_done.contains(&pair.cand) {
                continue;
            }
            if rivals(pairs, at, &rec_done, &cand_done) {
                continue;
            }
            rec_done.insert(pair.rec);
            cand_done.insert(pair.cand);
            let candidate = &pool[pair.cand];
            self.link_one(records[pair.rec], candidate);
            claimed.insert(candidate.id.clone());
        }
    }

    fn link_one(&mut self, agent: &Record, candidate: &Candidate) {
        let mut linked = Patch::new()
            .with("claudeId", candidate.id.as_str())
            .with(
                "transcriptFile",
                candidate.file.to_string_lossy().into_owned(),
            )
            .with("linkedAt", format_iso(self.at));
        let taken = self
            .all
            .iter()
            .any(|x| text(x, "hookId") == Some(candidate.id.as_str()));
        if !taken {
            linked = linked.with("hookId", candidate.id.as_str());
        }
        self.patch(agent, linked);
        self.counts.linked += 1;
        let description = meta_text(candidate, "description").unwrap_or("");
        let detail = format!(
            "matched to the transcript of \"{}\" ({})",
            clip(description, 60),
            candidate.id
        );
        self.finding(agent, Class::Linked, detail);
    }
}
