//! Proof images: bytes from the page or MCP, or a path inside an allowed folder; the magic bytes
//! must be png, jpeg, gif or webp. The image is kept under `<COORD_HOME>/<key>/proofs/`, the test
//! is stamped, and the board exported.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use argus_board::limits::limit;
use argus_board::proofs::{Image, keep_entry_image, keep_proof_bytes};
use argus_board::store::{LockWait, board_path, stamp, with_board_async};
use argus_board::{Board, BoardError};
use argus_core::ids::{PORTFOLIO, is_key};
use argus_core::{ApiError, ErrorCode, js};
use serde_json::{Value, json};

use crate::exporter::proof_url;
use crate::failure::board_error;
use crate::state::{State, clear_marks};

/// Where a proof comes from.
#[derive(Debug, Clone)]
pub enum ProofSource {
    /// The image bytes, with the type the sender declared.
    Bytes {
        /// The bytes.
        bytes: Vec<u8>,
        /// The declared type, for example `image/png`.
        mime: Option<String>,
    },
    /// A file inside an allowed folder.
    Path(PathBuf),
}

/// A kept proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    /// Its number on the test.
    pub n: i64,
    /// Where the page sees it.
    pub url: String,
}

/// The image type of some bytes: `(mime, extension)`.
#[must_use]
pub fn sniff_image(b: &[u8]) -> Option<(&'static str, &'static str)> {
    if b.len() > 8 && b[0] == 0x89 && &b[1..4] == b"PNG" {
        Some(("image/png", ".png"))
    } else if b.len() > 3 && b[..3] == [0xff, 0xd8, 0xff] {
        Some(("image/jpeg", ".jpg"))
    } else if b.len() > 6 && &b[..4] == b"GIF8" {
        Some(("image/gif", ".gif"))
    } else if b.len() > 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        Some(("image/webp", ".webp"))
    } else {
        None
    }
}

fn fail(code: ErrorCode, m: impl Into<String>) -> ApiError {
    ApiError::new(code, m)
}

fn canonical(p: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(p).ok()
}

fn check_caption(target: &Target, caption: &str) -> Result<String, ApiError> {
    let trimmed = js::trim(caption);
    let text = if trimmed.is_empty() {
        format!("{} {}", target.id(), target.noun())
    } else {
        trimmed.to_owned()
    };
    let max = limit("caption").unwrap_or(120);
    if js::len(&text) > max {
        return Err(ApiError::bad_request(format!(
            "caption is {} chars, the limit is {max}",
            js::len(&text)
        )));
    }
    Ok(text)
}

fn check_bytes(bytes: &[u8], mime: Option<&str>, max: u64) -> Result<&'static str, ApiError> {
    if bytes.is_empty() {
        return Err(ApiError::bad_request("the image is empty"));
    }
    if bytes.len() as u64 > max {
        return Err(fail(
            ErrorCode::TooLarge,
            format!("the image is {} bytes, the limit is {max}", bytes.len()),
        ));
    }
    let (kind, ext) = sniff_image(bytes).ok_or_else(|| {
        fail(
            ErrorCode::UnsupportedMedia,
            "a proof must be a png, jpeg, gif or webp image",
        )
    })?;
    let declared = mime.map(|m| {
        m.split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_lowercase()
            .replace("image/jpg", "image/jpeg")
    });
    match declared {
        Some(d) if d != kind => Err(fail(
            ErrorCode::UnsupportedMedia,
            format!("the bytes are {kind}, not {d}"),
        )),
        _ => Ok(ext),
    }
}

impl State {
    fn allowed_roots(&self) -> Vec<PathBuf> {
        let config = self.config();
        let mut roots = vec![self.coord_home().to_path_buf(), std::env::temp_dir()];
        roots.extend(config.attach.extra_roots.iter().map(PathBuf::from));
        roots.extend(config.project_roots.values().map(PathBuf::from));
        let reg = self
            .board(PORTFOLIO)
            .map_or(Value::Null, |p| p["projects"].clone());
        roots.extend(
            reg.as_object()
                .into_iter()
                .flatten()
                .filter_map(|(_, p)| p["root"].as_str().map(PathBuf::from)),
        );
        roots.iter().filter_map(|r| canonical(r)).collect()
    }

    fn read_source(&self, src: ProofSource) -> Result<(Vec<u8>, Option<String>), ApiError> {
        match src {
            ProofSource::Bytes { bytes, mime } => Ok((bytes, mime)),
            ProofSource::Path(p) => {
                let real = canonical(&p)
                    .ok_or_else(|| ApiError::bad_request(format!("no file at {}", p.display())))?;
                if !self.allowed_roots().iter().any(|r| real.starts_with(r)) {
                    return Err(fail(
                        ErrorCode::Forbidden,
                        "that path is outside the folders a proof may come from (project roots, COORD_HOME, the temp folder, attach.extraRoots)",
                    ));
                }
                let bytes = std::fs::read(&real)
                    .map_err(|_| ApiError::bad_request(format!("no file at {}", p.display())))?;
                Ok((bytes, None))
            }
        }
    }

    /// Keeps a proof image on a test.
    pub async fn attach_proof(
        self: &Arc<Self>,
        key: &str,
        test_id: &str,
        src: ProofSource,
        caption: &str,
        by: &str,
    ) -> Result<Proof, ApiError> {
        if !numbered(test_id, &["T"]) {
            return Err(ApiError::bad_request(format!(
                "invalid test id \"{test_id}\""
            )));
        }
        self.attach(key, Target::Test(test_id), (src, caption), by)
            .await
    }

