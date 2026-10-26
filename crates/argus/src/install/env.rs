//! The machine the install verbs look at: which platform, the folders the config files live
//! under and the binary itself. Tests build their own, so no verb needs the real environment.

use std::path::PathBuf;

/// The platforms whose Claude Desktop config path differs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Windows.
    Windows,
    /// macOS.
    Mac,
    /// Linux and the rest.
    Other,
}

impl Platform {
    /// The platform this binary runs on.
    #[must_use]
    pub fn current() -> Self {
        if cfg!(windows) {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::Mac
        } else {
            Platform::Other
        }
    }
}

/// The facts of the machine an install verb reads.
#[derive(Debug, Clone)]
pub struct Env {
    /// The platform.
    pub platform: Platform,
    /// `APPDATA`.
    pub appdata: Option<PathBuf>,
    /// `XDG_CONFIG_HOME`.
    pub xdg_config_home: Option<PathBuf>,
    /// The user's home folder.
    pub home: PathBuf,
    /// The folder the command runs in.
    pub cwd: PathBuf,
    /// The `argus` binary.
    pub exe: PathBuf,
}

fn var(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

impl Env {
    /// The real machine.
    #[must_use]
    pub fn process() -> Self {
        Env {
            platform: Platform::current(),
            appdata: var("APPDATA"),
            xdg_config_home: var("XDG_CONFIG_HOME"),
            home: var("USERPROFILE")
                .or_else(|| var("HOME"))
                .unwrap_or_default(),
            cwd: std::env::current_dir().unwrap_or_default(),
            exe: std::env::current_exe().unwrap_or_default(),
        }
    }

    /// `APPDATA`, or the folder it names by default.
    #[must_use]
    pub fn roaming(&self) -> PathBuf {
        self.appdata
            .clone()
            .unwrap_or_else(|| self.home.join("AppData").join("Roaming"))
    }
}

/// Where a verb writes: one sink for the answer and one for what is said around it.
pub struct Io<'a> {
    /// Standard output.
    pub out: &'a mut dyn FnMut(&str),
    /// Standard error.
    pub err: &'a mut dyn FnMut(&str),
    /// The machine.
    pub env: Env,
}
