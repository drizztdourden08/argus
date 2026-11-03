//! The name a folder gets just before it is deleted: `<name>.argus-deleting-<ms>`.

/// The mark between the old name and the time stamp.
pub const DELETING_MARK: &str = ".argus-deleting-";

fn mark_at(name: &str) -> Option<usize> {
    let at = name.rfind(DELETING_MARK)?;
    let stamp = &name[at + DELETING_MARK.len()..];
    (stamp.len() >= 8 && stamp.bytes().all(|b| b.is_ascii_digit())).then_some(at)
}

/// Is this a folder Argus renamed before deleting it (a crash can leave one behind)?
pub fn is_deleting(name: &str) -> bool {
    mark_at(name).is_some()
}

/// The name a deleting folder had before the rename.
pub fn original_name(name: &str) -> String {
    mark_at(name).map_or_else(|| name.to_string(), |at| name[..at].to_string())
}

/// The renamed name of a path for a time stamp.
pub fn deleting_path(path: &str, stamp: i64) -> String {
    format!("{path}{DELETING_MARK}{stamp}")
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_time_stamp_of_eight_digits_or_more_marks_a_deleting_folder() {
        assert!(is_deleting("agent-ab12.argus-deleting-1730000000000"));
        assert!(!is_deleting("agent-ab12.argus-deleting-abc"));
        assert!(!is_deleting("x.argus-deleting-12"));
        assert_eq!(
            original_name("agent-ab12.argus-deleting-1730000000000"),
            "agent-ab12"
        );
        assert_eq!(original_name("plain"), "plain");
        assert_eq!(deleting_path("X:/a/b", 17), "X:/a/b.argus-deleting-17");
    }
}