    /// Keeps a picture on a question or a decision entry. `option` is the 1-based choice it
    /// belongs to, `None` for the entry as a whole.
    pub async fn attach_entry_image(
        self: &Arc<Self>,
        key: &str,
        target: (&str, Option<i64>),
        (src, caption): (ProofSource, &str),
        by: &str,
    ) -> Result<Proof, ApiError> {
        let (entry_id, option) = target;
        if !numbered(entry_id, &["Q", "D"]) {
            return Err(ApiError::bad_request(format!(
                "invalid entry id \"{entry_id}\""
            )));
        }
        let target = Target::Entry(entry_id, option);
        self.attach(key, target, (src, caption), by).await
    }

    async fn attach(
        self: &Arc<Self>,
        key: &str,
        target: Target<'_>,
        (src, caption): (ProofSource, &str),
        by: &str,
    ) -> Result<Proof, ApiError> {
        if !is_key(key) {
            return Err(ApiError::bad_request(format!(
                "invalid project key \"{key}\""
            )));
        }
        let (bytes, mime) = self.read_source(src)?;
        let ext = check_bytes(&bytes, mime.as_deref(), self.config().attach.max_bytes)?;
        let text = check_caption(&target, caption)?;
        if !board_path(self.coord_home(), key).exists() {
            return Err(ApiError::not_found(format!(
                "no board for project \"{key}\""
            )));
        }
        let me = Arc::clone(self);
        let job = Keep {
            key,
            target,
            bytes: &bytes,
            ext,
            caption: &text,
            by,
        };
        self.opts
            .queue
            .run(key, async move { me.keep(job).await })
            .await
    }

    async fn keep(self: &Arc<Self>, job: Keep<'_>) -> Result<Proof, ApiError> {
        let home = self.coord_home().to_path_buf();
        let (saved, n) =
            with_board_async(self.coord_home(), job.key, LockWait::default(), |board| {
                keep_on(&home, board, &job)
            })
            .await
            .map_err(|e| match e {
                BoardError::Refused(m) if m.starts_with("no test") || m.starts_with("no entry") => {
                    ApiError::not_found(m)
                }
                other => board_error(other),
            })?;
        let file = saved[job.target.coll()][job.target.id()][job.target.field()]
            .as_array()
            .and_then(|ps| ps.iter().find(|p| p["n"] == json!(n)))
            .and_then(|p| p["file"].as_str())
            .unwrap_or_default()
            .to_owned();
        self.export_board(job.key, saved)?;
        Ok(Proof {
            n,
            url: proof_url(job.key, &file),
        })
    }
}

fn numbered(id: &str, prefixes: &[&str]) -> bool {
    id.split_once('-').is_some_and(|(prefix, n)| {
        prefixes.contains(&prefix) && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())
    })
}

/// What a picture is kept on.
#[derive(Clone, Copy)]
enum Target<'a> {
    /// A test, as a proof.
    Test(&'a str),
    /// A question or decision entry, tied to a 1-based option or to none.
    Entry(&'a str, Option<i64>),
}

impl Target<'_> {
    fn id(&self) -> &str {
        match self {
            Target::Test(id) | Target::Entry(id, _) => id,
        }
    }

    fn noun(&self) -> &'static str {
        match self {
            Target::Test(_) => "proof",
            Target::Entry(..) => "image",
        }
    }

    fn coll(&self) -> &'static str {
        match self {
            Target::Test(_) => "tests",
            Target::Entry(..) => "entries",
        }
    }

    fn field(&self) -> &'static str {
        match self {
            Target::Test(_) => "proofs",
            Target::Entry(..) => "images",
        }
    }
}

struct Keep<'a> {
    key: &'a str,
    target: Target<'a>,
    bytes: &'a [u8],
    ext: &'a str,
    caption: &'a str,
    by: &'a str,
}

fn keep_on(home: &Path, board: &mut Board, job: &Keep) -> Result<(Board, i64), BoardError> {
    let id = job.target.id();
    let what = if matches!(job.target, Target::Test(_)) {
        "test"
    } else {
        "entry"
    };
    let missing = || BoardError::Refused(format!("no {what} {id} on {}", job.key));
    let doc = board
        .get_mut(job.target.coll())
        .and_then(|t| t.get_mut(id))
        .ok_or_else(missing)?;
    if js::truthy(&doc["deletedAt"]) {
        return Err(missing());
    }
    if matches!(job.target, Target::Entry(..))
        && !matches!(doc["kind"].as_str(), Some("question" | "decision"))
    {
        return Err(BoardError::Refused(format!(
            "{id} is not a question or a decision: only those carry images."
        )));
    }
    let image = Image {
        bytes: job.bytes,
        ext: job.ext,
        caption: job.caption,
    };
    let n = match job.target {
        Target::Test(_) => {
            keep_proof_bytes(home, job.key, doc, image.bytes, image.ext, image.caption)
        }
        Target::Entry(_, option) => keep_entry_image(home, job.key, doc, (image, option)),
    }?;
    stamp(
        board,
        job.target.coll(),
        id,
        if job.by.is_empty() { "human" } else { job.by },
    )?;
    clear_marks(board);
    Ok((board.clone(), n))
}
