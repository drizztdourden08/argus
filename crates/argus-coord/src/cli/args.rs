//! The hand port of `parseArgs` (`cli-args.mjs`): positionals, `--flag` booleans and
//! `--key value` / `--key=value` options. Not `clap`, because the behaviour is quirky and every
//! typed command must parse as it did: a `--key` followed by a word that does not start with
//! `--` takes it as its value (a single dash included), a repeated option becomes a list, a bare
//! `--key` is `true`, and `--key=` is the empty text.

use serde_json::Value;

/// One value of an option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    /// A bare `--key`: JavaScript `true`.
    Flag,
    /// A text.
    Text(String),
}

impl Val {
    /// The value as JSON: `true` or the text.
    #[must_use]
    pub fn to_json(&self) -> Value {
        match self {
            Val::Flag => Value::Bool(true),
            Val::Text(t) => Value::String(t.clone()),
        }
    }
}

/// What an option holds: one value, or a list once the option was given again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opt {
    /// Given once.
    One(Val),
    /// Given twice or more, in order.
    Many(Vec<Val>),
}

/// The options of a call, in the order they first appeared.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    entries: Vec<(String, Opt)>,
}

impl Options {
    fn put(&mut self, key: &str, val: Val) {
        let Some(at) = self.entries.iter().position(|(k, _)| k == key) else {
            self.entries.push((key.to_owned(), Opt::One(val)));
            return;
        };
        let slot = &mut self.entries[at].1;
        match slot {
            Opt::Many(list) => list.push(val),
            Opt::One(first) => *slot = Opt::Many(vec![first.clone(), val]),
        }
    }

    /// What `options[name]` holds, `None` when it is undefined.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Opt> {
        self.entries.iter().find(|(k, _)| k == name).map(|(_, o)| o)
    }

    /// `options[name]` when it is a text (`typeof === 'string'`): a flag or a list is `None`.
    #[must_use]
    pub fn text(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(Opt::One(Val::Text(t))) => Some(t),
            _ => None,
        }
    }

    /// `flag(options, name)`: `true` as a bare flag or as the text `true`, given once.
    #[must_use]
    pub fn flag(&self, name: &str) -> bool {
        match self.get(name) {
            Some(Opt::One(Val::Flag)) => true,
            Some(Opt::One(Val::Text(t))) => t == "true",
            _ => false,
        }
    }

    /// `many(options, name)`: always a list, whether the option was given once, many times or
    /// not at all. A bare flag given once is not a value and gives an empty list.
    #[must_use]
    pub fn many(&self, name: &str) -> Vec<Val> {
        match self.get(name) {
            None | Some(Opt::One(Val::Flag)) => Vec::new(),
            Some(Opt::One(text)) => vec![text.clone()],
            Some(Opt::Many(list)) => list.clone(),
        }
    }

    /// `options[name]` as JSON: `true`, a text or a list of those; `None` when undefined.
    #[must_use]
    pub fn json(&self, name: &str) -> Option<Value> {
        Some(match self.get(name)? {
            Opt::One(v) => v.to_json(),
            Opt::Many(list) => Value::Array(list.iter().map(Val::to_json).collect()),
        })
    }
}

/// The result of splitting an argument vector.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Args {
    /// The words that are not options, in order.
    pub positional: Vec<String>,
    /// The options.
    pub options: Options,
}

/// Splits arguments as `parseArgs` does.
#[must_use]
pub fn parse_args(argv: &[String]) -> Args {
    let mut parsed = Args::default();
    let mut i = 0;
    while i < argv.len() {
        let arg = &argv[i];
        i += 1;
        let Some(body) = arg.strip_prefix("--") else {
            parsed.positional.push(arg.clone());
            continue;
        };
        if let Some((key, value)) = body.split_once('=') {
            parsed.options.put(key, Val::Text(value.to_owned()));
        } else if let Some(next) = argv.get(i).filter(|n| !n.starts_with("--")) {
            parsed.options.put(body, Val::Text(next.clone()));
            i += 1;
        } else {
            parsed.options.put(body, Val::Flag);
        }
    }
    parsed
}
