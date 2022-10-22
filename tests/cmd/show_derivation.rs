use crate::test_utils::tmp_file;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
use std::process::Command;
use std::str;
use tempfile::tempdir;

#[test]
fn show_derivation_help() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["show-derivation", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"));
}

#[test]
fn show_derivation() {
    let tmpdir = tempdir().unwrap();
    let derivation_contents = r#"Derive([("out","/foo","sha256","abc")],[("/drv1",["out"]),("/drv2",["dev"])],["/builder.sh"],"x86_64-linux","/bash",["-e","/builder.sh"],[("ENV1","val1"),("ENV2","val2")])"#;
    let derivation_path = tmp_file(&tmpdir, "foo.drv", derivation_contents);

    let expected_output: Value = serde_json::from_str(format!(
        "{{\"{}\":{}}}",
        &derivation_path,
        r#"{"args":["-e","/builder.sh"],"builder":"/bash","env":{"ENV1":"val1","ENV2":"val2"},"inputDrvs":{"/drv2":["dev"],"/drv1":["out"]},"inputSrcs":["/builder.sh"],"outputs":{"out":{"hash":"abc","hashAlgo":"sha256","path":"/foo"}},"system":"x86_64-linux"}"#,
    ).as_str()).unwrap();

    let cmd_result = Command::cargo_bin("rix")
        .unwrap()
        .args(["show-derivation", &derivation_path])
        .assert()
        .success();

    let output: Value =
        serde_json::from_str(str::from_utf8(&cmd_result.get_output().stdout).unwrap()).unwrap();

    assert_eq!(output, expected_output);
}
