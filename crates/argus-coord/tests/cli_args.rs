//! Ports the behaviour of `cli-args.mjs` (`parseArgs`, `flag`, `many`). Node has no test file of
//! its own for it; the cases are the promises of the module and its callers.

use argus_coord::cli::{Opt, Val, parse_args};

fn parse(words: &[&str]) -> argus_coord::cli::Args {
    let argv: Vec<String> = words.iter().map(|w| (*w).to_owned()).collect();
    parse_args(&argv)
}

fn text(s: &str) -> Val {
    Val::Text(s.to_owned())
}

#[test]
fn words_that_are_not_options_are_positional_in_order() {
    let args = parse(&["thread", "register", "ui", "-x", "-"]);
    assert_eq!(args.positional, ["thread", "register", "ui", "-x", "-"]);
}

#[test]
fn an_option_takes_the_next_word_as_its_value() {
    let args = parse(&["item", "add", "--subject", "ui", "Title"]);
    assert_eq!(args.positional, ["item", "add", "Title"]);
    assert_eq!(args.options.text("subject"), Some("ui"));
}

#[test]
fn key_equals_value_keeps_everything_after_the_first_equals() {
    let args = parse(&["--env=K=V", "--empty="]);
    assert_eq!(args.options.text("env"), Some("K=V"));
    assert_eq!(args.options.text("empty"), Some(""));
}

#[test]
fn a_bare_option_is_true_and_so_is_one_before_another_option() {
    let args = parse(&["--json", "--as", "ui", "--all"]);
    assert_eq!(args.options.get("json"), Some(&Opt::One(Val::Flag)));
    assert_eq!(args.options.get("all"), Some(&Opt::One(Val::Flag)));
    assert!(args.options.flag("json"));
    assert_eq!(args.options.text("as"), Some("ui"));
}

#[test]
fn a_value_that_starts_with_one_dash_is_still_a_value() {
    let args = parse(&["--title", "-1", "--note", "-x"]);
    assert_eq!(args.options.text("title"), Some("-1"));
    assert_eq!(args.options.text("note"), Some("-x"));
}

#[test]
fn an_empty_word_is_a_value_too() {
    let args = parse(&["--title", "", "tail"]);
    assert_eq!(args.options.text("title"), Some(""));
    assert_eq!(args.positional, ["tail"]);
}

#[test]
fn a_repeated_option_becomes_a_list_in_order() {
    let args = parse(&["--ac", "a", "--ac=b", "--ac", "c"]);
    assert_eq!(
        args.options.get("ac"),
        Some(&Opt::Many(vec![text("a"), text("b"), text("c")]))
    );
    assert_eq!(args.options.many("ac"), [text("a"), text("b"), text("c")]);
}

#[test]
fn a_list_is_not_a_text_so_a_twice_given_project_is_not_a_project() {
    let args = parse(&["--project", "a", "--project", "b"]);
    assert_eq!(args.options.text("project"), None);
}

#[test]
fn a_flag_given_twice_stays_in_the_list_as_true_twice() {
    let args = parse(&["--x", "--x"]);
    assert_eq!(
        args.options.get("x"),
        Some(&Opt::Many(vec![Val::Flag, Val::Flag]))
    );
    assert!(!args.options.flag("x"));
}

#[test]
fn flag_accepts_true_and_the_text_true_given_once() {
    assert!(parse(&["--force"]).options.flag("force"));
    assert!(parse(&["--force", "true"]).options.flag("force"));
    assert!(parse(&["--force=true"]).options.flag("force"));
    assert!(!parse(&["--force", "yes"]).options.flag("force"));
    assert!(!parse(&[]).options.flag("force"));
}

#[test]
fn many_is_always_a_list() {
    assert_eq!(parse(&[]).options.many("ac"), Vec::<Val>::new());
    assert_eq!(parse(&["--ac"]).options.many("ac"), Vec::<Val>::new());
    assert_eq!(parse(&["--ac", "one"]).options.many("ac"), [text("one")]);
}

#[test]
fn json_gives_true_a_text_or_a_list() {
    let args = parse(&["--a", "--b", "x", "--c", "1", "--c", "2"]);
    assert_eq!(args.options.json("a"), Some(serde_json::json!(true)));
    assert_eq!(args.options.json("b"), Some(serde_json::json!("x")));
    assert_eq!(args.options.json("c"), Some(serde_json::json!(["1", "2"])));
    assert_eq!(args.options.json("d"), None);
}

#[test]
fn a_double_dash_alone_is_an_option_with_an_empty_name() {
    let args = parse(&["--", "x"]);
    assert_eq!(args.options.text(""), Some("x"));
    assert_eq!(args.positional, Vec::<String>::new());
}

#[test]
fn help_is_an_option_not_a_word() {
    let args = parse(&["--help"]);
    assert_eq!(args.positional, Vec::<String>::new());
    assert!(args.options.flag("help"));
}
