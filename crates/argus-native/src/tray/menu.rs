//! The context menu as data: which entries, in which order, which are on. Pure, so it is tested
//! without a window; `view` turns it into a native menu.
// test-tag: critical

use super::state::TrayState;

/// What a menu item asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Open the Argus page (also a click on the icon).
    Open,
    /// Open the page on its notifications tab.
    OpenNotifications,
    /// Open the folder of the logs.
    OpenLogs,
    /// Stop the server, then start it.
    Restart,
    /// Start the stopped server.
    Start,
    /// Stop the server, write `quit.flag` and close the icon.
    Quit,
}

impl Command {
    /// The id of the menu item.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::OpenNotifications => "notifications",
            Self::OpenLogs => "logs",
            Self::Restart => "restart",
            Self::Start => "start",
            Self::Quit => "quit",
        }
    }

    /// The command whose item has this id.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        [
            Self::Open,
            Self::OpenNotifications,
            Self::OpenLogs,
            Self::Restart,
            Self::Start,
            Self::Quit,
        ]
        .into_iter()
        .find(|command| command.id() == id)
    }
}

/// One line of the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// An item that asks for a command.
    Action {
        /// What it asks for.
        command: Command,
        /// The text of the item.
        label: &'static str,
        /// False greys the item out.
        enabled: bool,
    },
    /// The disabled line with the server state.
    Status(String),
    /// A separator line.
    Separator,
}

const fn action(command: Command, label: &'static str, enabled: bool) -> Entry {
    Entry::Action {
        command,
        label,
        enabled,
    }
}

/// The menu for a state: Open Argus, the status line, Notifications, Open log folder, Restart
/// server, Start server (only while the server is down), Quit Argus. While Quit is carried out
/// Restart, Start and Quit are greyed out.
#[must_use]
pub fn entries(state: &TrayState) -> Vec<Entry> {
    let idle = !state.is_closing();
    let mut menu = vec![
        action(Command::Open, "Open Argus", true),
        Entry::Status(state.status_label()),
        Entry::Separator,
        action(Command::OpenNotifications, "Notifications", true),
        action(Command::OpenLogs, "Open log folder", true),
        action(Command::Restart, "Restart server", idle),
    ];
    if state.is_stopped() {
        menu.push(action(Command::Start, "Start server", true));
    }
    menu.push(Entry::Separator);
    menu.push(action(Command::Quit, "Quit Argus", idle));
    menu
}

#[cfg(test)]
mod tests {
    use super::{Command, Entry, entries};
    use crate::tray::state::TrayState;

    fn labels(state: &TrayState) -> Vec<String> {
        entries(state)
            .into_iter()
            .map(|entry| match entry {
                Entry::Action { label, .. } => label.to_owned(),
                Entry::Status(text) => text,
                Entry::Separator => "-".to_owned(),
            })
            .collect()
    }

    #[test]
    fn a_running_server_has_no_start_item() {
        assert_eq!(
            labels(&TrayState::running(2)),
            [
                "Open Argus",
                "Argus: running (2 runs)",
                "-",
                "Notifications",
                "Open log folder",
                "Restart server",
                "-",
                "Quit Argus"
            ]
        );
    }

    /// The Node tray (`tray.ps1`): Open dashboard, the status line, a separator, Open log folder,
    /// Restart server, Start server (only while stopped), a separator, Quit dashboard. Argus
    /// differs in two ways on purpose: the app name in the labels, and one more item,
    /// Notifications, after the first separator.
    #[test]
    fn the_menu_is_the_node_menu_with_the_argus_name_and_the_notifications_item() {
        let node = |state: &TrayState| -> Vec<String> {
            let mut menu = labels(state);
            menu.retain(|label| label != "Notifications");
            menu.iter()
                .map(|label| label.replace("Open Argus", "Open dashboard"))
                .map(|label| label.replace("Quit Argus", "Quit dashboard"))
                .collect()
        };
        assert_eq!(
            node(&TrayState::stopped()),
            [
                "Open dashboard",
                "Argus: stopped",
                "-",
                "Open log folder",
                "Restart server",
                "Start server",
                "-",
                "Quit dashboard"
            ]
        );
        assert!(!node(&TrayState::running(1)).contains(&"Start server".to_owned()));
        assert_eq!(labels(&TrayState::stopped())[3], "Notifications");
    }

    #[test]
    fn quitting_greys_out_restart_start_and_quit() {
        let all_on = |state: &TrayState| {
            entries(state)
                .iter()
                .all(|e| !matches!(e, Entry::Action { enabled: false, .. }))
        };
        assert!(all_on(&TrayState::running(0)));
        assert!(!all_on(&TrayState::running(0).closing()));
        let closing = entries(&TrayState::stopped().closing());
        let off: Vec<Command> = closing
            .iter()
            .filter_map(|e| match e {
                Entry::Action {
                    command,
                    enabled: false,
                    ..
                } => Some(*command),
                _ => None,
            })
            .collect();
        assert_eq!(off, [Command::Restart, Command::Quit]);
    }

    #[test]
    fn the_first_item_opens_argus_and_every_id_maps_back() {
        assert!(matches!(
            entries(&TrayState::default())[0],
            Entry::Action {
                command: Command::Open,
                ..
            }
        ));
        for command in [
            Command::Open,
            Command::OpenNotifications,
            Command::OpenLogs,
            Command::Restart,
            Command::Start,
            Command::Quit,
        ] {
            assert_eq!(Command::from_id(command.id()), Some(command));
        }
        assert_eq!(Command::from_id("nope"), None);
    }
}
