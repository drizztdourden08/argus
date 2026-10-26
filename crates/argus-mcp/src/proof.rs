//! The `attach_proof` tool: a picture, from a path or base64, onto a test or a run.

use std::path::PathBuf;

use argus_core::js;
use base64::Engine;
use base64::alphabet::STANDARD;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use serde_json::{Map, Value};

use crate::args::arg_str;
use crate::backend::Image;
use crate::call::Call;
use crate::error::ToolError;
use crate::files::read_allowed;
use crate::image::sniff;
use crate::json::pick;
use crate::lists::absolute;
use crate::project::need;
use crate::runs_view::RUNS_DOWN;

const BASE64_CAP: usize = 8 * 1024 * 1024;

fn lenient() -> GeneralPurpose {
    GeneralPurpose::new(
        &STANDARD,
        GeneralPurposeConfig::new()
            .with_decode_padding_mode(DecodePaddingMode::Indifferent)
            .with_decode_allow_trailing_bits(true),
    )
}

fn is_base64_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=') || c.is_whitespace()
}

fn decode_base64(data: &str, max: u64) -> Result<Vec<u8>, ToolError> {
    if !data.chars().all(is_base64_char) {
        return Err(ToolError::bad_request("base64 is not valid base64"));
    }
    if js::len(data) > BASE64_CAP {
        return Err(ToolError::new(
            "too_large",
            "base64 attachments are limited to 6 MB decoded; use path for larger files",
        ));
    }
    let clean: String = data.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = lenient()
        .decode(clean.trim_end_matches('='))
        .unwrap_or_default();
    if bytes.len() as u64 > max {
        return Err(ToolError::new(
            "too_large",
            format!("image is {} bytes, the limit is {max}", bytes.len()),
        ));
    }
    Ok(bytes)
}

fn roots(call: &Call, extra: &[String]) -> Vec<PathBuf> {
    let mut all = call.backend.file_roots();
    all.extend(extra.iter().map(PathBuf::from));
    all
}

fn read_image(call: &Call) -> Result<Image, ToolError> {
    let attach = call.backend.attach_config();
    let bytes = match arg_str(call.args, "path") {
        Some(path) => read_allowed(path, attach.max_bytes, &roots(call, &attach.extra_roots))?,
        None => decode_base64(
            arg_str(call.args, "base64").unwrap_or_default(),
            attach.max_bytes,
        )?,
    };
    let mime = sniff(&bytes).ok_or_else(|| {
        ToolError::new(
            "unsupported_media",
            "the file is not a png, jpeg, gif or webp image",
        )
    })?;
    match arg_str(call.args, "mime") {
        Some(said) if said != mime => Err(ToolError::bad_request(format!(
            "mime says {said} but the bytes are {mime}"
        ))),
        _ => Ok(Image { bytes, mime }),
    }
}

fn check_exclusive(call: &Call) -> Result<(), ToolError> {
    let has = |k: &str| arg_str(call.args, k).is_some();
    if has("test") == has("run") {
        return Err(ToolError::bad_request("give exactly one of test or run"));
    }
    if has("path") == has("base64") {
        return Err(ToolError::bad_request("give exactly one of path or base64"));
    }
    Ok(())
}

fn answer(out: &Value, keys: &[&str], base: &str) -> Value {
    let mut result = Map::new();
    pick(&mut result, out, keys);
    if let Some(url) = out.get("url") {
        result.insert("url".into(), absolute(url, base));
    }
    Value::Object(result)
}

/// `attach_proof`.
pub async fn attach_proof(call: &Call<'_>) -> Result<Value, ToolError> {
    check_exclusive(call)?;
    let image = read_image(call)?;
    let key = call.key();
    let caption = arg_str(call.args, "caption").unwrap_or_default();
    if let Some(test) = arg_str(call.args, "test") {
        need(
            call.backend.doc(key, "tests", test),
            &format!("test {test} in {key}"),
        )?;
        let out = call
            .backend
            .attach_proof(key, test, &image, caption, &call.via)
            .await?;
        return Ok(answer(&out, &["n"], &call.base));
    }
    let run = arg_str(call.args, "run").unwrap_or_default();
    if !call.backend.runs_up() {
        return Err(ToolError::unavailable(RUNS_DOWN));
    }
    let out = call
        .backend
        .run_attach(key, run, &image, caption, &call.via)
        .await?;
    Ok(answer(&out, &["id"], &call.base))
}
