//! Keys, labels, titles and tags of the senders of notifications.
// test-tag: critical

use std::fmt::Write;

use sha1::{Digest, Sha1};

const KEY_PART_MAX: usize = 60;
const TAG_HEX: usize = 12;

/// A project or thread name: trimmed, at most 60 characters, none when empty.
#[must_use]
pub fn key_part(value: Option<&str>) -> Option<String> {
    let part: String = value?.trim().chars().take(KEY_PART_MAX).collect();
    (!part.is_empty()).then_some(part)
}

/// `project/thread`, the key the limits count by.
#[must_use]
pub fn limit_key(project: Option<&str>, thread: Option<&str>) -> String {
    format!("{}/{}", project.unwrap_or(""), thread.unwrap_or(""))
}

/// How a sender reads in a sentence: `project/thread`, else the thread, else the project.
#[must_use]
pub fn label_of(project: Option<&str>, thread: Option<&str>) -> String {
    match (project, thread) {
        (Some(project), Some(thread)) => format!("{project}/{thread}"),
        (project, thread) => thread.or(project).unwrap_or("Argus").to_owned(),
    }
}

/// The default title: `<Project name>: <thread>`, else the project name, else the thread.
#[must_use]
pub fn title_from(project_name: Option<&str>, thread: Option<&str>) -> String {
    match (project_name, thread) {
        (Some(name), Some(thread)) => format!("{name}: {thread}"),
        (name, thread) => name.or(thread).unwrap_or("Argus").to_owned(),
    }
}

/// The replace tag of a sender: 12 hex characters of the sha1 of `project/thread`, so a newer
/// toast of the same thread replaces its older one.
#[must_use]
pub fn tag_of(project: Option<&str>, thread: Option<&str>) -> String {
    let digest = Sha1::digest(limit_key(project, thread).as_bytes());
    digest
        .iter()
        .take(TAG_HEX / 2)
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests {
    use super::{key_part, label_of, tag_of, title_from};

    #[test]
    fn the_tag_is_twelve_hex_characters_of_the_sha1() {
        assert_eq!(tag_of(Some("a"), Some("b")), "3ec69c85a4ff");
        assert_eq!(tag_of(None, None).len(), 12);
    }

    #[test]
    fn the_label_and_the_title_fall_back_in_order() {
        assert_eq!(label_of(Some("rotp"), Some("lead")), "rotp/lead");
        assert_eq!(label_of(Some("rotp"), None), "rotp");
        assert_eq!(label_of(None, None), "Argus");
        assert_eq!(title_from(Some("Rotp"), Some("lead")), "Rotp: lead");
        assert_eq!(title_from(None, Some("lead")), "lead");
        assert_eq!(title_from(None, None), "Argus");
    }

    #[test]
    fn a_key_part_is_trimmed_cut_and_never_empty() {
        assert_eq!(key_part(Some("  lead ")), Some("lead".into()));
        assert_eq!(key_part(Some("   ")), None);
        assert_eq!(key_part(Some(&"x".repeat(80))).map(|s| s.len()), Some(60));
    }
}
