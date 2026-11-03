//! Who the toasts come from: the app identity Windows shows on a toast and groups it under.
// test-tag: critical

use std::path::PathBuf;

/// The `AppUserModelID` of Argus.
pub const ARGUS_AUMID: &str = "Argus.Dashboard";
/// The name Windows shows on an Argus toast.
pub const ARGUS_NAME: &str = "Argus";

/// An application identity for toasts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The `AppUserModelID`: letters, digits and dots, no spaces.
    pub aumid: String,
    /// The name shown at the top of a toast and in the notification settings.
    pub display_name: String,
    /// An absolute path to the icon file (`static/argus.ico`).
    pub icon: Option<PathBuf>,
}

impl Identity {
    /// The identity of Argus, with its icon file when there is one.
    #[must_use]
    pub fn argus(icon: Option<PathBuf>) -> Self {
        Self {
            aumid: ARGUS_AUMID.to_owned(),
            display_name: ARGUS_NAME.to_owned(),
            icon,
        }
    }

    /// The registry key, under `HKEY_CURRENT_USER`, that makes this identity known to Windows.
    #[must_use]
    pub fn registry_path(&self) -> String {
        format!(r"Software\Classes\AppUserModelId\{}", self.aumid)
    }

    /// The string values of that key: `DisplayName`, and `IconUri` when there is an icon.
    #[must_use]
    pub fn registry_values(&self) -> Vec<(&'static str, String)> {
        let mut values = vec![("DisplayName", self.display_name.clone())];
        if let Some(icon) = &self.icon {
            values.push(("IconUri", icon.display().to_string()));
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Identity;

    #[test]
    fn the_argus_identity_registers_a_name_and_an_icon() {
        let identity = Identity::argus(Some(PathBuf::from(r"X:\argus\static\argus.ico")));
        assert_eq!(
            identity.registry_path(),
            r"Software\Classes\AppUserModelId\Argus.Dashboard"
        );
        assert_eq!(
            identity.registry_values(),
            vec![
                ("DisplayName", "Argus".to_owned()),
                ("IconUri", r"X:\argus\static\argus.ico".to_owned()),
            ]
        );
    }

    #[test]
    fn without_an_icon_only_the_name_is_registered() {
        let values = Identity::argus(None).registry_values();
        assert_eq!(values, vec![("DisplayName", "Argus".to_owned())]);
    }
}
