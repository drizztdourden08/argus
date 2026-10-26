//! `a.localeCompare(b)` for the paths and names the listings sort: the order of the root
//! collation for ASCII (punctuation, digits, then letters ignoring case, lower case before
//! upper case on a tie) and the code point order for the rest.

use std::cmp::Ordering;

const PUNCTUATION: &str = "_-,;:!?.'\"()[]{}@*/\\&#%`^+<=>|~$";

fn primary(c: char) -> u32 {
    if c.is_whitespace() {
        return 0;
    }
    if let Some(at) = PUNCTUATION.find(c) {
        return 1 + u32::try_from(at).unwrap_or(0);
    }
    if let Some(digit) = c.to_digit(10).filter(|_| c.is_ascii_digit()) {
        return 100 + digit;
    }
    if c.is_ascii_alphabetic() {
        return 200 + u32::from(c.to_ascii_lowercase());
    }
    1000 + u32::from(c)
}

fn case_rank(c: char) -> u8 {
    u8::from(c.is_uppercase())
}

/// How `a.localeCompare(b)` orders two texts.
#[must_use]
pub fn compare(a: &str, b: &str) -> Ordering {
    let keys = |s: &str| s.chars().map(primary).collect::<Vec<_>>();
    keys(a).cmp(&keys(b)).then_with(|| {
        let cases = |s: &str| s.chars().map(case_rank).collect::<Vec<_>>();
        cases(a).cmp(&cases(b))
    })
}

#[cfg(test)]
mod tests {
    use super::compare;
    use std::cmp::Ordering::{Greater, Less};

    #[test]
    fn punctuation_digits_then_letters_ignoring_case() {
        assert_eq!(compare("a-b", "ab"), Less);
        assert_eq!(compare("a1", "ab"), Less);
        assert_eq!(compare("B", "a"), Greater);
        assert_eq!(compare("a", "A"), Less);
        assert_eq!(compare("X:/a/b", "X:/a-b/c"), Greater);
    }
}
