//! The launcher script of `argus autostart`: one line of `VBScript` that starts the tray hidden.
//! Windows Script Host reads a script without a byte order mark as the ANSI code page, so a path
//! with other characters is written as UTF-16 with a mark.

const BOM: [u8; 2] = [0xff, 0xfe];

fn quoted(path: &str) -> String {
    format!("\"\"{}\"\"", path.replace('"', "\"\""))
}

/// The line of DESIGN 7.5. Window style 0 is hidden; inside a `VBScript` string a quote is doubled.
/// It runs `argus tray start`, which brings the tray and the server up unless the owner quit
/// the dashboard during this boot.
#[must_use]
pub fn content(exe: &str) -> String {
    format!(
        "CreateObject(\"WScript.Shell\").Run \"{} tray start\", 0, False\r\n",
        quoted(exe)
    )
}

/// The bytes to write: plain for ASCII text, UTF-16 little endian with a mark otherwise.
#[must_use]
pub fn encode(text: &str) -> Vec<u8> {
    if text.is_ascii() {
        return text.as_bytes().to_vec();
    }
    let mut out = BOM.to_vec();
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

/// The text of a launcher file, whichever way it was written.
#[must_use]
pub fn decode(bytes: &[u8]) -> String {
    match bytes.strip_prefix(&BOM) {
        Some(wide) => {
            let units: Vec<u16> = wide
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&units)
        }
        None => String::from_utf8_lossy(bytes).into_owned(),
    }
}
