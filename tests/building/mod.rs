use crate::test_utils::tmp_file;
use rix::building::{build_derivation_command, build_derivation_sandboxed, BuildConfig};
use rix::derivations::{Derivation, DerivationOutput};
use std::collections::{HashMap, HashSet};
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
    build_derivation_command(&derivation, &tmp_dir.path())
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
        build_derivation_sandboxed(&BuildConfig::new(&derivation, &build_dir.path())).unwrap(),
        0
    );
    assert!(build_dir.path().join("output/hello").exists());
}

#[test]
fn test_build_derivation_sandboxed_redirect_to_file() {
    let tmp_dir = tempdir().unwrap();
    let build_dir = tempdir().unwrap();
    let derivation = simple_derivation(
        &tmp_dir,
        Path::new("/output"),
        "echo hello world && echo broken world 1>&2",
    );
    let stdout_file = fs::File::create(tmp_dir.path().join("stdout")).unwrap();
    let stderr_file = fs::File::create(tmp_dir.path().join("stderr")).unwrap();
    let mut build_config = BuildConfig::new(&derivation, &build_dir.path());
    build_config.stdout_to_file(&stdout_file);
    build_config.stderr_to_file(&stderr_file);
    assert_eq!(build_derivation_sandboxed(&build_config).unwrap(), 0);
    assert_eq!(
        fs::read_to_string(tmp_dir.path().join("stdout")).unwrap(),
        "hello world\n"
    );
    assert_eq!(
        fs::read_to_string(tmp_dir.path().join("stderr")).unwrap(),
        "broken world\n"
    );
}

#[test]
fn test_build_derivation_sandboxed_missing_deps() {
    let tmp_dir = tempdir().unwrap();
    let builder = &get_pkg_closure(".#busybox-sandbox-shell")[0];
    let derivation = test_derivation(
        &tmp_dir,
        Path::new("/output"),
        "mkdir $out && touch $out/hello",
        builder,
        &vec![],
        HashMap::new(),
    );
    let stderr_path = tmp_dir.path().join("stderr");
    let stderr_file = fs::File::create(&stderr_path).unwrap();
    let mut build_config = BuildConfig::new(&derivation, &tmp_dir.path());
    build_config.stderr_to_file(&stderr_file);
    assert_ne!(build_derivation_sandboxed(&build_config).unwrap(), 0);
    assert!(!fs::read_to_string(&stderr_path).unwrap().is_empty());
}

#[test]
fn test_build_derivation_sandboxed_input_drvs() {
    let tmp_dir = tempdir().unwrap();
    let build_dir = tempdir().unwrap();
    let builder = &get_pkg_closure(".#busybox-sandbox-shell")[0];
    let coreutils_closure = &get_pkg_closure(".#coreutils");
    let input_drvs = get_derivation_paths(coreutils_closure);
    let derivation = test_derivation(
        &tmp_dir,
        Path::new("/output"),
        "mkdir $out && touch $out/hello",
        builder,
        &Vec::new(),
        input_drvs
            .into_iter()
            .map(|drv| (drv, HashSet::from(["out".to_owned()])))
            .collect(),
    );
    assert_eq!(
        build_derivation_sandboxed(&BuildConfig::new(&derivation, &build_dir.path())).unwrap(),
        0
    );
    assert!(build_dir.path().join("output/hello").exists());
}

pub fn simple_derivation(
    tmp_dir: &tempfile::TempDir,
    out_dir: &Path,
    builder_script: &str,
) -> Derivation {
    let builder = get_pkg_closure(".#busybox-sandbox-shell");
    let coreutils = get_pkg_closure(".#coreutils");
    return test_derivation(
        tmp_dir,
        out_dir,
        builder_script,
        &builder[0],
        &coreutils,
        HashMap::new(),
    );
}

pub fn test_derivation(
    src_dir: &tempfile::TempDir,
    out_dir: &Path,
    builder_script: &str,
    builder: &str,
    input_srcs: &Vec<String>,
    input_drvs: HashMap<String, HashSet<String>>,
) -> Derivation {
    let builder_script_file = tmp_file(&src_dir, "builder.sh", builder_script);
    fs::set_permissions(&builder_script_file, fs::Permissions::from_mode(0o640)).unwrap();

    Derivation {
        builder: format!("{}/bin/busybox", builder),
        args: vec!["sh".to_owned(), builder_script_file.clone()],
        env: HashMap::from([("out".to_owned(), out_dir.to_str().unwrap().to_owned())]),
        input_drvs: input_drvs,
        input_srcs: input_srcs
            .iter()
            .chain(&[builder.to_owned(), builder_script_file.clone()])
            .cloned()
            .collect(),
        outputs: HashMap::from([(
            "out".to_owned(),
            DerivationOutput {
                hash: Some("".to_owned()),
                hash_algo: Some("".to_owned()),
                path: out_dir.to_str().unwrap().to_owned(),
            },
        )]),
        system: "any".to_owned(),
    }
}

pub fn get_pkg_closure(nix_flake_attr: &str) -> Vec<String> {
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

pub fn get_derivation_paths(show_derivation_args: &Vec<String>) -> Vec<String> {
    let mut nix_args = vec!["show-derivation".to_owned()];
    nix_args.extend(show_derivation_args.iter().cloned());
    let show_drv_out = Command::new("nix")
        .args(&nix_args)
        .output()
        .expect("failed to show the derivation");

    let parsed_out: HashMap<String, Derivation> =
        serde_json::from_str(str::from_utf8(&show_drv_out.stdout).unwrap()).unwrap();

    parsed_out.keys().cloned().collect()
}
