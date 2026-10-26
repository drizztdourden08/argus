//! The run attachments over the `attachments` table (`owner_kind` `run`, `owner_id` the run id).

use argus_launchers::{AttachmentRow, AttachmentStore};
use argus_store::rusqlite::params;

use super::rows::int;
use super::store::SqliteRuns;

impl AttachmentStore for SqliteRuns {
    fn attachments_of(&self, scope: &str, rid: &str) -> Vec<AttachmentRow> {
        self.db.with_conn(|c| {
            let Ok(mut statement) = c.prepare_cached(
                "SELECT id, n, path, mime, bytes, sha256, caption, created_by, created_at FROM attachments WHERE scope = ? AND owner_kind = 'run' AND owner_id = ? ORDER BY created_at, rowid",
            ) else {
                return Vec::new();
            };
            statement
                .query_map(params![scope, rid], |r| {
                    Ok(AttachmentRow {
                        id: r.get(0)?,
                        n: u64::try_from(r.get::<_, Option<i64>>(1)?.unwrap_or(0)).unwrap_or(0),
                        path: r.get(2)?,
                        mime: r.get(3)?,
                        bytes: u64::try_from(r.get::<_, i64>(4)?).unwrap_or(0),
                        sha256: r.get(5)?,
                        caption: r.get(6)?,
                        created_by: r.get(7)?,
                        created_at: r.get(8)?,
                    })
                })
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default()
        })
    }

    fn add_attachment(&self, scope: &str, rid: &str, row: AttachmentRow) {
        let done = self.db.tx(|tx| {
            tx.conn().execute(
                "INSERT INTO attachments (id, scope, owner_kind, owner_id, n, path, mime, bytes, sha256, caption, created_by, created_at) VALUES (?, ?, 'run', ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    row.id,
                    scope,
                    rid,
                    int(row.n),
                    row.path,
                    row.mime,
                    int(row.bytes),
                    row.sha256,
                    row.caption,
                    row.created_by,
                    row.created_at
                ],
            )?;
            Ok(())
        });
        if let Err(e) = done {
            self.failed("run attachment not stored", &e);
        }
    }
}
