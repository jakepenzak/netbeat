use assert_cmd::Command;
use predicates::prelude::*;
use std::{process::Command as stdCommand, thread};

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["--help"]);

    cmd.assert().success().stdout(predicate::str::contains(
        "A fast, minimal, & lightweight tool",
    ));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["--version"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("netbeat"));
}

#[test]
fn test_invalid_command_returns_error() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["invalid-command"]);

    cmd.assert().failure().code(2); // clap's default error code
}

#[test]
fn test_run_without_target_fails() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["run"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_run_with_invalid_target_fails() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["run", "invalid-host-that-does-not-exist"])
        .timeout(std::time::Duration::from_secs(5)); // Don't wait forever

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("❌"));
}

#[test]
fn test_serve_help() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["serve", "--help"]);

    cmd.assert().success().stdout(predicate::str::contains(
        "Start listening for incoming connections",
    ));
}

#[test]
fn test_run_help() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["run", "--help"]);

    cmd.assert().success().stdout(predicate::str::contains(
        "Run a speed test against a target server",
    ));
}

#[test]
fn test_json_flag_parsing() {
    // This won't actually run a test, but will validate that args parse correctly
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["run", "127.0.0.1", "--json", "--timeout", "1"])
        .timeout(std::time::Duration::from_secs(3));

    // We expect this to fail (no server running), but it should parse args correctly
    cmd.assert().failure().code(1);
}

#[test]
fn test_verbose_flag_parsing() {
    let mut cmd = Command::cargo_bin("netbeat").unwrap();
    cmd.args(["run", "127.0.0.1", "--verbose", "--timeout", "1"])
        .timeout(std::time::Duration::from_secs(3));

    cmd.assert().failure().code(1);
}

#[test]
fn test_client_server_flow_cli() {
    let mut serve_cmd = stdCommand::new("cargo")
        .args(["run", "--bin", "netbeat", "--", "serve", "-i", "all", "-q"])
        .spawn()
        .expect("Failed to start server.");

    thread::sleep(std::time::Duration::from_secs(2));

    // Retry logic in case server takes longer to start
    for i in 1..=10 {
        println!("Attempt {}", i);
        let mut run_cmd = Command::cargo_bin("netbeat").unwrap();
        run_cmd.args(["run", "0.0.0.0", "-t", "2", "--retries", "10"]);
        match run_cmd.ok() {
            Ok(_) => break,
            Err(e) => {
                if i == 10 {
                    serve_cmd.kill().expect("Failed to kill server");
                    serve_cmd.wait().expect("Failed to wait for server");
                    panic!("Failed to run client-server flow: {}", e);
                }
                thread::sleep(std::time::Duration::from_secs(i));
                continue;
            }
        }
    }

    serve_cmd.kill().expect("Failed to kill server");
    serve_cmd.wait().expect("Failed to wait for server");
}
