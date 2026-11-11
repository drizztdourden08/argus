//! The real [`SidebarStore`]: the Desktop app's `Local Storage\leveldb` folder. A read works on
//! a copy of the folder so Claude may be running; a write goes to the live folder and is refused
//! while the app holds its `LOCK`.

use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_leveldb::{DB, Options};
use serde_json::Value;

use super::SidebarKeys;
use crate::ports::SidebarStore;

/// The prefix of every key the claude.ai origin owns in the store.
pub const KEY_PREFIX: &[u8] = b"_https://claude.ai\x00\x01";
/// The three key names, in the order of [`SidebarKeys::encodings`].
pub const KEY_NAMES: [&str; 3] = [
    "LSS-persisted.dframe-code-sections",
    "LSS-persisted.dframe-group-scopes",
    "dframe-store",
];

const UTF16: u8 = 0;
const LATIN1: u8 = 1;

/// Splits a stored value into its encoding byte and the JSON it holds.
///
/// # Errors
/// On an empty value, an unknown encoding byte, bad UTF-16 or bad JSON.
pub fn decode_value(raw: &[u8]) -> Result<(u8, Value), String> {
    let (&enc, body) = raw.split_first().ok_or("empty stored value")?;
    let text = match enc {
        UTF16 => {
            if body.len() % 2 != 0 {
                return Err("UTF-16 value has an odd length".into());
            }
            let units: Vec<u16> = body
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16(&units).map_err(|e| format!("bad UTF-16 value: {e}"))?
        }
        LATIN1 => body.iter().map(|&b| char::from(b)).collect(),
        other => return Err(format!("unknown value encoding byte {other}")),
    };
    let json = serde_json::from_str(&text).map_err(|e| format!("bad JSON in value: {e}"))?;
    Ok((enc, json))
}

/// Writes JSON as a stored value with the given encoding byte. Under Latin-1 a character
/// above U+00FF is written as a JSON `\u` escape, so any text fits.
///
/// # Errors
/// On an unknown encoding byte.
pub fn encode_value(enc: u8, json: &Value) -> Result<Vec<u8>, String> {
    let text = json.to_string();
    let mut out = vec![enc];
    match enc {
        UTF16 => {
            for unit in text.encode_utf16() {
                out.extend_from_slice(&unit.to_le_bytes());
            }
        }
        LATIN1 => {
            for c in text.chars() {
                if let Ok(b) = u8::try_from(u32::from(c)) {
                    out.push(b);
                } else {
                    let mut buf = [0u16; 2];
                    for unit in c.encode_utf16(&mut buf) {
                        out.extend_from_slice(format!("\\u{unit:04x}").as_bytes());
                    }
                }
            }
        }
        other => return Err(format!("unknown value encoding byte {other}")),
    }
    Ok(out)
}

/// Sets the wrapper's `timestamp` to `now_ms` when it has one.
pub fn bump_timestamp(wrapper: &mut Value, now_ms: i64) {
    if let Some(slot) = wrapper.get_mut("timestamp") {
        *slot = Value::from(now_ms);
    }
}

fn full_key(name: &str) -> Vec<u8> {
    let mut k = KEY_PREFIX.to_vec();
    k.extend_from_slice(name.as_bytes());
    k
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

fn open_db(dir: &Path) -> Result<DB, String> {
    let mut opt = Options::default();
    opt.create_if_missing = false;
    DB::open(dir, opt).map_err(|e| format!("cannot open the sidebar store {}: {e}", dir.display()))
}

fn load_keys(db: &mut DB) -> Result<SidebarKeys, String> {
    let mut parts: Vec<(u8, Value)> = Vec::with_capacity(3);
    for name in KEY_NAMES {
        let raw = db
            .get(&full_key(name))
            .ok_or_else(|| format!("the sidebar store has no `{name}` key"))?;
        parts.push(decode_value(&raw).map_err(|e| format!("{name}: {e}"))?);
    }
    let (e2, dframe_store) = parts.remove(2);
    let (e1, group_scopes) = parts.remove(1);
    let (e0, code_sections) = parts.remove(0);
    Ok(SidebarKeys {
        code_sections,
        group_scopes,
        dframe_store,
        encodings: [e0, e1, e2],
    })
}

/// A temporary copy of the live folder, removed when dropped.
struct TempCopy(PathBuf);

impl TempCopy {
    fn of(live: &Path) -> Result<Self, String> {
        let dir = std::env::temp_dir().join(format!("argus-sidebar-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).map_err(|e| format!("cannot make a temp folder: {e}"))?;
        let copy = Self(dir);
        copy.fill(live)?;
        Ok(copy)
    }

    fn fill(&self, live: &Path) -> Result<(), String> {
        let entries = fs::read_dir(live)
            .map_err(|e| format!("cannot read the sidebar store {}: {e}", live.display()))?;
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name == "LOCK" || !entry.path().is_file() {
                continue;
            }
            fs::copy(entry.path(), self.0.join(&name))
                .map_err(|e| format!("cannot copy {}: {e}", name.to_string_lossy()))?;
        }
        Ok(())
    }
}

impl Drop for TempCopy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(windows)]
fn open_exclusive(lock: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(lock)
}

#[cfg(not(windows))]
fn open_exclusive(lock: &Path) -> std::io::Result<File> {
    OpenOptions::new().read(true).write(true).open(lock)
}

/// Refuses when another process holds the folder's `LOCK`: an exclusive open of it then fails.
pub(crate) fn ensure_unlocked(dir: &Path) -> Result<(), String> {
    match open_exclusive(&dir.join("LOCK")) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "Claude Desktop holds the sidebar store ({e}); close it and try again"
        )),
    }
}

