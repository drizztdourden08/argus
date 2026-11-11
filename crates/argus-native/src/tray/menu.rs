//! The context menu as data: which entries, in which order, which are on. Pure, so it is tested
//! without a window; `view` turns it into a native menu.
// test-tag: critical

use super::accounts::AccountLines;
use super::state::TrayState;

/// What a menu item asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Open the page on the accounts tab to switch Desktop to the account with this key.
    SwitchDesktop(String),
    /// Cancel the switch countdown that runs.
    CancelSwitch,
}

const SWITCH_PREFIX: &str = "switch:";

impl Command {
    /// The id of the menu item.
    #[must_use]
    pub const fn id(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::OpenNotifications => "notifications",
            Self::OpenLogs => "logs",
            Self::Restart => "restart",
            Self::Start => "start",
            Self::Quit => "quit",
            Self::SwitchDesktop(_) => "switch-desktop",
            Self::CancelSwitch => "cancel-switch",
        }
    }

    /// The id of the native menu item: the account switch carries its key.
    #[must_use]
    pub fn item_id(&self) -> String {
        match self {
            Self::SwitchDesktop(key) => format!("{SWITCH_PREFIX}{key}"),
            other => other.id().to_owned(),
        }
    }

    /// The command whose item has this id.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        if let Some(key) = id.strip_prefix(SWITCH_PREFIX) {
            return Some(Self::SwitchDesktop(key.to_owned()));
        }
        [
            Self::Open,
            Self::OpenNotifications,
            Self::OpenLogs,
            Self::Restart,
            Self::Start,
            Self::Quit,
            Self::CancelSwitch,
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
    /// An item of the accounts section: its text is owned, since it names an account.
    Account {
        /// What it asks for.
        command: Command,
        /// The text of the item.
        label: String,
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

fn account_section(accounts: &AccountLines, idle: bool) -> Vec<Entry> {
    if accounts.lines.is_empty() {
        return Vec::new();
    }
    let mut section: Vec<Entry> = accounts
        .lines
        .iter()
        .map(|line| Entry::Status(line.text()))
        .collect();
    for line in accounts.lines.iter().filter(|line| !line.active) {
        section.push(Entry::Account {
            command: Command::SwitchDesktop(line.key.clone()),
            label: format!("Switch Desktop to {}", line.label),
            enabled: idle,
        });
    }
    if accounts.switching {
        section.push(Entry::Account {
            command: Command::CancelSwitch,
            label: "Cancel switch".to_owned(),
            enabled: idle,
        });
    }
    section.push(Entry::Separator);
    section
}

/// The menu for a state: Open Argus, the status line, Notifications, Open log folder, Restart
/// server, Start server (only while the server is down), Quit Argus. While Quit is carried out
/// Restart, Start and Quit are greyed out.
#[must_use]
pub fn entries(state: &TrayState) -> Vec<Entry> {
    entries_with(state, &AccountLines::default())
}

/// [`entries`] with the accounts section after the status line: one disabled line per account,
/// a Switch Desktop action for each account that is not active, and Cancel switch while a
/// countdown runs. No accounts, no section.
#[must_use]
pub fn entries_with(state: &TrayState, accounts: &AccountLines) -> Vec<Entry> {
    let idle = !state.is_closing();
    let mut menu = vec![
        action(Command::Open, "Open Argus", true),
        Entry::Status(state.status_label()),
        Entry::Separator,
    ];
    menu.extend(account_section(accounts, idle));
    menu.extend([
        action(Command::OpenNotifications, "Notifications", true),
        action(Command::OpenLogs, "Open log folder", true),
        action(Command::Restart, "Restart server", idle),
    ]);
    if state.is_stopped() {
        menu.push(action(Command::Start, "Start server", true));
    }
    menu.push(Entry::Separator);
    menu.push(action(Command::Quit, "Quit Argus", idle));
    menu
}

#[cfg(test)]
mod tests {
    use super::{Command, Entry, entries, entries_with};
    use crate::tray::accounts::{AccountLine, AccountLines};
    use crate::tray::state::TrayState;

    fn labels(state: &TrayState) -> Vec<String> {
        entries(state)
            .into_iter()
            .map(|entry| match entry {
                Entry::Action { label, .. } => label.to_owned(),
                Entry::Account { label, .. } => label,
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
                } => Some(command.clone()),
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
            assert_eq!(Command::from_id(&command.item_id()), Some(command));
        }
        assert_eq!(Command::from_id("nope"), None);
    }

    fn two_accounts(switching: bool) -> AccountLines {
        let line = |key: &str, label: &str, weekly, active| AccountLine {
            key: key.to_owned(),
            label: label.to_owned(),
            weekly,
            active,
        };
        AccountLines {
            lines: vec![
                line("hotmail", "Hotmail", Some(42), true),
                line("second", "Second", None, false),
            ],
            switching,
        }
    }

    #[test]
    fn accounts_list_their_weekly_usage_and_offer_a_switch_to_the_inactive_ones() {
        let menu = entries_with(&TrayState::running(0), &two_accounts(false));
        let text: Vec<String> = menu
            .iter()
            .filter_map(|e| match e {
                Entry::Status(t) => Some(t.clone()),
                Entry::Account { label, .. } => Some(label.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            text,
            [
                "Argus: running (0 runs)",
                "Hotmail: weekly 42%",
                "Second: weekly unknown",
                "Switch Desktop to Second"
            ]
        );
        assert!(menu.contains(&Entry::Account {
            command: Command::SwitchDesktop("second".to_owned()),
            label: "Switch Desktop to Second".to_owned(),
            enabled: true,
        }));
    }

    #[test]
    fn cancel_switch_shows_only_while_a_countdown_runs() {
        let has_cancel = |lines: &AccountLines| {
            entries_with(&TrayState::running(0), lines).iter().any(|e| {
                matches!(
                    e,
                    Entry::Account {
                        command: Command::CancelSwitch,
                        ..
                    }
                )
            })
        };
        assert!(!has_cancel(&two_accounts(false)));
        assert!(has_cancel(&two_accounts(true)));
    }

    #[test]
    fn no_accounts_no_section_and_quitting_greys_the_actions() {
        assert_eq!(
            entries_with(&TrayState::running(0), &AccountLines::default()),
            entries(&TrayState::running(0))
        );
        let closing = entries_with(&TrayState::running(0).closing(), &two_accounts(true));
        assert!(
            closing
                .iter()
                .all(|e| !matches!(e, Entry::Account { enabled: true, .. }))
        );
    }
}
