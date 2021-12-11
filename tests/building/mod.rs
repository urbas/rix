use crate::test_utils::tmp_file;
use rix::building;
use rix::derivations::{Derivation, DerivationOutput};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::str;
use tempfile::tempdir;

#[test]
fn test_build_derivation() {
    let tmp_dir = tempdir().unwrap();
    let out_dir = tmp_dir.path().join("output");
    let derivation = simple_derivation(&tmp_dir, &out_dir, "mkdir $out && touch $out/hello");
    building::build_derivation_command(&derivation, &tmp_dir.path())
        .output()
        .unwrap();
    assert!(out_dir.join("hello").exists());
}

#[test]
fn test_build_derivation_sandboxed_success() {
    let tmp_dir = tempdir().unwrap();
    let build_dir = tempdir().unwrap();
    let derivation = simple_derivation(
        &tmp_dir,
        Path::new("/output"),
        "mkdir -p $out && touch $out/hello",
    );
    assert_eq!(
        building::build_derivation_sandboxed(&derivation, &build_dir.path()).unwrap(),
        0
    );
    assert!(build_dir.path().join("output/hello").exists());
}

#[test]
fn test_build_derivation_sandboxed_missing_deps() {
    let tmp_dir = tempdir().unwrap();
    let builder = &get_pkg_closure(".#busybox-sandbox-shell")[0];
    let derivation = derivation_with_deps(
        &tmp_dir,
        Path::new("/output"),
        "mkdir $out && touch $out/hello",
        builder,
        &vec![],
    );
    assert_ne!(
        building::build_derivation_sandboxed(&derivation, &tmp_dir.path()).unwrap(),
        0
    );
}

fn simple_derivation(
    tmp_dir: &tempfile::TempDir,
    out_dir: &Path,
    builder_script: &str,
) -> Derivation {
    let builder = get_pkg_closure(".#busybox-sandbox-shell");
    let coreutils = get_pkg_closure(".#coreutils");
    return derivation_with_deps(tmp_dir, out_dir, builder_script, &builder[0], &coreutils);
}

fn derivation_with_deps(
    src_dir: &tempfile::TempDir,
    out_dir: &Path,
    builder_script: &str,
    builder: &str,
    input_srcs: &Vec<String>,
) -> Derivation {
    let builder_script_file = tmp_file(&src_dir, "builder.sh", builder_script);
    fs::set_permissions(&builder_script_file, fs::Permissions::from_mode(0o640)).unwrap();

    Derivation {
        builder: format!("{}/bin/busybox", builder),
        args: vec!["sh".to_owned(), builder_script_file.clone()],
        env: vec![("out".to_owned(), out_dir.to_str().unwrap().to_owned())]
            .into_iter()
            .collect(),
        input_drvs: HashMap::new(),
        input_srcs: input_srcs
            .iter()
            .chain(&[builder.to_owned(), builder_script_file.clone()])
            .map(|s| s.clone())
            .collect(),
        outputs: vec![(
            "out".to_owned(),
            DerivationOutput {
                hash: "".to_owned(),
                hash_algo: "".to_owned(),
                path: out_dir.to_str().unwrap().to_owned(),
            },
        )]
        .into_iter()
        .collect(),
        platform: "any".to_owned(),
    }
}

fn get_pkg_closure(nix_flake_attr: &str) -> Vec<String> {
    let drv_out = Command::new("nix")
        .args(&["path-info", "-r", nix_flake_attr])
        .output()
        .expect("failed to get the derivation");

    str::from_utf8(&drv_out.stdout)
        .unwrap()
        .trim()
        .lines()
        .map(String::from)
        .collect()
}
