//! Where Claude keeps the files of one thread's session:
//! `<claudeHome>/projects/<slug>/<sessionId>/subagents/agent-<id>.jsonl` (and `.meta.json`), and
//! `<claudeTemp>/<slug>/<sessionId>/tasks/<id>.output`. A sub-agent lives under the session that
//! launched it, so a lead that was resumed has older agents in older session folders.

use super::Env;
use crate::board::Board;
use crate::fs::FileSystem;
use crate::record::{Record, filled, text};
use serde_json::Value;
use std::cell::OnceCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const EARLIER_SESSIONS: usize = 4;

/// A sub-agent transcript found on disk, with its meta.
#[derive(Debug)]
pub(super) struct Candidate {
    pub id: String,
    pub file: PathBuf,
    pub birth_ms: i64,
    pub meta: Option<Record>,
}

struct Layout {
    dir: PathBuf,
    own: PathBuf,
    other_sessions: HashSet<String>,
    task_dir: PathBuf,
}

/// What is known about the session of a thread.
pub(super) struct Session<'a> {
    fs: &'a dyn FileSystem,
    sid: Option<String>,
    layout: Option<Layout>,
    earlier: OnceCell<Vec<PathBuf>>,
    /// Whether Claude has a folder or a transcript for the session.
    pub known: bool,
}

fn word_chars(text: &str, extra: &[char]) -> bool {
    text.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || extra.contains(&c))
}

fn valid_session(sid: &str) -> bool {
    (8..=64).contains(&sid.len()) && word_chars(sid, &['-'])
}

fn valid_agent_id(id: &str) -> bool {
    !id.is_empty() && word_chars(id, &['.', '-'])
}

fn agent_stem(name: &str) -> Option<&str> {
    name.strip_prefix("agent-")?.strip_suffix(".jsonl")
}

/// The agent id in a transcript file name, `agent-<id>.jsonl`.
pub(super) fn agent_id_in(name: &str) -> Option<String> {
    let id = agent_stem(name)?;
    valid_agent_id(id).then(|| id.to_owned())
}

/// The agent id in the last part of a path.
pub(super) fn agent_id_of_path(path: &str) -> Option<String> {
    agent_id_in(path.rsplit(['\\', '/']).next()?)
}

/// The meta file beside a transcript.
pub(super) fn meta_path(file: &Path) -> PathBuf {
    let name = file.to_string_lossy();
    let stem = name.strip_suffix(".jsonl").unwrap_or(&name);
    PathBuf::from(format!("{stem}.meta.json"))
}

