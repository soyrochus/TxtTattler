use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn list_voices_prints_catalog() {
    Command::cargo_bin("txttattler")
        .unwrap()
        .arg("--list-voices")
        .assert()
        .success()
        .stdout(predicate::str::contains("alloy"))
        .stdout(predicate::str::contains("shimmer"));
}

#[test]
fn missing_input_file_surfaces_a_clear_error() {
    Command::cargo_bin("txttattler")
        .unwrap()
        .env("OPENAI_API_KEY", "test-key")
        .args(["definitely-missing.txt", "--no-play"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn explicit_config_path_is_respected() {
    let tempdir = TempDir::new().unwrap();
    let config_path = tempdir.path().join("txttattler.toml");
    let input_path = tempdir.path().join("sample.txt");
    fs::write(
        &config_path,
        "speed = 9.0\n[openai]\napi_key = \"fake-key\"\n",
    )
    .unwrap();
    fs::write(&input_path, "hello").unwrap();

    Command::cargo_bin("txttattler")
        .unwrap()
        .args([
            input_path.to_str().unwrap(),
            "--config",
            config_path.to_str().unwrap(),
            "--no-play",
            "--no-cache",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Speed must be between 0.25 and 4.0",
        ));
}
