use crate::building::simple_derivation;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use rix::derivations::save_derivation;
use std::fs::{read_to_string, File};
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn help() {
    assert_cmd(&["--help"])
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn basic_api() {
    let tmp_dir = tempdir().unwrap();
    let build_dir = tempdir().unwrap();

    let derivation = simple_derivation(
        &tmp_dir,
        Path::new("/output"),
        "echo hello world && echo broken world 1>&2 && echo hello file > file.out",
    );
    let derivation_path = tmp_dir.path().join("foo.drv");
    let mut derivation_file = File::create(&derivation_path).unwrap();
    save_derivation(&mut derivation_file, &derivation).unwrap();

    let stdout_path = tmp_dir.path().join("stdout");
    File::create(&stdout_path).unwrap();
    let stderr_path = tmp_dir.path().join("stderr");
    File::create(&stderr_path).unwrap();

    assert_cmd(&[
        "--stdout",
        &stdout_path.to_str().unwrap(),
        "--stderr",
        &stderr_path.to_str().unwrap(),
        "--build-dir",
        &build_dir.path().to_str().unwrap(),
        &derivation_path.to_str().unwrap(),
    ])
    .success()
    .stderr(predicate::str::is_empty());

    assert_eq!(read_to_string(&stdout_path).unwrap(), "hello world\n");
    assert_eq!(read_to_string(&stderr_path).unwrap(), "broken world\n");
    assert_eq!(
        read_to_string(&build_dir.path().join("file.out")).unwrap(),
        "hello file\n"
    );
}

fn assert_cmd(hash_args: &[&str]) -> assert_cmd::assert::Assert {
    let mut rix_args = vec!["build-derivation"];
    rix_args.extend_from_slice(hash_args);
    return Command::cargo_bin("rix").unwrap().args(rix_args).assert();
}
