//! Showing one toast through the native notifier, with the tray balloon as the fallback.

use std::sync::Arc;

use argus_native::{Notifier, NotifyError, Toast};

use crate::level::Level;

/// The tray's balloon tip, tried when a toast cannot be shown.
pub trait TrayFallback: Send + Sync {
    /// Shows the balloon; false when there is no tray or it could not be shown.
    fn balloon(&self, title: &str, text: &str) -> bool;
}

/// One toast to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastRequest {
    /// The first line.
    pub title: String,
    /// The second line.
    pub text: String,
    /// The level.
    pub level: Level,
    /// The page a click opens.
    pub url: Option<String>,
    /// The replace tag.
    pub tag: String,
    /// True plays the system sound.
    pub sound: bool,
}

/// What became of one toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivered {
    /// Shown; the note says so when a tray balloon took its place.
    Shown(Option<String>),
    /// Not shown, and the reason.
    Failed(String),
}

/// The notifier and its fallback.
pub struct Delivery {
    notifier: Arc<dyn Notifier>,
    fallback: Option<Arc<dyn TrayFallback>>,
}

impl Delivery {
    /// A delivery through `notifier`; `fallback` is the tray balloon, when the tray provides one.
    #[must_use]
    pub fn new(notifier: Arc<dyn Notifier>, fallback: Option<Arc<dyn TrayFallback>>) -> Self {
        Self { notifier, fallback }
    }

    /// Shows the toast; when the platform refuses it the balloon is tried before giving up.
    #[must_use]
    pub fn deliver(&self, request: &ToastRequest) -> Delivered {
        match self.notifier.show(&native_toast(request)) {
            Ok(()) => Delivered::Shown(None),
            Err(error) => self.rescue(request, reason_of(&error)),
        }
    }

    fn rescue(&self, request: &ToastRequest, reason: String) -> Delivered {
        let shown = self
            .fallback
            .as_ref()
            .is_some_and(|tray| tray.balloon(&request.title, &request.text));
        if shown {
            Delivered::Shown(Some(format!("shown as a tray balloon ({reason})")))
        } else {
            Delivered::Failed(reason)
        }
    }
}

fn native_toast(request: &ToastRequest) -> Toast {
    let toast = Toast::new(&request.title, &request.text)
        .level(request.level.native())
        .tag(&request.tag)
        .sound(request.sound);
    match &request.url {
        Some(url) => toast.url(url),
        None => toast,
    }
}

fn reason_of(error: &NotifyError) -> String {
    match error {
        NotifyError::Platform(why) => why.clone(),
        other => other.to_string(),
    }
}
