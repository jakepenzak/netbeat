use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("--help");

    cmd.assert().success().stdout(predicate::str::contains(
        "A fast, minimal, & lightweight Rust tool",
    ));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("netbeat"));
}

#[test]
fn test_invalid_command_returns_error() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("invalid-command");

    cmd.assert().failure().code(2); // clap's default error code
}

#[test]
fn test_run_without_target_fails() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("run");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_run_with_invalid_target_fails() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("run")
        .arg("invalid-host-that-does-not-exist")
        .timeout(std::time::Duration::from_secs(5)); // Don't wait forever

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("❌"));
}

#[test]
fn test_serve_help() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("serve").arg("--help");

    cmd.assert().success().stdout(predicate::str::contains(
        "Start listening for incoming connections",
    ));
}

#[test]
fn test_run_help() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("run").arg("--help");

    cmd.assert().success().stdout(predicate::str::contains(
        "Run a speed test against a target server",
    ));
}

#[test]
fn test_json_flag_parsing() {
    // This won't actually run a test, but will validate that args parse correctly
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("run")
        .arg("127.0.0.1") // Use localhost to avoid network issues
        .arg("--json")
        .arg("--timeout")
        .arg("1") // Short timeout so it fails quickly
        .timeout(std::time::Duration::from_secs(3));

    // We expect this to fail (no server running), but it should parse args correctly
    cmd.assert().failure().code(1);
}

#[test]
fn test_verbose_flag_parsing() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.arg("run")
        .arg("127.0.0.1")
        .arg("--verbose")
        .arg("--timeout")
        .arg("1")
        .timeout(std::time::Duration::from_secs(3));

    cmd.assert().failure().code(1);
}
