//! A one pixel PNG, the proof image of the tests.

const PNG_HEX: &str = "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000154a24f5d0000000049454e44ae426082";

/// The bytes of a one pixel PNG.
///
/// # Panics
/// Never: the hex text is a constant.
#[must_use]
pub fn png() -> Vec<u8> {
    (0..PNG_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&PNG_HEX[i..i + 2], 16).expect("hex"))
        .collect()
}
