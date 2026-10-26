//! Whether a notification gets a toast: the level and config rules, then the rate limits.

use crate::config::NotifyConfig;
use crate::level::Level;
use crate::naming::limit_key;
use crate::record::Message;
use crate::service::{Notify, locked};
use crate::settings::Settings;

pub(crate) enum Verdict {
    Skip(String),
    Limited { why: String, summary: Option<usize> },
    Attempt,
}

fn skip_reason(level: Level, config: &NotifyConfig, settings: Settings) -> Option<String> {
    if level == Level::Info && !config.toast_info {
        Some("info is recorded only".to_owned())
    } else if !config.os {
        Some("OS notifications are off (config notify.os)".to_owned())
    } else if !settings.toast.get(level) {
        Some(format!(
            "{} is a quiet level (config notify.quietLevels)",
            level.as_str()
        ))
    } else {
        None
    }
}

impl Notify {
    pub(crate) fn verdict(
        &self,
        message: &Message,
        config: &NotifyConfig,
        settings: Settings,
        now: i64,
    ) -> Verdict {
        if let Some(why) = skip_reason(message.level, config, settings) {
            return Verdict::Skip(why);
        }
        let key = limit_key(message.project.as_deref(), message.thread.as_deref());
        let mut limiter = locked(&self.limiter);
        match limiter.admit(&key, now) {
            None => Verdict::Attempt,
            Some(why) => Verdict::Limited {
                why,
                summary: limiter.hold(&key, now),
            },
        }
    }
}
