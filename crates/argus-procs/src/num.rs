//! Numeric conversions kept in one place so the lossy casts are named and bounded.

/// An integer as a float; sample values stay far below 2^53.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn to_f64(n: i64) -> f64 {
    n as f64
}

/// A non-negative float as whole milliseconds, saturating at the ends.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn to_millis(x: f64) -> u64 {
    x.max(0.0) as u64
}

/// A float rounded half up to a whole number, saturating at the ends.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn round_to_i64(x: f64) -> i64 {
    x.round() as i64
}

/// Rounds to one decimal.
#[must_use]
pub fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Bytes as whole megabytes, rounded half up.
#[must_use]
pub fn mem_mb(bytes: u64) -> i64 {
    i64::try_from(bytes.saturating_add(524_288) / 1_048_576).unwrap_or(i64::MAX)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn megabytes_round_half_up() {
        assert_eq!(mem_mb(1_048_576 * 100), 100);
        assert_eq!(mem_mb(1_048_576 + 524_288), 2);
        assert_eq!(mem_mb(524_287), 0);
    }

    #[test]
    fn one_decimal() {
        assert!((round1(33.349) - 33.3).abs() < 1e-9);
        assert_eq!(round_to_i64(2.5), 3);
        assert_eq!(to_millis(-4.0), 0);
    }
}
