use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn lists_voices_without_input_file() {
    let mut cmd = Command::cargo_bin("txttattler").unwrap();
    cmd.arg("--list-voices")
        .assert()
        .success()
        .stdout(predicate::str::contains("alloy"))
        .stdout(predicate::str::contains("shimmer"));
}

#[test]
fn rejects_unsupported_speed() {
    let mut cmd = Command::cargo_bin("txttattler").unwrap();
    cmd.args(["sample.txt", "--speed", "9"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
