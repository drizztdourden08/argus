//! `send`: clean a ping, decide on its toast, show it, record it, and tell the page.

use argus_native::{TEXT_MAX, TITLE_MAX, plain};

use crate::clock::iso;
use crate::deliver::{Delivered, ToastRequest};
use crate::gate::Verdict;
use crate::kinds::ToastState;
use crate::level::Level;
use crate::lines::activity_line;
use crate::naming::{key_part, label_of, tag_of, title_from};
use crate::record::Message;
use crate::request::{Outcome, PingRequest};
use crate::service::Notify;
use crate::settings::Settings;

const WHY_MAX: usize = 160;
const NO_REASON: &str = "the toast could not be shown";

impl Notify {
    /// Records a notification and, when its level says so, shows the toast. A ping over a limit is
    /// recorded as `limited`, never refused; a ping with no text is not recorded. It never fails.
    pub fn send(&self, request: PingRequest) -> Outcome {
        let text = plain(&request.text, TEXT_MAX);
        if text.is_empty() {
            return Outcome::unrecorded(ToastState::Skipped, "a notification needs text");
        }
        let config = self.ports.config.load();
        let settings = crate::settings::settings_of(&config);
        let message = self.compose(request, text, settings);
        let verdict = self.verdict(&message, &config, settings, self.ports.clock.now_ms());
        let (toast, why) = self.settle(&message, &verdict, settings);
        let (project, thread) = (message.project.clone(), message.thread.clone());
        let outcome = self.record(message, toast, why);
        if let Verdict::Limited {
            summary: Some(count),
            ..
        } = verdict
        {
            self.send_summary(project.as_deref(), thread.as_deref(), count, settings);
        }
        outcome
    }

    fn compose(&self, request: PingRequest, text: String, settings: Settings) -> Message {
        let project = key_part(request.project.as_deref());
        let thread = key_part(request.thread.as_deref());
        let given = plain(&request.title.unwrap_or_default(), TITLE_MAX);
        let title = if given.is_empty() {
            let name = project.as_deref().map(|key| self.name_of(key));
            plain(&title_from(name.as_deref(), thread.as_deref()), TITLE_MAX)
        } else {
            given
        };
        let toasts = request.level != Level::Info || settings.toast.info;
        let url = toasts.then(|| {
            self.links
                .url_for(project.as_deref(), request.url.as_deref())
        });
        Message {
            title,
            text,
            level: request.level,
            project,
            thread,
            by: request.by,
            url,
        }
    }

    fn settle(
        &self,
        message: &Message,
        verdict: &Verdict,
        settings: Settings,
    ) -> (ToastState, Option<String>) {
        match verdict {
            Verdict::Skip(why) => (ToastState::Skipped, Some(why.clone())),
            Verdict::Limited { why, .. } => (ToastState::Limited, Some(why.clone())),
            Verdict::Attempt => self.attempt(message, settings),
        }
    }

    fn attempt(&self, message: &Message, settings: Settings) -> (ToastState, Option<String>) {
        let request = ToastRequest {
            title: message.title.clone(),
            text: message.text.clone(),
            level: message.level,
            url: message.url.clone(),
            tag: tag_of(message.project.as_deref(), message.thread.as_deref()),
            sound: settings.sound.get(message.level),
        };
        match self.ports.delivery.deliver(&request) {
            Delivered::Shown(note) => (ToastState::Shown, note),
            Delivered::Failed(reason) if reason.is_empty() => {
                (ToastState::Failed, Some(NO_REASON.to_owned()))
            }
            Delivered::Failed(reason) => (ToastState::Failed, Some(reason)),
        }
    }

    fn record(&self, message: Message, toast: ToastState, why: Option<String>) -> Outcome {
        let why = why
            .map(|why| plain(&why, WHY_MAX))
            .filter(|why| !why.is_empty());
        let record = message.record(iso(self.ports.clock.now_ms()), toast, why);
        if let Ok(note) = self.ports.store.insert(record.clone()) {
            self.ports.events.recorded(&note);
            if let Some(line) = activity_line(&record) {
                self.ports.activity.note(line);
            }
            Outcome::recorded(note)
        } else {
            self.warn("notification not recorded");
            Outcome::unrecorded(ToastState::Failed, "the notification could not be stored")
        }
    }

    fn send_summary(
        &self,
        project: Option<&str>,
        thread: Option<&str>,
        count: usize,
        settings: Settings,
    ) {
        let plural = if count == 1 { "" } else { "s" };
        let label = label_of(project, thread);
        let request = ToastRequest {
            title: "Argus".to_owned(),
            text: format!("{count} more notification{plural} from {label}: see Argus"),
            level: Level::Ping,
            url: Some(self.links.url_for(None, None)),
            tag: tag_of(project, thread),
            sound: settings.sound.ping,
        };
        let _ = self.ports.delivery.deliver(&request);
    }
}
