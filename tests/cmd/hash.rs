use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn help() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["hash", "--help"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn to_base16_help() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["hash", "to-base16", "--help"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn md5_to_base16() {
    Command::cargo_bin("rix")
        .unwrap()
        .args([
            "hash",
            "to-base16",
            "--type=md5",
            "61h2nin3nx3lj7vj2ywixsiv5y",
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff("beeca87be45ec87d241ddd0e1bad80c1\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn sha1_to_base16() {
    Command::cargo_bin("rix")
        .unwrap()
        .args([
            "hash",
            "to-base16",
            "--type=sha1",
            "1upkmUx5+XtipytCb75gVqGUu5A=",
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(
            "d6ea64994c79f97b62a72b426fbe6056a194bb90\n",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn sha256_to_base16() {
    Command::cargo_bin("rix")
        .unwrap()
        .args([
            "hash",
            "to-base16",
            "--type=sha256",
            "1TE4YoVvd3C9/+0t/oxBeoTz9tXhHDtcGUIPITB2b4E=",
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(
            "d5313862856f7770bdffed2dfe8c417a84f3f6d5e11c3b5c19420f2130766f81\n",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn sha512_to_base16() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["hash", "to-base16", "--type=sha512", "+y4ZnePpvWs1fc/LhZRTHkTesbXkyBYuOB+5CyodZqrEuETXi3zOVfpAQIdgC3lXbHLTDG9dQosxR9BhvLKDLQ=="])
        .assert()
        .success()
        .stdout(predicate::str::diff("fb2e199de3e9bd6b357dcfcb8594531e44deb1b5e4c8162e381fb90b2a1d66aac4b844d78b7cce55fa404087600b79576c72d30c6f5d428b3147d061bcb2832d\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn sha512_to_base32() {
    Command::cargo_bin("rix")
        .unwrap()
        .args([
            "hash",
            "to-base32",
            "--type=sha512",
            "+y4ZnePpvWs1fc/LhZRTHkTesbXkyBYuOB+5CyodZqrEuETXi3zOVfpAQIdgC3lXbHLTDG9dQosxR9BhvLKDLQ==",
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff("0nq7cmwc784fccb89fny36kf9n5fy8bc23l0h7sap77r2yp8jwc9ak63lm0pf8z70p1dj74nnqxwi0yafa8bjygglsnpgg9wffijbpv\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn sha512_to_base64() {
    Command::cargo_bin("rix")
        .unwrap()
        .args([
            "hash",
            "to-base64",
            "--type=sha512",
            "0nq7cmwc784fccb89fny36kf9n5fy8bc23l0h7sap77r2yp8jwc9ak63lm0pf8z70p1dj74nnqxwi0yafa8bjygglsnpgg9wffijbpv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff("+y4ZnePpvWs1fc/LhZRTHkTesbXkyBYuOB+5CyodZqrEuETXi3zOVfpAQIdgC3lXbHLTDG9dQosxR9BhvLKDLQ==\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn to_base16_invalid_sri() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["hash", "to-base16", "61h2nin3nx3lj7vj2ywixsiv5y"])
        .assert()
        .failure()
        .stderr(predicate::str::diff(
            "error: Failed to parse '61h2nin3nx3lj7vj2ywixsiv5y'. Not an SRI hash.\n",
        ));
}

#[test]
fn to_base16_sri_unknown_hash_type() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["hash", "to-base16", "foobar-61h2nin3nx3lj7vj2ywixsiv5y"])
        .assert()
        .failure()
        .stderr(predicate::str::diff(
            "error: Hash type 'foobar' not supported.\n",
        ));
}

#[test]
fn to_base16_sri_md5() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["hash", "to-base16", "md5-61h2nin3nx3lj7vj2ywixsiv5y"])
        .assert()
        .success()
        .stdout(predicate::str::diff("beeca87be45ec87d241ddd0e1bad80c1\n"));
}

#[test]
fn to_base32_sri_sha1() {
    Command::cargo_bin("rix")
        .unwrap()
        .args(["hash", "to-base32", "sha1-1upkmUx5+XtipytCb75gVqGUu5A="])
        .assert()
        .success()
        .stdout(predicate::str::diff("j2xr98anc2z6yhiblxi7pybr9jcn9snn\n"));
}

#[test]
fn to_base64_sri_sha256() {
    Command::cargo_bin("rix")
        .unwrap()
        .args([
            "hash",
            "to-base64",
            "sha256-10bgfqq223s235f3n771spvg713s866gwbgdzyyp0xvghmi3hcfm",
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff(
            "1TE4YoVvd3C9/+0t/oxBeoTz9tXhHDtcGUIPITB2b4E=\n",
        ));
}

#[test]
fn to_base32_sri_sha512() {
    Command::cargo_bin("rix")
        .unwrap()
        .args([
            "hash",
            "to-base32",
            "sha512:+y4ZnePpvWs1fc/LhZRTHkTesbXkyBYuOB+5CyodZqrEuETXi3zOVfpAQIdgC3lXbHLTDG9dQosxR9BhvLKDLQ==",
        ])
        .assert()
        .success()
        .stdout(predicate::str::diff("0nq7cmwc784fccb89fny36kf9n5fy8bc23l0h7sap77r2yp8jwc9ak63lm0pf8z70p1dj74nnqxwi0yafa8bjygglsnpgg9wffijbpv\n"));
}
