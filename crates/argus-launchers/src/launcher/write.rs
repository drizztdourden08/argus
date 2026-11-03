//! Making and changing a launcher (`coord launcher set`).

use super::audit::audit;
use super::authorize::authorize_launcher;
use super::join::{is_master, joined_bundle};
use super::master_role;
use super::need_link;
use super::play::sync_launcher_play;
use super::record::{Launcher, LauncherBoard, RuleError, Who};

/// The fields a write changes, already validated by the caller: `None` leaves a field
/// alone, `Some(None)` clears a text field, `Some(Some(v))` sets it. `use_every` is a
/// duration text `parse_every` accepted. `for_item` is an item id the caller checked (or `None`
/// inside, to clear the link of the master build).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LauncherPatch {
    pub title: Option<Option<String>>,
    pub cmd: Option<Option<String>>,
    pub cwd: Option<Option<String>>,
    pub kind: Option<String>,
    pub branch: Option<Option<String>>,
    pub commit: Option<Option<String>>,
    pub note: Option<Option<String>>,
    pub current: Option<bool>,
    pub window: Option<bool>,
    pub use_every: Option<Option<String>>,
    pub for_item: Option<Option<String>>,
    /// `Some(true)` hands the master role to this build; a write never takes the role away.
    pub master: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteResult {
    pub created: bool,
    pub revived: bool,
    pub changed: Vec<&'static str>,
    pub play: String,
    pub ignored: Vec<&'static str>,
    /// The build that held the master role before this write handed it over.
    pub previous_master: Option<String>,
}

/// Where and when a write happens.
#[derive(Clone, Copy, Debug)]
pub struct WriteContext<'a> {
    /// Where the CLI runs: the last resort for the project root of the Play definition.
    pub cwd: &'a str,
    /// The bundle a new launcher is made for (`coord build --cmd`).
    pub bundle: Option<&'a str>,
    /// The time every field and the audit line of this write carry.
    pub at: &'a str,
}

fn differs<T: PartialEq>(current: &T, patch: Option<&T>) -> bool {
    patch.is_some_and(|wanted| wanted != current)
}

fn set<T: Clone>(slot: &mut T, patch: Option<&T>) {
    if let Some(value) = patch {
        slot.clone_from(value);
    }
}

fn changed_fields(rec: &Launcher, patch: &LauncherPatch) -> Vec<&'static str> {
    [
        ("title", differs(&rec.title, patch.title.as_ref())),
        ("cmd", differs(&rec.cmd, patch.cmd.as_ref())),
        ("cwd", differs(&rec.cwd, patch.cwd.as_ref())),
        ("kind", differs(&rec.kind, patch.kind.as_ref())),
        ("branch", differs(&rec.branch, patch.branch.as_ref())),
        ("commit", differs(&rec.commit, patch.commit.as_ref())),
        ("note", differs(&rec.note, patch.note.as_ref())),
        ("current", differs(&rec.current, patch.current.as_ref())),
        ("window", differs(&rec.window, patch.window.as_ref())),
        (
            "useEvery",
            differs(&rec.use_every, patch.use_every.as_ref()),
        ),
        ("for", differs(&rec.for_item, patch.for_item.as_ref())),
        ("master", master_role::wanted(patch) && !rec.link.master),
    ]
    .into_iter()
    .filter_map(|(name, changed)| changed.then_some(name))
    .collect()
}

fn apply_patch(rec: &mut Launcher, patch: &LauncherPatch) {
    set(&mut rec.title, patch.title.as_ref());
    set(&mut rec.cmd, patch.cmd.as_ref());
    set(&mut rec.cwd, patch.cwd.as_ref());
    set(&mut rec.kind, patch.kind.as_ref());
    set(&mut rec.branch, patch.branch.as_ref());
    set(&mut rec.commit, patch.commit.as_ref());
    set(&mut rec.note, patch.note.as_ref());
    set(&mut rec.current, patch.current.as_ref());
    set(&mut rec.window, patch.window.as_ref());
    set(&mut rec.use_every, patch.use_every.as_ref());
    set(&mut rec.for_item, patch.for_item.as_ref());
    if master_role::wanted(patch) {
        rec.link.master = true;
    }
}

fn needs_every(name: &str) -> RuleError {
    RuleError::new(format!(
        "launcher {name} is a standalone build (not master, not made for a bundle), so it needs the window you expect it to be used in: add --use-every <window> (12h, 3d, 2w or 1mo, from 1h to 365d), for example coord launcher set {name} --use-every 7d. When it goes unused for longer than that it is reported to you: keep it, retire it or change the window. master and a launcher made for a bundle do not take it."
    ))
}

fn new_launcher(name: &str, who: &Who, at: &str, bundle: Option<&str>) -> Launcher {
    Launcher {
        name: name.to_owned(),
        created_at: at.to_owned(),
        created_by: Some(who.actor.clone()),
        updated_at: at.to_owned(),
        by: Some(who.actor.clone()),
        bundle: bundle.map(str::to_owned),
        ..Launcher::default()
    }
}

