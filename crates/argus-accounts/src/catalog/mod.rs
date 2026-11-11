//! The chat catalog of the Desktop app and the sync of it between accounts: reading the files
//! ([`read`]), the pure merge rules ([`merge`]), the plan ([`plan`]) and applying it ([`apply`]).

pub mod apply;
pub mod merge;
pub mod plan;
pub mod read;

pub use apply::{StepStatus, apply, preflight};
pub use plan::{Change, ChangeKind, Conflict, Plan, PlanSummary, TombstoneSkip, build_plan};
pub use read::{FsCatalog, FsTranscripts};

/// Chat folders and transcripts in a temp dir, for the tests of the sync.
#[cfg(test)]
pub(crate) mod fixtures {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::{Value, json};
    use tempfile::TempDir;

    use super::merge::json_bytes;
    use super::read::{FsCatalog, FsTranscripts};
    use crate::types::Scope;

    pub(crate) fn session_id(n: u32) -> String {
        format!("local_aaaaaaaa-0000-0000-0000-{n:012}")
    }

    pub(crate) fn cli_id(n: u32) -> String {
        format!("cccccccc-0000-0000-0000-{n:012}")
    }

    /// A chat record that ran, with the given last activity.
    pub(crate) fn record(n: u32, activity: i64) -> Value {
        json!({
            "sessionId": session_id(n),
            "cliSessionId": cli_id(n),
            "cwd": "C:/work",
            "title": format!("chat {n}"),
            "lastActivityAt": activity,
        })
    }

    pub(crate) fn scope(account: &str, org: &str) -> Scope {
        Scope {
            account: account.to_owned(),
            org: org.to_owned(),
        }
    }

    /// The scopes `a1/o1`, `a2/o2` and `a3/o3`; none exists until it is written to.
    pub(crate) fn three_scopes() -> (Scope, Scope, Scope) {
        (scope("a1", "o1"), scope("a2", "o2"), scope("a3", "o3"))
    }

    pub(crate) struct Fixture {
        dir: TempDir,
    }

    impl Fixture {
        pub(crate) fn new() -> Self {
            Fixture {
                dir: TempDir::new().expect("temp dir"),
            }
        }

        /// The folder that plays the Desktop app's data folder.
        pub(crate) fn root(&self) -> &Path {
            self.dir.path()
        }

        fn sessions(&self) -> PathBuf {
            self.dir.path().join("claude-code-sessions")
        }

        fn projects(&self) -> PathBuf {
            self.dir.path().join("projects")
        }

        fn folder(&self, scope: &Scope) -> PathBuf {
            self.sessions().join(&scope.account).join(&scope.org)
        }

        /// Makes the scope exist with no chats.
        pub(crate) fn ensure(&self, scope: &Scope) {
            fs::create_dir_all(self.folder(scope)).expect("scope folder");
        }

        pub(crate) fn put(&self, scope: &Scope, n: u32, record: &Value) {
            self.ensure(scope);
            let path = self.folder(scope).join(format!("{}.json", session_id(n)));
            fs::write(path, json_bytes(record)).expect("write record");
        }

        pub(crate) fn tombstone(&self, scope: &Scope, n: u32) {
            self.ensure(scope);
            let uuid = session_id(n).replace("local_", "");
            fs::write(self.folder(scope).join(format!("deleted_{uuid}")), b"").expect("tombstone");
        }

        pub(crate) fn write_index(&self, scope: &Scope, ids: &[String]) {
            self.ensure(scope);
            let index = json!({ "v": 1, "archived": ids });
            fs::write(
                self.folder(scope).join("archived-sessions.idx"),
                json_bytes(&index),
            )
            .expect("index");
        }

        /// A valid transcript of the chat with `cli_id(n)`, in the default project.
        pub(crate) fn transcript(&self, n: u32) {
            self.transcript_in(n, "project-a");
        }

        pub(crate) fn transcript_in(&self, n: u32, project: &str) {
            let id = cli_id(n);
            let line = json!({ "type": "user", "message": { "content": "hi" },
                "sessionId": id, "uuid": cli_id(n + 1000) });
            self.write_transcript(&id, project, &format!("{line}\n"));
        }

        /// A transcript file that is not a readable chat.
        pub(crate) fn bad_transcript(&self, n: u32) {
            self.write_transcript(&cli_id(n), "project-a", "this is not json\n");
        }

        fn write_transcript(&self, id: &str, project: &str, text: &str) {
            let dir = self.projects().join(project);
            fs::create_dir_all(&dir).expect("project folder");
            fs::write(dir.join(format!("{id}.jsonl")), text).expect("transcript");
        }

        pub(crate) fn catalog(&self) -> FsCatalog {
            FsCatalog::new(self.sessions())
        }

        pub(crate) fn transcripts(&self) -> FsTranscripts {
            FsTranscripts::scan(&self.projects()).expect("scan transcripts")
        }
    }
}
