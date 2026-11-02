//! Tolerant board fields: a board is written by many tools, so a field of an unexpected type reads as absent.

use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// A string field; anything else, and an empty string, reads as absent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Text(Option<String>);

impl Text {
    /// The text when it is set and not empty.
    #[must_use]
    pub fn some(&self) -> Option<&str> {
        self.0.as_deref().filter(|s| !s.is_empty())
    }

    /// The text, or an empty string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.some().unwrap_or("")
    }

    /// Is the text one of these?
    #[must_use]
    pub fn is_any(&self, words: &[&str]) -> bool {
        self.some().is_some_and(|s| words.contains(&s))
    }

    /// Is the text exactly this?
    #[must_use]
    pub fn is(&self, word: &str) -> bool {
        self.some() == Some(word)
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self(Some(value.to_owned()))
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self(match Value::deserialize(d)? {
            Value::String(s) => Some(s),
            _ => None,
        }))
    }
}

/// A flag field read with JavaScript truthiness.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flag(bool);

impl Flag {
    /// Is the flag on?
    #[must_use]
    pub fn on(self) -> bool {
        self.0
    }
}

impl<'de> Deserialize<'de> for Flag {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self(match Value::deserialize(d)? {
            Value::Null => false,
            Value::Bool(b) => b,
            Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
            Value::String(s) => !s.is_empty(),
            Value::Array(_) | Value::Object(_) => true,
        }))
    }
}

/// A number field; numeric strings count, anything else reads as absent.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Num(Option<f64>);

impl Num {
    /// The number when there is one.
    #[must_use]
    pub fn get(self) -> Option<f64> {
        self.0.filter(|n| n.is_finite())
    }
}

impl<'de> Deserialize<'de> for Num {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self(match Value::deserialize(d)? {
            Value::Number(n) => n.as_f64(),
            Value::String(s) if s.trim().is_empty() => Some(0.0),
            Value::String(s) => s.trim().parse().ok(),
            Value::Null | Value::Bool(false) => Some(0.0),
            Value::Bool(true) => Some(1.0),
            _ => None,
        }))
    }
}

/// A map of records; entries that are not records are dropped.
///
/// # Errors
/// Never; the signature is the one serde expects.
pub fn map<'de, D, T>(d: D) -> Result<IndexMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Ok(match Value::deserialize(d)? {
        Value::Object(fields) => fields
            .into_iter()
            .filter_map(|(key, value)| {
                serde_json::from_value(value)
                    .ok()
                    .map(|record| (key, record))
            })
            .collect(),
        _ => IndexMap::new(),
    })
}

/// A list of records; entries that are not records are dropped.
///
/// # Errors
/// Never; the signature is the one serde expects.
pub fn list<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Ok(match Value::deserialize(d)? {
        Value::Array(values) => values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
        _ => Vec::new(),
    })
}

/// A mirror field: an object is one record, an array is many (entries that are not records are
/// dropped), anything else is none.
///
/// # Errors
/// Never; the signature is the one serde expects.
pub fn one_or_many<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Ok(match Value::deserialize(d)? {
        value @ Value::Object(_) => serde_json::from_value(value).into_iter().collect(),
        Value::Array(values) => values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
        _ => Vec::new(),
    })
}

/// A record that is an object, or absent.
///
/// # Errors
/// Never; the signature is the one serde expects.
pub fn object<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Ok(match Value::deserialize(d)? {
        value @ Value::Object(_) => serde_json::from_value(value).ok(),
        _ => None,
    })
}
