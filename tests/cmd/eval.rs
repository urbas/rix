use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn help() {
    assert_cmd(&["--help"])
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn eval_arithmetic_expr() {
    assert_cmd(&["--expr", "1 + 2 + 3"])
        .success()
        .stdout(predicate::str::diff("6\n"))
        .stderr(predicate::str::is_empty());
}

fn assert_cmd(eval_args: &[&str]) -> assert_cmd::assert::Assert {
    let mut rix_args = vec!["eval"];
    rix_args.extend_from_slice(eval_args);
    return Command::cargo_bin("rix").unwrap().args(rix_args).assert();
}
