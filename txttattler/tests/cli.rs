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
        .stdout(predicate::str::contains("shimmer"))
        .stdout(predicate::str::contains("balanced and versatile"));
}

#[test]
fn invalid_voice_fails_during_argument_parsing() {
    Command::cargo_bin("txttattler")
        .unwrap()
        .args(["sample.txt", "--voice", "robot"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn invalid_model_fails_during_argument_parsing() {
    Command::cargo_bin("txttattler")
        .unwrap()
        .args(["sample.txt", "--model", "not-a-model"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn invalid_speed_fails_during_argument_parsing() {
    Command::cargo_bin("txttattler")
        .unwrap()
        .args(["sample.txt", "--speed", "9.0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Speed must be between 0.25 and 4.0",
        ));
}

#[test]
fn verbose_short_flag_is_accepted() {
    let tempdir = TempDir::new().unwrap();
    let input_path = tempdir.path().join("sample.txt");
    fs::write(&input_path, "hello").unwrap();

    Command::cargo_bin("txttattler")
        .unwrap()
        .args([
            input_path.to_str().unwrap(),
            "--no-play",
            "--no-cache",
            "--verbose",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Nothing to do"));
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

#[test]
fn no_play_no_cache_without_output_fails_before_credentials() {
    let tempdir = TempDir::new().unwrap();
    let input_path = tempdir.path().join("sample.txt");
    fs::write(&input_path, "hello").unwrap();

    Command::cargo_bin("txttattler")
        .unwrap()
        .env_remove("OPENAI_API_KEY")
        .args([input_path.to_str().unwrap(), "--no-play", "--no-cache"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Nothing to do"));
}
