use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn transpile_help() {
    assert_cmd(&["--help"])
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn transpile_bool_expr() {
    assert_cmd(&["--expr", "1.0"])
        .success()
        .stdout(predicate::str::contains(
            "export const __nixValue = (ctx) => new n.NixFloat(1.0);\n",
        ))
        .stderr(predicate::str::is_empty());
}

fn assert_cmd(eval_args: &[&str]) -> assert_cmd::assert::Assert {
    let mut rix_args = vec!["transpile"];
    rix_args.extend_from_slice(eval_args);
    return Command::cargo_bin("rix").unwrap().args(rix_args).assert();
}
