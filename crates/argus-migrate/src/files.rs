//! Step 7: the home files that carry a key: the listen stamps `dashboard/listen-<key>-<thread>.json`
//! (renamed), `remote.json` `syncs` and `dashboard/config.json` `projectRoots` (re-keyed).

use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::jsonio;
use crate::map::IdMap;
use crate::rewrite::rekey;

const PREFIX: &str = "listen-";
const SUFFIX: &str = ".json";

/// The new name of a stamp: the longest key of `map` that starts its `<key>-<thread>` part.
pub fn renamed_stamp(name: &str, map: &IdMap) -> Option<String> {
    let rest = name.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    keys.into_iter().find_map(|key| {
        let thread = rest.strip_prefix(key.as_str())?.strip_prefix('-')?;
        (!thread.is_empty()).then(|| format!("{PREFIX}{}-{thread}{SUFFIX}", map[key]))
    })
}

fn rename_stamps(dir: &Path, map: &IdMap) -> Result<()> {
    let Ok(read) = fs::read_dir(dir) else {
        return Ok(());
    };
    let names: Vec<String> = read
        .filter_map(std::result::Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    for name in names {
        let new = renamed_stamp(&name, map).filter(|n| !dir.join(n).exists());
        if let Some(new) = new {
            fs::rename(dir.join(&name), dir.join(&new))?;
        }
    }
    Ok(())
}

fn rekey_file(file: &Path, field: &str, map: &IdMap) -> Result<()> {
    if !file.is_file() {
        return Ok(());
    }
    let mut value = jsonio::read(file)?;
    let before = value.clone();
    if let Some(table) = value.get_mut(field) {
        rekey(table, map);
    }
    if value != before {
        jsonio::write(file, &value)?;
    }
    Ok(())
}

/// Rewrites the home files of step 7.
pub fn rewrite(home: &Path, map: &IdMap) -> Result<()> {
    let dashboard = home.join("dashboard");
    rename_stamps(&dashboard, map)?;
    rekey_file(&home.join("remote.json"), "syncs", map)?;
    rekey_file(&dashboard.join("config.json"), "projectRoots", map)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! The stamp names: the longest key wins, the thread keeps its name.
    use super::renamed_stamp;
    use crate::map::IdMap;

    fn map() -> IdMap {
        IdMap::from([
            ("rotp".to_owned(), "b1".to_owned()),
            ("rotp-web".to_owned(), "b2".to_owned()),
        ])
    }

    #[test]
    fn the_longest_key_names_the_stamp() {
        let got = renamed_stamp("listen-rotp-web-coordinator.json", &map());
        assert_eq!(got.as_deref(), Some("listen-b2-coordinator.json"));
        let got = renamed_stamp("listen-rotp-coordinator.json", &map());
        assert_eq!(got.as_deref(), Some("listen-b1-coordinator.json"));
    }

    #[test]
    fn other_names_stay() {
        assert_eq!(renamed_stamp("listen-other-lead.json", &map()), None);
        assert_eq!(renamed_stamp("listen-rotp-.json", &map()), None);
        assert_eq!(renamed_stamp("config.json", &map()), None);
    }
}
