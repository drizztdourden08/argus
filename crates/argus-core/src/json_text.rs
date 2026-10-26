//! JSON text as `JSON.stringify` prints it. serde_json writes the float `1.0` as `1.0`; Node
//! writes `1`. Every JSON text a client or a stored doc sees goes through [`to_string`] or
//! [`to_vec`], so one rule covers every doc and route: a number with no fraction has none.

use std::io::{self, Write};

use serde::Serialize;
use serde_json::ser::{CompactFormatter, Formatter, Serializer};

/// Past this a JavaScript number prints with an exponent, as serde_json does.
const WHOLE_LIMIT: f64 = 1e21;

/// The compact format with whole floats written as integers.
struct NodeFormatter;

impl Formatter for NodeFormatter {
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        if value == 0.0 {
            return writer.write_all(b"0");
        }
        if value.fract() == 0.0 && value.abs() < WHOLE_LIMIT {
            return write!(writer, "{value:.0}");
        }
        CompactFormatter.write_f64(writer, value)
    }
}

/// The compact JSON bytes of a value, numbers as Node writes them.
///
/// # Errors
///
/// The value cannot be serialised (a map with a key that is not text).
pub fn to_vec<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let mut out = Vec::with_capacity(128);
    value.serialize(&mut Serializer::with_formatter(&mut out, NodeFormatter))?;
    Ok(out)
}

/// The compact JSON text of a value, numbers as Node writes them.
///
/// # Errors
///
/// The value cannot be serialised (a map with a key that is not text).
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String, serde_json::Error> {
    let bytes = to_vec(value)?;
    String::from_utf8(bytes).map_err(|e| serde::ser::Error::custom(e.to_string()))
}
