//! The `Range` header of a feed request, for differential downloads. One byte range is served
//! as a 206; several ranges, a unit other than bytes and a malformed header are ignored and the
//! whole file is sent (the standard allows it); a range outside the file is a 416.

/// What to send.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    /// The whole file, 200.
    Whole,
    /// These bytes, first and last inclusive, 206.
    Part(u64, u64),
    /// Nothing of the file is in the range, 416.
    Unsatisfiable,
}

/// A side of `first-last`: missing, or a number; `Err` when it is neither.
fn bound(text: &str) -> Result<Option<u64>, ()> {
    if text.is_empty() {
        Ok(None)
    } else {
        text.parse().map(Some).map_err(|_| ())
    }
}

/// The span a `Range` header asks of a file of `len` bytes.
#[must_use]
pub fn span(header: Option<&str>, len: u64) -> Span {
    let Some(spec) = header.and_then(|h| h.trim().strip_prefix("bytes=")) else {
        return Span::Whole;
    };
    let Some((from, to)) = spec.split_once('-').filter(|_| !spec.contains(',')) else {
        return Span::Whole;
    };
    let (Ok(from), Ok(to)) = (bound(from.trim()), bound(to.trim())) else {
        return Span::Whole;
    };
    if len == 0 {
        return Span::Whole;
    }
    match (from, to) {
        (Some(a), Some(b)) if b < a => Span::Whole,
        (Some(a), _) if a >= len => Span::Unsatisfiable,
        (Some(a), Some(b)) => Span::Part(a, b.min(len - 1)),
        (Some(a), None) => Span::Part(a, len - 1),
        (None, Some(0)) => Span::Unsatisfiable,
        (None, Some(n)) => Span::Part(len.saturating_sub(n), len - 1),
        (None, None) => Span::Whole,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_range_is_a_part() {
        assert_eq!(span(Some("bytes=0-3"), 10), Span::Part(0, 3));
        assert_eq!(span(Some("bytes=4-"), 10), Span::Part(4, 9));
        assert_eq!(span(Some("bytes=-3"), 10), Span::Part(7, 9));
        assert_eq!(span(Some("bytes=5-500"), 10), Span::Part(5, 9));
        assert_eq!(span(Some("bytes=-50"), 10), Span::Part(0, 9));
    }

    #[test]
    fn a_range_past_the_end_is_unsatisfiable() {
        assert_eq!(span(Some("bytes=10-12"), 10), Span::Unsatisfiable);
        assert_eq!(span(Some("bytes=-0"), 10), Span::Unsatisfiable);
    }

    #[test]
    fn no_range_several_ranges_and_nonsense_send_the_whole_file() {
        for header in [
            None,
            Some("bytes=0-1,4-5"),
            Some("items=0-1"),
            Some("bytes=x-1"),
            Some("bytes=5-2"),
            Some("bytes=-"),
        ] {
            assert_eq!(span(header, 10), Span::Whole, "{header:?}");
        }
        assert_eq!(span(Some("bytes=0-0"), 0), Span::Whole);
    }
}
