//! Runs the demo steps through the real `coord` verbs, in this process, against the preview
//! home. A step that fails is recorded and the run goes on, so one refusal never leaves the
//! board half empty; a step whose `{name}` was never bound is skipped.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use argus_board::demo::Step;
use argus_board::deps::{BoardRoster, Roster};

use crate::cli::{Env, Invocation};

/// The state of one seed.
pub(super) struct Runner {
    home: PathBuf,
    key: String,
    cwd: PathBuf,
    binds: HashMap<String, String>,
    /// What went wrong, one line each.
    pub(super) failed: Vec<String>,
    /// How many calls passed.
    pub(super) passed: usize,
}

/// The first id in a text, such as `I-12`, `T-3`, `A-1` or `PR-2`.
fn id_in(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-'))
        .find(|t| is_id(t))
        .map(str::to_owned)
}

fn is_id(token: &str) -> bool {
    let Some((letters, digits)) = token.split_once('-') else {
        return false;
    };
    let lettered =
        (1..=3).contains(&letters.len()) && letters.bytes().all(|b| b.is_ascii_uppercase());
    lettered && !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

impl Runner {
    pub(super) fn new(home: &Path, key: &str, cwd: &Path) -> Self {
        Runner {
            home: home.to_path_buf(),
            key: key.to_owned(),
            cwd: cwd.to_path_buf(),
            binds: HashMap::new(),
            failed: Vec::new(),
            passed: 0,
        }
    }

    /// Names a value for the `{name}` placeholders of the steps.
    pub(super) fn bind(&mut self, name: &str, value: &str) {
        self.binds.insert(name.to_owned(), value.to_owned());
    }

    /// The value bound to a name.
    pub(super) fn bound(&self, name: &str) -> Option<&str> {
        self.binds.get(name).map(String::as_str)
    }

    /// The home the steps write into.
    pub(super) fn home(&self) -> &Path {
        &self.home
    }

    /// The key of the demo board.
    pub(super) fn key(&self) -> &str {
        &self.key
    }

    fn fill(&self, word: &str) -> Option<String> {
        let mut text = word.to_owned();
        for (name, value) in &self.binds {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        (!(text.contains('{') && text.contains('}'))).then_some(text)
    }

    /// The thread to act as on a board: `lead` is whoever leads a board of the home that is not
    /// the demo board.
    fn thread_for(&self, project: &str, who: &str) -> String {
        if who != argus_board::demo::LEAD || project == self.key {
            return who.to_owned();
        }
        BoardRoster::new(&self.home)
            .lead(project)
            .unwrap_or_else(|| who.to_owned())
    }

    /// One call of `coord` with these words on a board, as a thread when `who` is not empty.
    pub(super) fn call(
        &self,
        project: &str,
        who: &str,
        words: &[String],
    ) -> Result<String, String> {
        let mut argv: Vec<String> = words.to_vec();
        argv.extend(["--project".to_owned(), project.to_owned()]);
        if !who.is_empty() {
            argv.extend(["--as".to_owned(), self.thread_for(project, who)]);
        }
        let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        let home = self.home.to_string_lossy().into_owned();
        let dash = self.home.join("dashboard").to_string_lossy().into_owned();
        let env = Env::of(&[
            ("COORD_HOME", home.as_str()),
            ("DASHBOARD_HOME", dash.as_str()),
            ("DASHBOARD_PORT", "0"),
        ]);
        let (mut inv, out, err) = Invocation::captured(&refs, env, self.cwd.clone(), "");
        let code = crate::run(&mut inv);
        if code == 0 {
            Ok(out.text())
        } else {
            let said = err.text();
            let line = said.lines().next().unwrap_or("no message");
            Err(format!("exit {code}: {line}"))
        }
    }

    /// Runs one step: fills its placeholders, calls, and keeps the id it binds.
    pub(super) fn run(&mut self, step: &Step) {
        let filled: Option<Vec<String>> = step.words.iter().map(|w| self.fill(w)).collect();
        let shown = step
            .words
            .iter()
            .take(3)
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
        let Some(words) = filled else {
            self.failed
                .push(format!("skipped {shown}: an id it needs was never made"));
            return;
        };
        let project = if step.project.is_empty() {
            self.key.clone()
        } else {
            step.project.to_owned()
        };
        match self.call(&project, step.who, &words) {
            Ok(out) => self.kept(step.bind, &out),
            Err(why) => self.failed.push(format!("{shown}: {why}")),
        }
    }

    fn kept(&mut self, bind: &str, out: &str) {
        self.passed += 1;
        if let Some(id) = (!bind.is_empty()).then(|| id_in(out)).flatten() {
            self.binds.insert(bind.to_owned(), id);
        }
    }

    /// Runs every step in order.
    pub(super) fn run_all(&mut self, steps: &[Step]) {
        for step in steps {
            self.run(step);
        }
    }
}