/// A JSON object read from a file, `None` when it is missing, broken or not an object.
pub(super) fn read_object(fs: &dyn FileSystem, file: &Path) -> Option<Record> {
    match serde_json::from_str::<Value>(&fs.read_text(file)?).ok()? {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

fn slug_of(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn holds_session(fs: &dyn FileSystem, dir: &Path, sid: &str) -> bool {
    fs.stat(&dir.join(sid)).is_some() || fs.stat(&dir.join(format!("{sid}.jsonl"))).is_some()
}

/// The project folder of a sub-agent transcript path that sits in session `sid`.
fn folder_of_transcript(file: &str, sid: &str) -> Option<String> {
    let parts: Vec<&str> = file.split(['\\', '/']).collect();
    let [.., session, subagents, name] = parts.as_slice() else {
        return None;
    };
    let named = agent_stem(name).is_some_and(|stem| !stem.is_empty());
    if *session != sid || *subagents != "subagents" || !named {
        return None;
    }
    let tail = session.len() + subagents.len() + name.len() + 3;
    file.len()
        .checked_sub(tail)
        .map(|end| file[..end].to_owned())
}

fn guesses(env: &Env, board: &Board, thread: &Record, sid: &str) -> Vec<PathBuf> {
    let thread_id = text(thread, "id");
    let mut out: Vec<PathBuf> = board
        .agents
        .iter()
        .filter(|a| text(a, "thread") == thread_id)
        .filter_map(|a| filled(a, "transcriptFile"))
        .filter_map(|file| folder_of_transcript(file, sid))
        .map(PathBuf::from)
        .collect();
    if let Some(root) = filled(thread, "root") {
        out.push(env.projects_dir().join(slug_of(root)));
    }
    out
}

fn find_project_dir(env: &Env, board: &Board, thread: &Record, sid: &str) -> Option<PathBuf> {
    if let Some(known) = env.cached_dir(sid)
        && env.fs.stat(&known.join(sid)).is_some()
    {
        return Some(known);
    }
    let from_guess = guesses(env, board, thread, sid)
        .into_iter()
        .find(|g| holds_session(env.fs, g, sid));
    let found = from_guess.or_else(|| scan_projects(env, sid));
    if let Some(dir) = &found {
        env.remember_dir(sid, dir.clone());
    }
    found
}

fn scan_projects(env: &Env, sid: &str) -> Option<PathBuf> {
    let root = env.projects_dir();
    let entries = env.fs.list(&root).ok()?;
    entries
        .iter()
        .map(|d| root.join(&d.name))
        .find(|p| holds_session(env.fs, p, sid))
}

impl<'a> Session<'a> {
    /// The session of a thread of the board.
    pub(super) fn open(env: &Env<'a>, board: &Board, thread_id: Option<&str>) -> Self {
        let thread = thread_id.and_then(|id| board.thread(id));
        let sid = thread
            .and_then(|t| text(t, "sessionId"))
            .filter(|s| valid_session(s))
            .map(str::to_owned);
        let dir = match (&sid, thread) {
            (Some(sid), Some(thread)) => find_project_dir(env, board, thread, sid),
            _ => None,
        };
        let layout = sid
            .as_deref()
            .zip(dir)
            .map(|(sid, dir)| Layout::new(env, board, sid, dir));
        let known = match (&layout, sid.as_deref()) {
            (Some(l), Some(sid)) => holds_session(env.fs, &l.dir, sid),
            _ => false,
        };
        Self {
            fs: env.fs,
            sid,
            layout,
            earlier: OnceCell::new(),
            known,
        }
    }

    /// Whether the thread has a session id at all.
    pub(super) fn has_session_id(&self) -> bool {
        self.sid.is_some()
    }

    fn earlier_dirs(&self, layout: &Layout) -> &Vec<PathBuf> {
        self.earlier.get_or_init(|| {
            let sid = self.sid.as_deref().unwrap_or("");
            let Ok(entries) = self.fs.list(&layout.dir) else {
                return Vec::new();
            };
            let mut found: Vec<(PathBuf, i64)> = entries
                .iter()
                .filter(|d| d.is_dir && d.name != sid && !layout.other_sessions.contains(&d.name))
                .map(|d| layout.dir.join(&d.name).join("subagents"))
                .filter_map(|p| self.fs.stat(&p).map(|s| (p, s.mtime_ms)))
                .filter(|(_, m)| *m > 0)
                .collect();
            found.sort_by_key(|entry| std::cmp::Reverse(entry.1));
            found.truncate(EARLIER_SESSIONS);
            found.into_iter().map(|(p, _)| p).collect()
        })
    }

    fn dirs(&self) -> Vec<PathBuf> {
        let Some(layout) = &self.layout else {
            return Vec::new();
        };
        let mut dirs = vec![layout.own.join("subagents")];
        dirs.extend(self.earlier_dirs(layout).iter().cloned());
        dirs
    }

    /// The transcript of a sub-agent in this session or its earlier ones.
    pub(super) fn find_file(&self, id: &str) -> Option<PathBuf> {
        if !valid_agent_id(id) {
            return None;
        }
        self.dirs()
            .into_iter()
            .map(|d| d.join(format!("agent-{id}.jsonl")))
            .find(|f| self.fs.stat(f).is_some())
    }

    /// The output file of a background task of this session.
    pub(super) fn task_file(&self, id: &str) -> Option<PathBuf> {
        let layout = self.layout.as_ref()?;
        if !valid_agent_id(id) {
            return None;
        }
        let file = layout.task_dir.join(format!("{id}.output"));
        self.fs.stat(&file).map(|_| file)
    }

    /// Every transcript of the session's folders, with its meta, to match a record that has no id.
    pub(super) fn candidates(&self) -> Vec<Candidate> {
        let mut out = Vec::new();
        for dir in self.dirs() {
            let Ok(names) = self.fs.list(&dir) else {
                continue;
            };
            for entry in names {
                let Some(id) = agent_id_in(&entry.name) else {
                    continue;
                };
                let file = dir.join(&entry.name);
                let Some(stat) = self.fs.stat(&file) else {
                    continue;
                };
                let meta = read_object(self.fs, &meta_path(&file));
                out.push(Candidate {
                    id,
                    file,
                    birth_ms: stat.birth_ms,
                    meta,
                });
            }
        }
        out
    }
}

impl Layout {
    fn new(env: &Env, board: &Board, sid: &str, dir: PathBuf) -> Self {
        let own = dir.join(sid);
        let other_sessions = board
            .threads
            .iter()
            .filter_map(|t| filled(t, "sessionId"))
            .filter(|s| *s != sid)
            .map(str::to_owned)
            .collect();
        let folder = dir.file_name().map(PathBuf::from).unwrap_or_default();
        let task_dir = env.opts.claude_temp.join(folder).join(sid).join("tasks");
        Self {
            dir,
            own,
            other_sessions,
            task_dir,
        }
    }
}
