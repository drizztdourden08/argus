//! UTF-8 decoding of a byte stream read in arbitrary chunks: a character split across two
//! reads is held back until its rest arrives; bytes that can never be valid become U+FFFD.

#[derive(Debug, Default)]
pub(crate) struct Utf8Decoder {
    held: Vec<u8>,
}

impl Utf8Decoder {
    pub(crate) fn push(&mut self, bytes: &[u8]) -> String {
        self.held.extend_from_slice(bytes);
        let mut out = String::new();
        loop {
            match std::str::from_utf8(&self.held) {
                Ok(valid) => {
                    out.push_str(valid);
                    self.held.clear();
                    return out;
                }
                Err(err) => {
                    let good = err.valid_up_to();
                    out.push_str(&String::from_utf8_lossy(&self.held[..good]));
                    match err.error_len() {
                        None => {
                            self.held.drain(..good);
                            return out;
                        }
                        Some(bad) => {
                            out.push(char::REPLACEMENT_CHARACTER);
                            self.held.drain(..good + bad);
                        }
                    }
                }
            }
        }
    }

    /// The end of the stream: an incomplete character left over becomes U+FFFD.
    pub(crate) fn finish(&mut self) -> String {
        let rest = String::from_utf8_lossy(&self.held).into_owned();
        self.held.clear();
        rest
    }
}
