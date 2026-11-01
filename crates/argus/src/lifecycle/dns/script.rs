//! The text of `argus dns install --print` and `argus dns remove --print`: the one-time
//! PowerShell the owner runs himself, in an elevated window. Argus never runs it and never
//! changes a system setting; every line that is not a command is a PowerShell comment, so the
//! whole text can be pasted as it is.

use argus_core::names::LocalNames;

/// The setup script: one NRPT rule that sends `.<suffix>` to the responder of Argus.
#[must_use]
pub fn install(names: &LocalNames) -> String {
    let (suffix, address) = (&names.suffix, names.address);
    format!(
        "# Run this once in a PowerShell opened as administrator. Argus never runs it itself.\n\
         # It sends every name ending in .{suffix} to the Argus responder at {address} and changes nothing else.\n\
         Add-DnsClientNrptRule -Namespace \".{suffix}\" -NameServers \"{address}\"\n\
         #\n\
         # Then set \"localNames\": {{ \"enabled\": true }} in the config.json of the dashboard home\n\
         # and restart Argus (argus stop, then argus start). Check it with: argus dns status\n\
         # Chrome and Edge honour NRPT rules. Type the first visit with http:// in front\n\
         # (http://argus.{suffix}/) so the browser does not search for the name.\n"
    )
}

/// The undo script: removes the rule `install` added.
#[must_use]
pub fn remove(names: &LocalNames) -> String {
    let suffix = &names.suffix;
    format!(
        "# Run this once in a PowerShell opened as administrator. Argus never runs it itself.\n\
         # It removes the rule that sends names ending in .{suffix} to the Argus responder.\n\
         Get-DnsClientNrptRule | Where-Object {{ $_.Namespace -contains \".{suffix}\" }} | Remove-DnsClientNrptRule -Force\n"
    )
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K2, K4: the printed NRPT script is the one rule and nothing else.
    use argus_core::names::LocalNames;

    use super::{install, remove};

    #[test]
    fn install_is_the_one_nrpt_rule_for_the_suffix_and_address() {
        let text = install(&LocalNames::default());
        assert!(text.contains(
            "Add-DnsClientNrptRule -Namespace \".argus\" -NameServers \"127.65.82.71\"\n"
        ));
        assert!(text.contains("http://argus.argus/"));
    }

    #[test]
    fn remove_is_the_matching_removal() {
        let names = LocalNames {
            suffix: "home.argus".to_owned(),
            ..LocalNames::default()
        };
        let text = remove(&names);
        assert!(text.contains("Remove-DnsClientNrptRule"));
        assert!(text.contains("\".home.argus\""));
        assert!(!text.contains("Add-DnsClientNrptRule"));
    }

    #[test]
    fn every_line_is_a_comment_or_a_dns_client_command() {
        for text in [
            install(&LocalNames::default()),
            remove(&LocalNames::default()),
        ] {
            for line in text.lines() {
                assert!(
                    line.starts_with('#') || line.contains("DnsClientNrptRule"),
                    "{line}"
                );
            }
        }
    }
}
