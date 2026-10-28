//! The taskbar theme: which ink the mark takes to stand out on it.

use windows_registry::CURRENT_USER;

use crate::tray::icon::Ink;

const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const LIGHT_VALUE: &str = "SystemUsesLightTheme";

/// The ink for the taskbar as it is now: black on a light one, white on a dark one or when
/// Windows does not say.
pub fn ink() -> Ink {
    let value = CURRENT_USER
        .open(PERSONALIZE)
        .and_then(|key| key.get_u32(LIGHT_VALUE))
        .ok();
    Ink::for_light_theme(value)
}
