//! The toast a caller asks for: plain data, checked when its payload is built.

/// How loud a toast is, as the notification levels of the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Level {
    /// Recorded only by the dashboard; a toast is shown when the owner switches it on.
    Info,
    /// The default: a toast that goes to the action center when it times out.
    #[default]
    Ping,
    /// A reminder toast that stays on screen until it is dismissed.
    Urgent,
}

/// One toast. Title and text are plain text and are cleaned when the payload is built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    /// The first line, at most 60 characters after cleaning.
    pub title: String,
    /// The second line, at most 240 characters after cleaning.
    pub text: String,
    /// How loud the toast is.
    pub level: Level,
    /// The page a click opens: an `http` or `https` address.
    pub url: Option<String>,
    /// A newer toast with the same tag replaces its older one in the action center.
    pub tag: Option<String>,
    /// True plays the system notification sound, false shows a silent toast.
    pub sound: bool,
}

impl Toast {
    /// A `ping` toast with the system sound, no address and no tag.
    #[must_use]
    pub fn new(title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            text: text.into(),
            level: Level::Ping,
            url: None,
            tag: None,
            sound: true,
        }
    }

    /// Sets the level.
    #[must_use]
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Sets the page a click opens.
    #[must_use]
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the replace tag.
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Sets whether the toast plays the system sound.
    #[must_use]
    pub fn sound(mut self, sound: bool) -> Self {
        self.sound = sound;
        self
    }
}
