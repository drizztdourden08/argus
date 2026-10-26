//! What the flags of `build add|set` ask for, nothing else: the bundle's own fields (`patch`:
//! `--title`, `--note`, `--thread`), its launcher's (`launch`: `--cmd`, `--cwd`, `--window`) and
//! the launcher to link (`--launcher`). An empty value clears a field.

use argus_board::Board;
use argus_board::json::doc;
use argus_core::js;
use argus_launchers::launcher::{LauncherPatch, require_launcher_name};
use serde_json::Value;

use crate::cli::Options;
use crate::cli::exit::CoordError;
use crate::verbs::agent::base::opt::{Text, is_list, js_string, not_one_text, one_text};
use crate::verbs::agent::rules::refused;

/// The bundle's own fields.
#[derive(Default)]
pub struct Patch {
    /// `--title`.
    pub title: Text,
    /// `--note`.
    pub note: Text,
    /// `--thread`.
    pub thread: Text,
}

impl Patch {
    /// The fields given, in the order they are written.
    #[must_use]
    pub fn entries(&self) -> Vec<(&'static str, &Option<String>)> {
        [
            ("title", &self.title),
            ("note", &self.note),
            ("thread", &self.thread),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.as_ref().map(|v| (key, v)))
        .collect()
    }
}

/// The launcher's fields.
#[derive(Default)]
pub struct Launch {
    /// `--cmd`.
    pub cmd: Text,
    /// `--cwd`.
    pub cwd: Text,
    /// `--window` or `--no-window`.
    pub window: Option<bool>,
}

impl Launch {
    /// The names of the fields given, in order.
    #[must_use]
    pub fn keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        if self.cmd.is_some() {
            keys.push("cmd");
        }
        if self.cwd.is_some() {
            keys.push("cwd");
        }
        if self.window.is_some() {
            keys.push("window");
        }
        keys
    }

    /// The same fields as a launcher write.
    #[must_use]
    pub fn patch(&self) -> LauncherPatch {
        LauncherPatch {
            cmd: self.cmd.clone(),
            cwd: self.cwd.clone(),
            window: self.window,
            ..LauncherPatch::default()
        }
    }
}

/// Everything the flags ask for.
#[derive(Default)]
pub struct Parts {
    /// The bundle's own fields.
    pub patch: Patch,
    /// The launcher's fields.
    pub launch: Launch,
    /// The launcher to link: `Some(None)` unlinks.
    pub link: Text,
}

impl Parts {
    /// Whether any flag was given.
    #[must_use]
    pub fn any(&self) -> bool {
        !self.patch.entries().is_empty() || !self.launch.keys().is_empty() || self.link.is_some()
    }
}

fn text_flag(options: &Options, name: &str, kind: &str) -> Result<Text, CoordError> {
    one_text(options, name, kind, true)
}

fn thread_flag(board: &Board, options: &Options) -> Result<Text, CoordError> {
    if options.get("thread").is_none() {
        return Ok(None);
    }
    if not_one_text(options, "thread") {
        return Err(CoordError::refuse(
            "--thread takes a thread name (\"\" clears it).",
        ));
    }
    let id = js::trim(&js_string(options, "thread")).to_owned();
    if !id.is_empty() && doc(board, "threads", &id).is_none() {
        return Err(CoordError::refuse(format!("no thread \"{id}\".")));
    }
    Ok(Some((!id.is_empty()).then_some(id)))
}

fn window_flag(options: &Options) -> Result<Option<bool>, CoordError> {
    if options.get("window").is_none() && options.get("no-window").is_none() {
        return Ok(None);
    }
    if is_list(options, "window") {
        return Err(CoordError::refuse(
            "--window takes no value (--no-window clears it).",
        ));
    }
    let word = js_string(options, "window").to_lowercase();
    let off =
        options.get("no-window").is_some() || ["false", "0", "no", "off"].contains(&word.as_str());
    Ok(Some(!off))
}

fn link_flag(options: &Options, launch: &Launch) -> Result<Text, CoordError> {
    if options.get("launcher").is_none() {
        return Ok(None);
    }
    if not_one_text(options, "launcher") {
        return Err(CoordError::refuse(
            "--launcher takes a launcher name (\"\" unlinks it).",
        ));
    }
    let named = js_string(options, "launcher");
    let link = if js::trim(&named).is_empty() {
        None
    } else {
        Some(require_launcher_name(&named).map_err(refused)?)
    };
    if !launch.keys().is_empty() {
        return Err(CoordError::refuse(
            "pass --launcher <name> or --cmd (with --cwd, --window), not both: change the command, folder or window of a launcher with coord launcher set <name>.",
        ));
    }
    Ok(Some(link))
}

/// `partsFrom`.
///
/// # Errors
/// A flag with no value or too many, a text over its limit, an unknown thread, a bad launcher
/// name, both a launcher and a command.
pub fn parts_from(board: &Board, options: &Options) -> Result<Parts, CoordError> {
    let patch = Patch {
        title: text_flag(options, "title", "title")?,
        note: text_flag(options, "note", "buildnote")?,
        thread: thread_flag(board, options)?,
    };
    let launch = Launch {
        cmd: text_flag(options, "cmd", "buildcmd")?,
        cwd: text_flag(options, "cwd", "buildcwd")?,
        window: window_flag(options)?,
    };
    let link = link_flag(options, &launch)?;
    Ok(Parts {
        patch,
        launch,
        link,
    })
}

/// A bundle field as the JSON it is stored as.
#[must_use]
pub fn json_of(text: Option<&String>) -> Value {
    text.map_or(Value::Null, |t| Value::from(t.as_str()))
}
