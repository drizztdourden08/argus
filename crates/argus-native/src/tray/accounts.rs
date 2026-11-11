//! The account lines of the tray menu: one line per Claude account with its weekly usage, the
//! switch actions, and whether a switch countdown runs. Pure, so it is tested without a window.
// test-tag: standard

/// One account as the menu shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountLine {
    /// The account key the switch action carries.
    pub key: String,
    /// The name the menu shows.
    pub label: String,
    /// The weekly usage in percent, `None` before a usage read.
    pub weekly: Option<u32>,
    /// True when Desktop runs as this account: it has no switch action.
    pub active: bool,
}

impl AccountLine {
    /// The disabled line of the account: `Hotmail: weekly 42%`.
    #[must_use]
    pub fn text(&self) -> String {
        match self.weekly {
            Some(percent) => format!("{}: weekly {percent}%", self.label),
            None => format!("{}: weekly unknown", self.label),
        }
    }
}

/// The accounts section of the menu.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountLines {
    /// The accounts, in the order the menu shows them.
    pub lines: Vec<AccountLine>,
    /// True while a switch countdown runs: the menu offers Cancel switch.
    pub switching: bool,
}

#[cfg(test)]
mod tests {
    use super::AccountLine;

    fn line(weekly: Option<u32>) -> AccountLine {
        AccountLine {
            key: "hotmail".to_owned(),
            label: "Hotmail".to_owned(),
            weekly,
            active: false,
        }
    }

    #[test]
    fn a_line_says_the_weekly_percent_or_that_it_is_unknown() {
        assert_eq!(line(Some(42)).text(), "Hotmail: weekly 42%");
        assert_eq!(line(None).text(), "Hotmail: weekly unknown");
    }
}
