use crate::test_utils::tmp_file;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use rix::derivations::save_derivation;
use rix::derivations::{Derivation, DerivationOutput};
use std::collections::{HashMap, HashSet};
use std::fs::{read_to_string, File};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::{fs, str, thread};
use tempfile::tempdir;

#[test]
fn help() {
    assert_cmd(&["--help"])
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn build_derivations() {
    // We have to call nix in order to get some bacis dependencies for the tests.
    // Unfortunately, calling nix over and over again is expensive. This is why
    // we call nix here upfront just once and then call test functions in parallel.
    let test_data = TestData::new();
    thread::scope(|scope| {
        scope.spawn(|| build_derivation_success(&test_data));
        scope.spawn(|| build_derivation_missing_deps(&test_data));
        scope.spawn(|| build_derivation_sandboxed_input_drvs(&test_data));
    });
}

fn build_derivation_success(test_data: &TestData) {
    let tmp_dir = tempdir().unwrap();
    let build_dir = tempdir().unwrap();

    let derivation = simple_derivation(
        test_data,
        &tmp_dir,
        Path::new("/output"),
        "echo hello world && echo broken world 1>&2 && mkdir -p $out && echo hello file > $out/file.out",
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
        read_to_string(&build_dir.path().join("output/file.out")).unwrap(),
        "hello file\n"
    );
    assert!(build_dir.path().join("dev/null").exists());
    assert!(fs::read_to_string(build_dir.path().join("dev/null"))
        .unwrap()
        .is_empty());
}

fn build_derivation_missing_deps(test_data: &TestData) {
    let tmp_dir = tempdir().unwrap();
    let build_dir = tempdir().unwrap();

    let derivation = test_derivation(
        &tmp_dir,
        Path::new("/output"),
        "mkdir $out && touch $out/hello",
        &test_data.busybox_closure[0],
        &vec![],
        HashMap::new(),
        HashMap::new(),
    );
    let derivation_path = tmp_dir.path().join("foo.drv");
    let mut derivation_file = File::create(&derivation_path).unwrap();
    save_derivation(&mut derivation_file, &derivation).unwrap();

    let stderr_path = tmp_dir.path().join("stderr");
    fs::File::create(&stderr_path).unwrap();

    assert_cmd(&[
        "--stderr",
        &stderr_path.to_str().unwrap(),
        "--build-dir",
        &build_dir.path().to_str().unwrap(),
        &derivation_path.to_str().unwrap(),
    ])
    .failure()
    .stderr(predicate::str::is_empty());

    assert!(!fs::read_to_string(&stderr_path).unwrap().is_empty());
}

fn build_derivation_sandboxed_input_drvs(test_data: &TestData) {
    let tmp_dir = tempdir().unwrap();
    let build_dir = tempdir().unwrap();

    let derivation = test_derivation(
        &tmp_dir,
        Path::new("/output"),
        "mkdir $out && touch $out/hello",
        &test_data.busybox_closure[0],
        &Vec::new(),
        test_data
            .coreutils_drvs_closure
            .iter()
            .cloned()
            .map(|drv| (drv, HashSet::from(["out".to_owned()])))
            .collect(),
        HashMap::from([(
            "PATH".to_owned(),
            build_path(test_data.coreutils_closure.iter()),
        )]),
    );
    let derivation_path = tmp_dir.path().join("foo.drv");
    let mut derivation_file = File::create(&derivation_path).unwrap();
    save_derivation(&mut derivation_file, &derivation).unwrap();

    assert_cmd(&[
        "--build-dir",
        &build_dir.path().to_str().unwrap(),
        &derivation_path.to_str().unwrap(),
    ])
    .success()
    .stderr(predicate::str::is_empty());

    assert!(build_dir.path().join("output/hello").exists());
}

fn assert_cmd(hash_args: &[&str]) -> assert_cmd::assert::Assert {
    let mut rix_args = vec!["build-derivation"];
    rix_args.extend_from_slice(hash_args);
    return Command::cargo_bin("rix").unwrap().args(rix_args).assert();
}

#[derive(Clone)]
struct TestData {
    busybox_closure: Vec<String>,
    coreutils_closure: Vec<String>,
    coreutils_drvs_closure: Vec<String>,
}

impl TestData {
    pub fn new() -> Self {
        let coreutils_closure = get_pkg_closure(".#coreutils");
        let coreutils_drvs_closure = show_derivation(&coreutils_closure);
        TestData {
            busybox_closure: get_pkg_closure(".#busybox-sandbox-shell"),
            coreutils_closure: coreutils_closure,
            coreutils_drvs_closure: coreutils_drvs_closure,
        }
    }
}

fn simple_derivation(
    test_data: &TestData,
    tmp_dir: &tempfile::TempDir,
    out_dir: &Path,
    builder_script: &str,
) -> Derivation {
    let coreutils = get_pkg_closure(".#coreutils");
    return test_derivation(
        tmp_dir,
        out_dir,
        builder_script,
        &test_data.busybox_closure[0],
        &test_data.coreutils_closure,
        HashMap::new(),
        HashMap::from([("PATH".to_owned(), build_path(coreutils.iter()))]),
    );
}

fn test_derivation(
    src_dir: &tempfile::TempDir,
    out_dir: &Path,
    builder_script: &str,
    builder: &str,
    input_srcs: &Vec<String>,
    input_drvs: HashMap<String, HashSet<String>>,
    mut env: HashMap<String, String>,
) -> Derivation {
    let builder_script_file = tmp_file(&src_dir, "builder.sh", builder_script);
    fs::set_permissions(&builder_script_file, fs::Permissions::from_mode(0o640)).unwrap();
    env.extend([("out".to_owned(), out_dir.to_str().unwrap().to_owned())]);

    Derivation {
        builder: format!("{}/bin/busybox", builder),
        args: vec!["sh".to_owned(), builder_script_file.clone()],
        env: env,
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

fn show_derivation(show_derivation_args: &Vec<String>) -> Vec<String> {
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

fn build_path<'a>(store_paths: impl Iterator<Item = &'a String>) -> String {
    store_paths
        .map(|path| format!("{}/bin", path))
        .collect::<Vec<String>>()
        .join(":")
}
