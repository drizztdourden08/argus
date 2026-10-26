//! The db shim's query: `where` clauses, `orderBy` and `limit`, with the JavaScript comparison
//! rules of the Node `db.mjs`.

use std::cmp::Ordering;

use serde_json::Value;

use crate::{DocRow, StoreError};

/// A `where` operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryOp {
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `in`
    In,
    /// `contains`
    Contains,
}

const OPS: [(&str, QueryOp); 8] = [
    ("==", QueryOp::Eq),
    ("!=", QueryOp::Ne),
    ("<", QueryOp::Lt),
    ("<=", QueryOp::Le),
    (">", QueryOp::Gt),
    (">=", QueryOp::Ge),
    ("in", QueryOp::In),
    ("contains", QueryOp::Contains),
];

/// A validated query.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Query {
    /// `[field, op, value]` clauses, all of which must hold.
    pub filters: Vec<(String, QueryOp, Value)>,
    /// The field to sort by and whether descending.
    pub order_by: Option<(String, bool)>,
    /// At most this many docs (1000 when absent).
    pub limit: Option<usize>,
}

impl Query {
    /// Validates a query body as the Node `validateQuery` does.
    ///
    /// # Errors
    /// A `bad_request` with the Node message.
    pub fn parse(q: &Value) -> Result<Query, StoreError> {
        if q.is_null() {
            return Ok(Query::default());
        }
        let Some(obj) = q.as_object() else {
            return Err(StoreError::bad("query must be an object"));
        };
        Ok(Query {
            filters: obj
                .get("where")
                .map(parse_where)
                .transpose()?
                .unwrap_or_default(),
            order_by: obj.get("orderBy").map(parse_order).transpose()?,
            limit: obj.get("limit").map(parse_limit).transpose()?,
        })
    }

    /// The rows that match, sorted and limited.
    #[must_use]
    pub fn run(&self, rows: Vec<DocRow>) -> Vec<DocRow> {
        let mut rows: Vec<DocRow> = rows
            .into_iter()
            .filter(|r| {
                self.filters
                    .iter()
                    .all(|(f, op, want)| matches(path(&r.data, f), *op, want))
            })
            .collect();
        if let Some((field, desc)) = &self.order_by {
            rows.sort_by(|a, b| {
                let o = compare(path(&a.data, field), path(&b.data, field));
                if *desc { o.reverse() } else { o }
            });
        }
        rows.truncate(self.limit.unwrap_or(1000));
        rows
    }
}

fn parse_where(w: &Value) -> Result<Vec<(String, QueryOp, Value)>, StoreError> {
    let Some(list) = w.as_array() else {
        return Err(StoreError::bad(
            "where must be an array of [field, op, value]",
        ));
    };
    list.iter().map(parse_clause).collect()
}

fn parse_clause(c: &Value) -> Result<(String, QueryOp, Value), StoreError> {
    let parts = c.as_array().filter(|a| a.len() == 3);
    let Some((field, op, value)) = parts.and_then(|a| Some((a[0].as_str()?, &a[1], &a[2]))) else {
        return Err(StoreError::bad("each where clause is [field, op, value]"));
    };
    let found = OPS
        .iter()
        .find(|(name, _)| op.as_str() == Some(name))
        .map(|(_, o)| *o);
    let names: Vec<&str> = OPS.iter().map(|(n, _)| *n).collect();
    let op = found.ok_or_else(|| {
        StoreError::bad(format!(
            "unknown operator \"{}\" (use {})",
            argus_core::js::string(op),
            names.join(" ")
        ))
    })?;
    Ok((field.to_owned(), op, value.clone()))
}

fn parse_order(o: &Value) -> Result<(String, bool), StoreError> {
    let bad = || StoreError::bad("orderBy is [field, 'asc'|'desc']");
    let arr = o.as_array().ok_or_else(bad)?;
    let field = arr.first().and_then(Value::as_str).ok_or_else(bad)?;
    match arr.get(1) {
        None => Ok((field.to_owned(), false)),
        Some(Value::String(d)) if d == "asc" || d == "desc" => Ok((field.to_owned(), d == "desc")),
        Some(_) => Err(bad()),
    }
}

fn parse_limit(l: &Value) -> Result<usize, StoreError> {
    l.as_u64()
        .filter(|n| *n <= 1000)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| StoreError::bad("limit must be an integer 0..1000"))
}

fn path<'a>(data: &'a Value, field: &str) -> Option<&'a Value> {
    field.split('.').try_fold(data, |v, k| v.get(k))
}

fn scalar_eq(a: &Value, b: &Value) -> bool {
    !a.is_object() && !a.is_array() && a == b
}

fn ordered(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64()?.partial_cmp(&y.as_f64()?),
        (Value::String(x), Value::String(y)) => Some(x.encode_utf16().cmp(y.encode_utf16())),
        (Value::Bool(x), Value::Bool(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

fn matches(value: Option<&Value>, op: QueryOp, want: &Value) -> bool {
    let present = value.filter(|v| !v.is_null());
    match op {
        QueryOp::Eq => value.is_some_and(|v| scalar_eq(v, want)),
        QueryOp::Ne => !value.is_some_and(|v| scalar_eq(v, want)),
        QueryOp::Lt => present.and_then(|v| ordered(v, want)) == Some(Ordering::Less),
        QueryOp::Le => present
            .and_then(|v| ordered(v, want))
            .is_some_and(Ordering::is_le),
        QueryOp::Gt => present.and_then(|v| ordered(v, want)) == Some(Ordering::Greater),
        QueryOp::Ge => present
            .and_then(|v| ordered(v, want))
            .is_some_and(Ordering::is_ge),
        QueryOp::In => want
            .as_array()
            .is_some_and(|a| value.is_some_and(|v| a.iter().any(|x| scalar_eq(x, v)))),
        QueryOp::Contains => contains(value, want),
    }
}

fn contains(value: Option<&Value>, want: &Value) -> bool {
    match value {
        Some(Value::Array(a)) => a.iter().any(|x| scalar_eq(x, want)),
        Some(Value::String(s)) => want.as_str().is_some_and(|w| s.contains(w)),
        _ => false,
    }
}

fn compare(a: Option<&Value>, b: Option<&Value>) -> Ordering {
    let a = a.filter(|v| !v.is_null());
    let b = b.filter(|v| !v.is_null());
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => ordered(x, y).unwrap_or(Ordering::Equal),
    }
}
