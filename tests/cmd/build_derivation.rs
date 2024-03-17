use crate::test_utils::tmp_file;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use rix::derivations::{load_derivation, save_derivation, Derivation, DerivationOutput, InputDrv};
use std::collections::{BTreeMap, BTreeSet};
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
    // We have to call nix in order to get some basic dependencies for the tests.
    // Unfortunately, calling nix over and over again is expensive. This is why
    // we call nix here upfront just once and then call test functions in parallel.
    let test_data = TestData::new();
    thread::scope(|scope| {
        scope.spawn(|| load_and_save_derivation_stable(&test_data));
        scope.spawn(|| build_derivation_success(&test_data));
        scope.spawn(|| build_derivation_missing_deps(&test_data));
        scope.spawn(|| build_derivation_sandboxed_input_drvs(&test_data));
    });
}

fn load_and_save_derivation_stable(test_data: &TestData) {
    let parsed_derivation = load_derivation(&test_data.coreutils_drv_path).unwrap();
    let mut derivation_bytes = Vec::new();
    save_derivation(&mut derivation_bytes, &parsed_derivation).unwrap();
    assert_eq!(
        str::from_utf8(&derivation_bytes).unwrap(),
        fs::read_to_string(&test_data.coreutils_drv_path).unwrap(),
    );
}

fn build_derivation_success(test_data: &TestData) {
    let tmp_dir = tempdir().unwrap();
    let build_dir = tempdir().unwrap();

    let builder_script = "echo hello world && echo broken world 1>&2 && mkdir -p $out && echo hello file > $out/file.out";
    let derivation = simple_derivation(test_data, &tmp_dir, builder_script);
    let derivation_path = tmp_dir.path().join("foo.drv");
    save_derivation(&mut File::create(&derivation_path).unwrap(), &derivation).unwrap();

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
    let busybox_derivation = load_derivation(&test_data.busybox_drv_path).unwrap();
    let coreutils_derivation = load_derivation(&test_data.coreutils_drv_path).unwrap();

    let derivation = test_derivation(
        &tmp_dir,
        Path::new("/output"),
        "mkdir $out && touch $out/hello",
        &format!("{}/bin/sh", busybox_derivation.outputs["out"].path),
        &vec![],
        BTreeMap::from([(
            test_data.busybox_drv_path.clone(),
            InputDrv {
                dynamic_outputs: BTreeMap::new(),
                outputs: BTreeSet::from(["out".to_owned()]),
            },
        )]),
        BTreeMap::from([(
            "PATH".to_owned(),
            format!("{}/bin", coreutils_derivation.outputs["out"].path),
        )]),
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

    let derivation = simple_derivation(test_data, &tmp_dir, "mkdir $out && touch $out/hello");
    let derivation_path = tmp_dir.path().join("foo.drv");
    save_derivation(&mut File::create(&derivation_path).unwrap(), &derivation).unwrap();

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
    busybox_drv_path: String,
    coreutils_drv_path: String,
}

impl TestData {
    pub fn new() -> Self {
        TestData {
            busybox_drv_path: get_derivation_path(".#pkgs.busybox-sandbox-shell")
                .expect("Couldn't find the derivation for busybox."),
            coreutils_drv_path: get_derivation_path(".#pkgs.coreutils")
                .expect("Couldn't find derivation for coreutils."),
        }
    }
}

fn simple_derivation(
    test_data: &TestData,
    tmp_dir: &tempfile::TempDir,
    builder_script: &str,
) -> Derivation {
    let busybox_derivation = load_derivation(&test_data.busybox_drv_path).unwrap();
    let coreutils_derivation = load_derivation(&test_data.coreutils_drv_path).unwrap();
    return test_derivation(
        tmp_dir,
        Path::new("/output"),
        builder_script,
        &format!("{}/bin/sh", busybox_derivation.outputs["out"].path),
        &vec![],
        BTreeMap::from([
            (
                test_data.coreutils_drv_path.clone(),
                InputDrv {
                    dynamic_outputs: BTreeMap::new(),
                    outputs: BTreeSet::from(["out".to_owned()]),
                },
            ),
            (
                test_data.busybox_drv_path.clone(),
                InputDrv {
                    dynamic_outputs: BTreeMap::new(),
                    outputs: BTreeSet::from(["out".to_owned()]),
                },
            ),
        ]),
        BTreeMap::from([(
            "PATH".to_owned(),
            format!("{}/bin", coreutils_derivation.outputs["out"].path),
        )]),
    );
}

fn test_derivation(
    src_dir: &tempfile::TempDir,
    out_dir: &Path,
    builder_script: &str,
    builder: &str,
    input_srcs: &Vec<String>,
    input_drvs: BTreeMap<String, InputDrv>,
    mut env: BTreeMap<String, String>,
) -> Derivation {
    let builder_script_file = tmp_file(&src_dir, "builder.sh", builder_script);
    fs::set_permissions(&builder_script_file, fs::Permissions::from_mode(0o640)).unwrap();
    env.extend([("out".to_owned(), out_dir.to_str().unwrap().to_owned())]);

    Derivation {
        builder: builder.to_owned(),
        args: vec![builder_script_file.clone()],
        env: env,
        input_drvs: input_drvs,
        input_srcs: input_srcs
            .iter()
            .chain(&[builder_script_file.clone()])
            .cloned()
            .collect(),
        outputs: BTreeMap::from([(
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

fn get_derivation_path(installable_arg: &str) -> Option<String> {
    let mut nix_args = vec!["show-derivation".to_owned()];
    nix_args.push(installable_arg.to_owned());
    let show_drv_out = Command::new("nix")
        .args(&nix_args)
        .output()
        .expect("failed to show the derivation");

    let parsed_out: BTreeMap<String, Derivation> =
        serde_json::from_str(str::from_utf8(&show_drv_out.stdout).unwrap()).unwrap();

    parsed_out.keys().next().cloned()
}
