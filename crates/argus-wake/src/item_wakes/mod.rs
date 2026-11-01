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
//! Never reminded: done, dropped, archived or deleted items; an item that waits on someone else (an open
//! question or decision of the owner, a to-do of the owner, a manual test of the build handed to the
//! owner, a live sub-agent working on it); a thread that is paused. There are no quiet hours.

mod stale;
mod text;

pub use argus_core::clock::age_text;
pub use stale::Stale;

use std::collections::HashMap;

use serde_json::Value;

use crate::activity::{ActivityLine, Level};
use crate::config::section_ms;
use crate::reasons::MAIN_SCOPE;
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

    fn last_of(
        &self,
        env: &ItemEnv<'_>,
        mem_key: &str,
        scope: &str,
        thread: &str,
        subject: &str,
    ) -> i64 {
        let stored = (env.last_sent)(scope, thread, REASON, subject);
        self.sent.get(mem_key).copied().unwrap_or(0).max(stored)
    }

    /// The stale items of every board, not yet grouped.
    #[must_use]
    pub fn stale_items(&self, now: i64, boards: &dyn BoardSource) -> Vec<Stale> {
        stale::stale_items(&self.config, now, boards)
    }

    /// The wakes due now.
    pub fn tick(&mut self, now: i64, env: &ItemEnv<'_>) -> ItemTick {
        let stale = self.stale_items(now, env.boards);
        let mut out = ItemTick::default();
        for ((scope, thread), mut list) in stale::by_thread(&stale) {
            if now - self.last_of(env, &format!("{scope}/{thread}"), &scope, &thread, &thread)
                < self.config.repeat_ms
            {
                continue;
            }
            list.sort_by(stale::worst_first);
            out.wakes.push(WakeInput::about(
                &scope,
                &thread,
                REASON,
                &thread,
                text::reminder_text(&list),
            ));
            self.sent.insert(format!("{scope}/{thread}"), now);
        }
        if let Some(main) = &env.main {
            self.digests(now, env, main, &stale, &mut out);
        }
        out
    }

    fn digests(
        &mut self,
        now: i64,
        env: &ItemEnv<'_>,
        main: &str,
        stale: &[Stale],
        out: &mut ItemTick,
    ) {
        for (scope, mut list) in stale::by_project(stale, self.config.escalate_ms) {
            if now - self.last_of(env, &format!("digest/{scope}"), MAIN_SCOPE, main, &scope)
                < self.config.digest_repeat_ms
            {
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
                .push(WakeInput::about(MAIN_SCOPE, main, REASON, &scope, text));
            self.sent.insert(format!("digest/{scope}"), now);
        }
    }
}
