//! The router (`coord.mjs`): every verb and alias, the project lookup, the error to exit code
//! map, and the exit 120 of a verb that is not ported yet.
//!
//! A verb that is not in the ported table returns [`NOT_PORTED`] at once: before the project
//! lookup, before any read of stdin and before any file is touched, so the shim can run the Node
//! verb with nothing done twice. Help and an unknown verb are answered here and touch nothing.

use std::panic::{AssertUnwindSafe, catch_unwind};

use super::args::{Options, parse_args};
use super::exit::{CoordError, NOT_PORTED, Outcome, report};
use super::invocation::Invocation;
use super::usage::USAGE;
use crate::who::find_project;

/// The state a verb runs with.
pub struct Call<'a> {
    /// The streams, environment, working directory and clock.
    pub inv: &'a mut Invocation,
    /// The words after the verb. `entry` and `purge` keep the typed verb as the first word.
    pub positional: Vec<String>,
    /// The options.
    pub options: Options,
    /// The project, resolved for the verbs that need one (every verb but `hook`, `portfolio`,
    /// `dashboard`, `lead`, `wake-fault`, `proc` and `activity`).
    pub project: Option<String>,
}

/// A verb group that is served here.
pub type Handler = fn(&mut Call<'_>) -> Outcome;

/// A ported verb group: its module name (the value of the `VERBS` table of `coord.mjs`) and its
/// handler.
pub type Entry = (&'static str, Handler);

/// Every verb and alias, with the module that serves it.
const VERBS: [(&str, &str); 40] = [
    ("hook", "hook"),
    ("dashboard", "dashboard"),
    ("thread", "thread"),
    ("agent", "agent"),
    ("item", "item"),
    ("test", "test"),
    ("build", "build"),
    ("launcher", "launcher"),
    ("ask", "entry"),
    ("decide", "entry"),
    ("notify", "entry"),
    ("message", "entry"),
    ("note", "entry"),
    ("answer", "entry"),
    ("entry", "entry"),
    ("lock", "lock"),
    ("action", "action"),
    ("todo", "action"),
    ("log", "log"),
    ("inbox", "inbox"),
    ("board", "board"),
    ("sync", "sync"),
    ("check", "check"),
    ("tick", "tick"),
    ("dispatch", "dispatch"),
    ("purge", "purge"),
    ("restore", "purge"),
    ("trash", "purge"),
    ("portfolio", "portfolio"),
    ("lead", "lead"),
    ("ping", "ping"),
    ("wake-fault", "wakefault"),
    ("stall", "stall"),
    ("proc", "proc"),
    ("activity", "activity"),
    ("run", "run"),
    ("worktree", "worktree"),
    ("remote", "remote"),
    ("pr", "pr"),
    ("dep", "dep"),
];

/// These modules work outside any project and find their own caller when they need one.
const NO_PROJECT: [&str; 9] = [
    "hook",
    "portfolio",
    "dashboard",
    "lead",
    "wakefault",
    "proc",
    "activity",
    "remote",
    "stall",
];

/// These modules serve several verbs and are told which one was typed.
const TAKES_VERB: [&str; 2] = ["entry", "purge"];

/// The module that serves a verb or alias.
#[must_use]
pub fn module_of(verb: &str) -> Option<&'static str> {
    VERBS.iter().find(|(v, _)| *v == verb).map(|(_, m)| *m)
}

fn help(inv: &mut Invocation) -> i32 {
    inv.say(&[USAGE]);
    0
}

fn unknown(inv: &mut Invocation, verb: &str) -> i32 {
    inv.warn(&format!("Unknown verb \"{verb}\".\n\n{USAGE}"));
    1
}

/// A hook must never break the session it runs in: any error or panic is exit 0 and no output.
fn run_hook(handler: Handler, call: &mut Call<'_>) -> i32 {
    match catch_unwind(AssertUnwindSafe(|| handler(call))) {
        Ok(Err(CoordError::Exit(code))) => code,
        _ => 0,
    }
}

fn run_verb(handler: Handler, call: &mut Call<'_>) -> i32 {
    match handler(call) {
        Ok(()) => 0,
        Err(e) => report(&e, &mut call.inv.stderr),
    }
}

fn with_project(inv: &mut Invocation, options: &Options) -> Result<String, i32> {
    let sid = inv.env.session_id().map(str::to_owned);
    find_project(&inv.env, &inv.cwd, options, sid.as_deref())
        .map_err(|e| report(&e, &mut inv.stderr))
}

/// Runs one call of `coord`: the verbs in `ported` are served, every other verb answers
/// [`NOT_PORTED`]. Returns the exit code of the process.
pub fn dispatch(inv: &mut Invocation, ported: &[Entry]) -> i32 {
    let args = parse_args(&inv.argv);
    let Some((verb, rest)) = args.positional.split_first() else {
        return help(inv);
    };
    if verb == "help" {
        return help(inv);
    }
    let Some(module) = module_of(verb) else {
        return unknown(inv, verb);
    };
    let Some((_, handler)) = ported.iter().find(|(m, _)| *m == module) else {
        return NOT_PORTED;
    };
    let mut positional = rest.to_vec();
    if TAKES_VERB.contains(&module) {
        positional.insert(0, verb.clone());
    }
    let project = if NO_PROJECT.contains(&module) {
        None
    } else {
        match with_project(inv, &args.options) {
            Ok(project) => Some(project),
            Err(code) => return code,
        }
    };
    let mut call = Call {
        inv,
        positional,
        options: args.options,
        project,
    };
    if module == "hook" {
        run_hook(*handler, &mut call)
    } else {
        run_verb(*handler, &mut call)
    }
}