/// Refusals that need nothing but the patch and the record: a new launcher needs its
/// command and an existing one never loses it.
fn check_cmd(name: &str, have: Option<&Launcher>, patch: &LauncherPatch) -> Result<(), RuleError> {
    let cmd = patch.cmd.as_ref().map(|c| c.as_deref().unwrap_or_default());
    if have.is_none() && cmd.is_none_or(str::is_empty) {
        return Err(RuleError::new(format!(
            "launcher {name} is new: give it the command that starts it, --cmd \"<command>\"."
        )));
    }
    if have.is_some() && cmd == Some("") {
        return Err(RuleError::new(format!(
            "a launcher always has a command: give --cmd a value, or retire it with coord launcher retire {name}."
        )));
    }
    Ok(())
}

/// A standalone launcher needs its window, given now or already stored.
fn check_every(
    name: &str,
    have: Option<&Launcher>,
    patch: &LauncherPatch,
    standalone: bool,
) -> Result<(), RuleError> {
    let every = patch.use_every.as_ref().is_some_and(Option::is_some);
    let stored = have.is_some_and(|h| h.use_every.is_some());
    if standalone && !every && !stored {
        return Err(needs_every(name));
    }
    Ok(())
}

/// Clears the retirement of a launcher; true when it was retired.
fn revive(rec: &mut Launcher) -> bool {
    let revived = rec.retired_at.is_some();
    if revived {
        rec.retired_at = None;
        rec.retired_by = None;
        rec.retire_note = None;
        rec.retired_by_bundle = None;
        rec.retired_by_item = None;
    }
    revived
}

fn restore(board: &mut LauncherBoard, name: &str, previous: Option<Launcher>) {
    match previous {
        Some(rec) => board.launchers.insert(name.to_owned(), rec),
        None => board.launchers.remove(name),
    };
}

fn audit_text(name: &str, created: bool, revived: bool, changed: &[&str]) -> String {
    let verb = match (created, revived) {
        (true, _) => "added",
        (false, true) => "brought back",
        (false, false) => "updated",
    };
    let fields = if changed.is_empty() {
        String::new()
    } else {
        format!(" ({})", changed.join(", "))
    };
    format!("{verb} launcher {name}{fields}")
}

/// Whether the launcher has a usage window after the write: it is not the master build, does not
/// serve an item and is not joined to a bundle.
fn has_window(
    board: &LauncherBoard,
    name: &str,
    have: Option<&Launcher>,
    (patch, ctx): (&LauncherPatch, &WriteContext<'_>),
) -> bool {
    let joined = ctx
        .bundle
        .map(str::to_owned)
        .or_else(|| have.and_then(|h| joined_bundle(board, h)));
    let serves = match patch.for_item.as_ref() {
        Some(wanted) => wanted.is_some(),
        None => have.is_some_and(|h| h.for_item.is_some()),
    };
    !(is_master(board, name) || master_role::wanted(patch)) && joined.is_none() && !serves
}

/// Creates or updates one launcher. A retired one is brought back. A standalone launcher
/// (not master, not joined to a bundle, not serving an item) must have `useEvery`: given now,
/// or already stored; one that is master, joined or serving an item takes none and the value
/// is dropped (`ignored`).
/// Every build except the master serves an item: a new one without `for`, or a write that
/// clears the link, is refused.
/// A refusal leaves the board as it was.
pub fn write_launcher(
    board: &mut LauncherBoard,
    who: &Who,
    name: &str,
    patch: &LauncherPatch,
    ctx: &WriteContext<'_>,
) -> Result<WriteResult, RuleError> {
    let have = board.launchers.get(name).cloned();
    authorize_launcher(board, who, have.as_ref())?;
    master_role::check_hand(who, patch)?;
    let standalone = has_window(board, name, have.as_ref(), (patch, ctx));
    let mut patch = patch.clone();
    let mut ignored = Vec::new();
    if patch.use_every.is_some() && !standalone {
        patch.use_every = None;
        ignored.push("useEvery");
    }
    check_cmd(name, have.as_ref(), &patch)?;
    need_link::check(board, name, have.as_ref(), &patch)?;
    check_every(name, have.as_ref(), &patch, standalone)?;
    let created = have.is_none();
    let mut rec = have
        .clone()
        .unwrap_or_else(|| new_launcher(name, who, ctx.at, ctx.bundle));
    let changed = if created {
        Vec::new()
    } else {
        changed_fields(&rec, &patch)
    };
    apply_patch(&mut rec, &patch);
    let revived = revive(&mut rec);
    let touched = created || revived || !changed.is_empty();
    if touched {
        ctx.at.clone_into(&mut rec.updated_at);
        rec.by = Some(who.actor.clone());
    }
    board.launchers.insert(name.to_owned(), rec);
    let play = match sync_launcher_play(board, who, name, Some(ctx.cwd), ctx.at) {
        Ok(note) => note,
        Err(err) => {
            restore(board, name, have);
            return Err(err);
        }
    };
    if touched {
        audit(
            board,
            who,
            &audit_text(name, created, revived, &changed),
            ctx.at,
        );
    }
    let previous_master = master_role::pass_on(board, &patch, (name, who, ctx.at));
    Ok(WriteResult {
        created,
        revived,
        changed,
        play,
        ignored,
        previous_master,
    })
}
