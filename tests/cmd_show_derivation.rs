use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn show_derivation_help() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["show-derivation", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"));
}
