//! Attaching an image to a run, and listing a run's attachments.

use crate::attach::{AttachmentRow, SaveRequest, save_attachment, url_of};
use crate::error::RunError;
use crate::host::ProcessHost;
use crate::run_service::RunService;

/// An attachment as the API shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentInfo {
    pub id: String,
    pub url: String,
    pub caption: String,
    pub mime: String,
    pub bytes: u64,
    pub n: u64,
    pub by: String,
    pub at: String,
}

impl AttachmentInfo {
    fn of(scope: &str, row: AttachmentRow) -> Self {
        Self {
            url: url_of(scope, &row.id),
            id: row.id,
            caption: row.caption.unwrap_or_default(),
            mime: row.mime,
            bytes: row.bytes,
            n: row.n,
            by: row.created_by,
            at: row.created_at,
        }
    }
}

impl<H: ProcessHost> RunService<H> {
    /// Saves `bytes` (an image; the magic number decides, `mime` is only checked against it)
    /// as the next attachment of the run.
    pub fn attach(
        &self,
        key: &str,
        rid: &str,
        bytes: &[u8],
        mime: &str,
        caption: &str,
        by: &str,
    ) -> Result<AttachmentInfo, RunError> {
        self.known_run(key, rid)?;
        let n =
            u64::try_from(self.inner.deps.store.attachments_of(key, rid).len()).unwrap_or(0) + 1;
        let at = self.now_iso();
        let config = self.config();
        let row = save_attachment(&SaveRequest {
            dir: &config.attachments_dir,
            scope: key,
            bytes,
            mime,
            max_bytes: config.attach_max_bytes,
            n,
            caption,
            by,
            at: &at,
        })?;
        self.inner.deps.store.add_attachment(key, rid, row.clone());
        Ok(AttachmentInfo::of(key, row))
    }

    pub fn attachments(&self, key: &str, rid: &str) -> Result<Vec<AttachmentInfo>, RunError> {
        self.known_run(key, rid)?;
        Ok(self
            .inner
            .deps
            .store
            .attachments_of(key, rid)
            .into_iter()
            .map(|row| AttachmentInfo::of(key, row))
            .collect())
    }
}
