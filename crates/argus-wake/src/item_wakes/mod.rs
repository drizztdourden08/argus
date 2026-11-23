//! A wake about the Kanban: work items that sit on a board for far too long. Nobody should have to look
//! at the board to notice.
//!
//! * `kanban-stale`: a work item has not changed for `stale_doing_ms` (a doing item, default 4 h),
//!   `stale_review_ms` (review, 6 h) or `stale_todo_ms` (todo, 24 h). "Changed" is the newest of its own
//!   `updatedAt` and the records that touch it: its tests, to-dos, sub-agents and entries. The thread
//!   that owns the item (the project's lead when that thread ended or is missing) gets ONE wake per tick
//!   that lists the worst offenders, and the same again every `repeat_ms` (12 h) while any stays that way.
//! * digest: the Overlord hears once a day (`digest_repeat_ms`) about each project that has items quiet for
//!   longer than `escalate_ms` (3 days), so a lead that ignores the reminders is visible.
//!
//! The items and the thresholds are not fixed: the sweep reads the Kanban config of each board
//! (`kanban_sweep`), and a board without one runs the default, which is the reminders above. The same
//! sweep also tells `assignee-gone`, `over-limit` and `unassigned` breaks (docs/kanban-config.md).
//!
//! Never reminded: done, dropped, archived or deleted items; an item that waits on someone else (an open
//! question or decision of the owner, a to-do of the owner, a test run handed to the
//! owner, a live sub-agent working on it, a request to another board that is still being answered, a
//! request of another board that was blocked on the requester); a thread that is paused. There are no
//! quiet hours.

mod kanban_sweep;
mod stale;
mod text;

pub use argus_core::clock::age_text;
use kanban_sweep::Group;
pub use stale::Stale;
pub(crate) use stale::{has_live_agent, last_change, target_for};

use std::collections::HashMap;

use serde_json::Value;

use crate::activity::{ActivityLine, Level};
use crate::config::section_ms;
use crate::reminders::{OwnerNotice, TOPIC_KANBAN};
use crate::source::BoardSource;
use crate::wake::WakeInput;

/// The reason of every reminder.
pub const REASON: &str = "kanban-stale";

/// The thresholds and periods of the Kanban reminders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemWakeConfig {
    /// How often the owner calls `tick`.
    pub tick_ms: i64,
    /// A doing item is stale after this long.
    pub stale_doing_ms: i64,
    /// A review item is stale after this long.
    pub stale_review_ms: i64,
    /// A todo item is stale after this long.
    pub stale_todo_ms: i64,
    /// A thread is reminded again after this long.
    pub repeat_ms: i64,
    /// The Overlord hears of items quiet for this long.
    pub escalate_ms: i64,
    /// The digest repeats after this long.
    pub digest_repeat_ms: i64,
}

impl Default for ItemWakeConfig {
    fn default() -> Self {
        const HOUR: i64 = 3_600_000;
        const DAY: i64 = 24 * HOUR;
        Self {
            tick_ms: 5 * 60_000,
            stale_doing_ms: 4 * HOUR,
            stale_review_ms: 6 * HOUR,
            stale_todo_ms: DAY,
            repeat_ms: 12 * HOUR,
            escalate_ms: 3 * DAY,
            digest_repeat_ms: DAY,
        }
    }
}

impl ItemWakeConfig {
    /// The settings from the `items` section of config.json; a value that is not a positive number keeps its default.
    #[must_use]
    pub fn from_json(section: &Value) -> Self {
        let d = Self::default();
        Self {
            tick_ms: section_ms(section, "tickMs", d.tick_ms),
            stale_doing_ms: section_ms(section, "staleDoingMs", d.stale_doing_ms),
            stale_review_ms: section_ms(section, "staleReviewMs", d.stale_review_ms),
            stale_todo_ms: section_ms(section, "staleTodoMs", d.stale_todo_ms),
            repeat_ms: section_ms(section, "repeatMs", d.repeat_ms),
            escalate_ms: section_ms(section, "escalateMs", d.escalate_ms),
            digest_repeat_ms: section_ms(section, "digestRepeatMs", d.digest_repeat_ms),
        }
    }
}

/// When the wakes table last sent a reminder: `(scope, thread, reason, subject)` to epoch ms, 0 when never.
pub type LastSent<'a> = &'a dyn Fn(&str, &str, &str, &str) -> i64;

/// What one tick produced.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemTick {
    /// The wakes to submit.
    pub wakes: Vec<WakeInput>,
    /// The activity lines (one per digest).
    pub activity: Vec<ActivityLine>,
    /// The notices for the owner: the sweep alerts that go to him.
    pub owner: Vec<OwnerNotice>,
}

