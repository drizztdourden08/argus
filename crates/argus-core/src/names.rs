//! The local names of Argus: a name suffix of its own (`argus.argus`, `rotp.argus`) on a loopback
//! address of its own, read from the `localNames` key of `config.json`.
//!
//! The key is `{ "enabled": false, "address": "127.65.82.71", "suffix": "argus" }`. Every field
//! is optional and the names are off until the owner sets `enabled`. The address must be a
//! loopback address other than `127.0.0.1`, so nothing else on the machine ever binds it and
//! ports 53 and 80 on it can never conflict with other software.

use std::net::Ipv4Addr;

use serde_json::Value;

use crate::Config;

/// The key of `config.json`.
pub const KEY: &str = "localNames";
/// The address when the config names none.
pub const DEFAULT_ADDRESS: Ipv4Addr = Ipv4Addr::new(127, 65, 82, 71);
/// The suffix when the config names none.
pub const DEFAULT_SUFFIX: &str = "argus";
/// The port a front door tries first.
pub const HTTP_PORT: u16 = 80;
const SUFFIX_MAX: usize = 100;
const LABEL_MAX: usize = 63;

/// The `localNames` setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalNames {
    /// Whether the names are on.
    pub enabled: bool,
    /// The loopback address of Argus.
    pub address: Ipv4Addr,
    /// The suffix, lowercase, without the leading dot.
    pub suffix: String,
}

impl Default for LocalNames {
    fn default() -> Self {
        Self {
            enabled: false,
            address: DEFAULT_ADDRESS,
            suffix: DEFAULT_SUFFIX.to_owned(),
        }
    }
}

impl LocalNames {
    /// The setting of a config; a missing key is the default (off).
    ///
    /// # Errors
    ///
    /// A sentence naming the field that is refused.
    pub fn of(config: &Config) -> Result<Self, String> {
        config
            .extra
            .get(KEY)
            .map_or_else(|| Ok(Self::default()), Self::parse)
    }

    /// The setting of a JSON value; a missing field is the default.
    ///
    /// # Errors
    ///
    /// A sentence naming the field that is refused.
    pub fn parse(value: &Value) -> Result<Self, String> {
        let Some(object) = value.as_object() else {
            return Err(format!("{KEY} must be an object"));
        };
        let mut out = Self::default();
        if let Some(enabled) = object.get("enabled") {
            out.enabled = enabled
                .as_bool()
                .ok_or_else(|| format!("{KEY}.enabled must be true or false"))?;
        }
        if let Some(address) = object.get("address") {
            out.address = address_of(address)?;
        }
        if let Some(suffix) = object.get("suffix") {
            out.suffix = suffix_of(suffix)?;
        }
        Ok(out)
    }
}

fn address_of(value: &Value) -> Result<Ipv4Addr, String> {
    let text = value.as_str().unwrap_or_default();
    let address: Ipv4Addr = text
        .parse()
        .map_err(|_| format!("{KEY}.address must be an IPv4 address, got {value}"))?;
    if address.octets()[0] != 127 || address == Ipv4Addr::LOCALHOST {
        return Err(format!(
            "{KEY}.address must be a loopback address in 127.0.0.0/8 other than 127.0.0.1, got {address}"
        ));
    }
    Ok(address)
}

fn suffix_of(value: &Value) -> Result<String, String> {
    let given = value.as_str().unwrap_or_default().to_ascii_lowercase();
    let suffix = given.trim_start_matches('.');
    let labels_ok = suffix.split('.').all(label_ok);
    if suffix.len() > SUFFIX_MAX || !labels_ok || suffix == "localhost" {
        return Err(format!(
            "{KEY}.suffix must be letters, digits and dashes (dots between labels), and not localhost, got {value}"
        ));
    }
    Ok(suffix.to_owned())
}

fn label_ok(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= LABEL_MAX
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// The front door of the names: where the server answers on the address of Argus. A browser sends
/// `Host: rotp.argus` for port 80 and `Host: rotp.argus:47615` for any other port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Door {
    /// The address of Argus.
    pub address: Ipv4Addr,
    /// The suffix.
    pub suffix: String,
    /// The port the server holds on the address.
    pub port: u16,
}

impl Door {
    /// The suffix as a `Host` header carries it: with the port unless it is 80.
    #[must_use]
    pub fn bare(&self) -> String {
        if self.port == HTTP_PORT {
            self.suffix.clone()
        } else {
            format!("{}:{}", self.suffix, self.port)
        }
    }

    /// The `Host` of a name under the suffix.
    #[must_use]
    pub fn host(&self, label: &str) -> String {
        format!("{label}.{}", self.bare())
    }

    /// The address of a page under a name.
    #[must_use]
    pub fn url(&self, label: &str, path: &str) -> String {
        format!("http://{}{path}", self.host(label))
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use serde_json::json;

    use super::{DEFAULT_ADDRESS, Door, LocalNames};

    #[test]
    fn no_key_is_off_with_the_default_address_and_suffix() {
        let names = LocalNames::default();
        assert!(!names.enabled);
        assert_eq!(names.address, DEFAULT_ADDRESS);
        assert_eq!(names.suffix, "argus");
    }

    #[test]
    fn a_partial_key_keeps_the_defaults_and_cleans_the_suffix() {
        let names = LocalNames::parse(&json!({ "enabled": true, "suffix": ".Home.Argus" }))
            .expect("parses");
        assert_eq!(names.suffix, "home.argus");
        assert_eq!(names.address, DEFAULT_ADDRESS);
        assert!(names.enabled);
    }

    #[test]
    fn refused_values_name_their_field() {
        let cases = [
            (json!(5), "must be an object"),
            (json!({ "enabled": "yes" }), "localNames.enabled"),
            (json!({ "address": "127.0.0.1" }), "localNames.address"),
            (json!({ "address": "10.0.0.5" }), "localNames.address"),
            (json!({ "address": "nope" }), "localNames.address"),
            (json!({ "suffix": "localhost" }), "localNames.suffix"),
            (json!({ "suffix": "a..b" }), "localNames.suffix"),
            (json!({ "suffix": "-a" }), "localNames.suffix"),
            (json!({ "suffix": "a b" }), "localNames.suffix"),
            (json!({ "suffix": "" }), "localNames.suffix"),
        ];
        for (value, part) in cases {
            let error = LocalNames::parse(&value).expect_err(&value.to_string());
            assert!(error.contains(part), "{value}: {error}");
        }
    }

    #[test]
    fn a_door_on_port_80_names_no_port_and_another_door_does() {
        let at = |port| Door {
            address: Ipv4Addr::new(127, 65, 82, 71),
            suffix: "argus".to_owned(),
            port,
        };
        assert_eq!(at(80).host("rotp"), "rotp.argus");
        assert_eq!(at(80).url("rotp", "/"), "http://rotp.argus/");
        assert_eq!(at(47615).host("rotp"), "rotp.argus:47615");
        assert_eq!(at(47615).bare(), "argus:47615");
    }
}
