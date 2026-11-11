//! Turns the menu model into a native popup menu. Windows draws it, with the rounded corners of
//! Windows 11, in the light or dark mode of the system.

use tray_icon::menu::{Menu, MenuId, MenuItem, PredefinedMenuItem};

use crate::tray::error::TrayError;
use crate::tray::menu::Entry;

/// Builds the native menu for the entries.
pub fn build(entries: &[Entry]) -> Result<Menu, TrayError> {
    let menu = Menu::new();
    for entry in entries {
        let added = match entry {
            Entry::Action {
                command,
                label,
                enabled,
            } => menu.append(&MenuItem::with_id(
                MenuId::new(command.item_id()),
                label,
                *enabled,
                None,
            )),
            Entry::Account {
                command,
                label,
                enabled,
            } => menu.append(&MenuItem::with_id(
                MenuId::new(command.item_id()),
                label,
                *enabled,
                None,
            )),
            Entry::Status(text) => menu.append(&MenuItem::new(text, false, None)),
            Entry::Separator => menu.append(&PredefinedMenuItem::separator()),
        };
        added.map_err(|e| TrayError::Platform(e.to_string()))?;
    }
    Ok(menu)
}