/// What a tick reads.
pub struct ItemEnv<'a> {
    /// The project boards.
    pub boards: &'a dyn BoardSource,
    /// The Overlord's thread, when it has one.
    pub main: Option<String>,
    /// The last reminder the wakes table knows, so a restart does not send it again at once.
    pub last_sent: LastSent<'a>,
}

/// The Kanban reminders.
pub struct ItemWakes {
    config: ItemWakeConfig,
    sent: HashMap<String, i64>,
}

impl ItemWakes {
    /// Reminders with a config.
    #[must_use]
    pub fn new(config: ItemWakeConfig) -> Self {
        Self {
            config,
            sent: HashMap::new(),
        }
    }

    /// The timing.
    #[must_use]
    pub fn config(&self) -> &ItemWakeConfig {
        &self.config
    }

    fn last_of(&self, env: &ItemEnv<'_>, group: &Group) -> i64 {
        let stored = (env.last_sent)(
            &group.scope,
            &group.thread,
            group.reason(),
            &group.subject(),
        );
        self.sent
            .get(&group.memory_key())
            .copied()
            .unwrap_or(0)
            .max(stored)
    }

    fn last_digest(&self, env: &ItemEnv<'_>, overlord: &str, main: &str, scope: &str) -> i64 {
        let stored = (env.last_sent)(overlord, main, REASON, scope);
        self.sent
            .get(&format!("digest/{scope}"))
            .copied()
            .unwrap_or(0)
            .max(stored)
    }

    /// The stale items of every board, not yet grouped.
    #[must_use]
    pub fn stale_items(&self, now: i64, boards: &dyn BoardSource) -> Vec<Stale> {
        kanban_sweep::stale_of(&kanban_sweep::sweep(&self.config, now, boards))
    }

    /// The wakes due now.
    pub fn tick(&mut self, now: i64, env: &ItemEnv<'_>) -> ItemTick {
        let findings = kanban_sweep::sweep(&self.config, now, env.boards);
        let stale = kanban_sweep::stale_of(&findings);
        let mut out = ItemTick::default();
        for mut group in kanban_sweep::groups(findings) {
            if now - self.last_of(env, &group) < group.repeat_ms {
                continue;
            }
            group
                .list
                .sort_by(|a, b| stale::worst_first(&a.stale, &b.stale));
            Self::tell(&group, now, &mut out);
            self.sent.insert(group.memory_key(), now);
        }
        if let Some(main) = &env.main {
            self.digests(now, env, main, &stale, &mut out);
        }
        out
    }

    /// One group told: a notice for the owner, else a wake (or an inbox notice) for its thread.
    fn tell(group: &Group, now: i64, out: &mut ItemTick) {
        let text = kanban_sweep::alert_text(group.kind, &group.list);
        if group.for_owner() {
            out.owner.push(owner_notice(group, text, now));
            return;
        }
        out.wakes.push(WakeInput::about(
            &group.scope,
            &group.thread,
            group.reason(),
            &group.subject(),
            text,
        ));
    }

    fn digests(
        &mut self,
        now: i64,
        env: &ItemEnv<'_>,
        main: &str,
        stale: &[Stale],
        out: &mut ItemTick,
    ) {
        let overlord = env.boards.overlord();
        for (scope, mut list) in stale::by_project(stale, &overlord, self.config.escalate_ms) {
            if now - self.last_digest(env, &overlord, main, &scope) < self.config.digest_repeat_ms {
                continue;
            }
            list.sort_by(stale::worst_first);
            let text = text::digest_text(&scope, &list, self.config.escalate_ms);
            out.activity.push(ActivityLine {
                kind: REASON.to_owned(),
                level: Level::Info,
                text: format!("kanban digest: {text}"),
                project: Some(scope.clone()),
                thread: None,
            });
            out.wakes
                .push(WakeInput::about(&overlord, main, REASON, &scope, text));
            self.sent.insert(format!("digest/{scope}"), now);
        }
    }
}

/// The notice for the owner of a group of breaks. Each telling is a notice of its own, so the repeat is
/// the sweep's `repeatMs` and nothing else.
fn owner_notice(group: &Group, text: String, now: i64) -> OwnerNotice {
    let kind = group.kind.key();
    OwnerNotice {
        topic: TOPIC_KANBAN.to_owned(),
        title: format!("Kanban {kind}: {} to look at", group.list.len()),
        text,
        project: group.scope.clone(),
        thread: "kanban".to_owned(),
        since: format!("{kind}:{now}"),
    }
}
