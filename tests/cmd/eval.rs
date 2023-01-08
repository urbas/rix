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
fn eval_bool_expr() {
    assert_cmd(&["--expr", "false || true"])
        .success()
        .stdout(predicate::str::diff("true\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn eval_float_arithmetic_expr() {
    assert_cmd(&["--expr", "3.0 * 4.0 + 1.0 / 2.0"])
        .success()
        .stdout(predicate::str::diff("12.5\n"))
        .stderr(predicate::str::is_empty());
}

fn assert_cmd(eval_args: &[&str]) -> assert_cmd::assert::Assert {
    let mut rix_args = vec!["eval"];
    rix_args.extend_from_slice(eval_args);
    return Command::cargo_bin("rix").unwrap().args(rix_args).assert();
}
