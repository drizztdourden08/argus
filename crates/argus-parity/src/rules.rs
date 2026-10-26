//! What one route's comparison forgives: the volatile fields it leaves out, the arrays whose
//! order the servers do not promise, the rows that belong to one server's own life, and how far
//! apart two timestamps may be.

/// Rows to leave out of an array: those whose `field` (a dot path inside the row) starts with
/// one of the `prefixes`.
#[derive(Debug)]
pub struct RowDrop {
    /// The pattern of the array.
    pub array: &'static str,
    /// The dot path of the text inside a row.
    pub field: &'static str,
    /// The beginnings that drop a row.
    pub prefixes: &'static [&'static str],
}

/// The static part of the rules of one kind of route.
#[derive(Debug)]
pub struct Spec {
    /// Patterns of the values that are never compared.
    pub ignore: &'static [&'static str],
    /// Arrays compared as sets: the pattern of the array and the field its rows are sorted by
    /// (empty for the whole row).
    pub unordered: &'static [(&'static str, &'static str)],
    /// Rows that are not compared.
    pub drop_rows: &'static [RowDrop],
}

/// The rules of one comparison.
#[derive(Debug, Clone, Copy)]
pub struct Rules {
    /// The static part.
    pub spec: &'static Spec,
    /// How far apart two timestamps may be and still be equal.
    pub tolerance_ms: i64,
}