/// The sidebar store of one Desktop app data folder.
pub struct LevelDbSidebar {
    dir: PathBuf,
}

impl LevelDbSidebar {
    /// The store under `<app data>\Local Storage\leveldb`.
    #[must_use]
    pub fn new(app_data: &Path) -> Self {
        Self::at(app_data.join("Local Storage").join("leveldb"))
    }

    /// The store in the LevelDB folder `dir` itself.
    #[must_use]
    pub fn at(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The three keys, read from a copy of the folder.
    ///
    /// # Errors
    /// When the folder cannot be copied or opened, or a key is missing or unreadable.
    pub fn read_keys(&self) -> Result<SidebarKeys, String> {
        let copy = TempCopy::of(&self.dir)?;
        let mut db = open_db(&copy.0)?;
        let keys = load_keys(&mut db)?;
        db.close()
            .map_err(|e| format!("cannot close the copy: {e}"))?;
        Ok(keys)
    }

    /// Writes the three keys to the live folder. Their wrapper timestamps become now and each
    /// value keeps its encoding byte.
    ///
    /// # Errors
    /// When Claude holds the store's lock, or on any read or write failure.
    pub fn write_keys(&self, keys: &SidebarKeys) -> Result<(), String> {
        ensure_unlocked(&self.dir)?;
        let mut stamped = keys.clone();
        let now = now_ms();
        bump_timestamp(&mut stamped.code_sections, now);
        bump_timestamp(&mut stamped.group_scopes, now);
        let values = [
            &stamped.code_sections,
            &stamped.group_scopes,
            &stamped.dframe_store,
        ];
        let mut db = open_db(&self.dir)?;
        for ((name, value), enc) in KEY_NAMES.iter().zip(values).zip(stamped.encodings) {
            let raw = encode_value(enc, value)?;
            db.put(&full_key(name), &raw)
                .map_err(|e| format!("cannot write `{name}`: {e}"))?;
        }
        db.flush()
            .map_err(|e| format!("cannot flush the store: {e}"))?;
        db.close()
            .map_err(|e| format!("cannot close the store: {e}"))
    }
}

impl SidebarStore for LevelDbSidebar {
    fn read(&self) -> Result<Value, String> {
        self.read_keys().map(|k| k.to_value())
    }

    fn write(&self, keys: &Value) -> Result<(), String> {
        self.write_keys(&SidebarKeys::from_value(keys)?)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn utf16_round_trip() {
        let v = json!({"name": "Café ☕ 😀"});
        let raw = encode_value(UTF16, &v).unwrap();
        assert_eq!(raw[0], UTF16);
        assert_eq!(decode_value(&raw).unwrap(), (UTF16, v));
    }

    #[test]
    fn latin1_round_trip() {
        let v = json!({"name": "Café"});
        let raw = encode_value(LATIN1, &v).unwrap();
        assert_eq!(raw[0], LATIN1);
        assert!(raw.contains(&0xE9));
        assert_eq!(decode_value(&raw).unwrap(), (LATIN1, v));
    }

    #[test]
    fn latin1_escapes_wide_characters() {
        let v = json!({"name": "a ☕ 😀"});
        let raw = encode_value(LATIN1, &v).unwrap();
        assert!(raw[1..].iter().all(u8::is_ascii));
        assert_eq!(decode_value(&raw).unwrap(), (LATIN1, v));
    }

    #[test]
    fn bad_values_are_errors() {
        assert!(decode_value(&[]).is_err());
        assert!(decode_value(&[7, b'{', b'}']).is_err());
        assert!(decode_value(&[UTF16, 1]).is_err());
        assert!(decode_value(&[LATIN1, b'{']).is_err());
        assert!(encode_value(9, &json!({})).is_err());
    }

    #[test]
    fn timestamp_is_bumped_only_when_present() {
        let mut with = json!({"value": {}, "tabId": "", "timestamp": 1});
        bump_timestamp(&mut with, 99);
        assert_eq!(with["timestamp"], 99);
        let mut without = json!({"state": {}});
        bump_timestamp(&mut without, 99);
        assert!(without.get("timestamp").is_none());
    }

    #[test]
    fn keys_round_trip_through_the_port_shape() {
        let keys = SidebarKeys {
            code_sections: json!({"value": {}}),
            group_scopes: json!({"value": {}}),
            dframe_store: json!({"state": {}}),
            encodings: [1, 0, 1],
        };
        assert_eq!(SidebarKeys::from_value(&keys.to_value()).unwrap(), keys);
    }

    #[test]
    fn a_missing_value_is_an_error() {
        assert!(SidebarKeys::from_value(&json!({"codeSections": {}})).is_err());
    }
}
