#![cfg(not(target_env = "musl"))]

use crate::common::{run_picolayer, run_picolayer_with_retry, run_picolayer_with_retry_as_root};
use serial_test::serial;

#[test]
#[serial]
fn test_devcontainer_feature_help() {
    let output = run_picolayer_with_retry(&["devcontainer-feature", "--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("devcontainer-feature"));
    assert!(stdout.contains("OCI feature reference"));
}

#[test]
#[serial]
fn test_devcontainer_feature_invalid_reference() {
    let output = run_picolayer_with_retry(&[
        "devcontainer-feature",
        "invalid-reference-that-does-not-exist",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("Failed") || stderr.contains("Not authorized"),
        "Should provide error message for invalid reference: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_devcontainer_feature_bash_installation() {
    let output = run_picolayer_with_retry_as_root(&[
        "devcontainer-feature",
        "ghcr.io/devcontainers-extra/features/bash-command:1",
        "--option",
        "command=echo 'test successful' > /tmp/bash_test.txt",
    ]);

    if let Ok(content) = std::fs::read_to_string("/tmp/bash_test.txt") {
        assert_eq!(
            content.trim(),
            "test successful",
            "Command should have executed successfully"
        );
    }
    let _ = std::fs::remove_file("/tmp/bash_test.txt");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Feature installation should succeed: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_devcontainer_feature_black_installation() {
    let output = run_picolayer_with_retry_as_root(&[
        "devcontainer-feature",
        "ghcr.io/devcontainers-extra/features/black:2",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Feature installation should succeed: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_devcontainer_feature_with_options() {
    let output = run_picolayer_with_retry_as_root(&[
        "devcontainer-feature",
        "ghcr.io/devcontainers/features/common-utils:2",
        "--option",
        "installZsh=false",
        "--option",
        "installOhMyZsh=false",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Feature installation should succeed: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_devcontainer_feature_with_env_vars() {
    let output = run_picolayer_with_retry_as_root(&[
        "devcontainer-feature",
        "ghcr.io/devcontainers/features/common-utils:2",
        "--env",
        "TEST_VAR=test_value",
        "--remote-user",
        "vscode",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Feature installation should succeed: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_devcontainer_feature_custom_script() {
    let output = run_picolayer_with_retry_as_root(&[
        "devcontainer-feature",
        "ghcr.io/devcontainers/features/common-utils:2",
        "--script",
        "install.sh",
        "--user",
        "root",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Feature installation should succeed: {}",
        stderr
    );
}
#[test]
#[serial]
fn test_devcontainer_feature_retry_functionality() {
    let output = run_picolayer(&[
        "--max-retries",
        "1",
        "--retry-delay-ms",
        "100",
        "devcontainer-feature",
        "invalid-retry-test-reference",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("failed (attempt 1/2)") || stderr.contains("retrying in 100ms"),
        "Should show retry attempts in logs: {}",
        stderr
    );
    assert!(
        stderr.contains("failed after 2 attempts") || stderr.contains("Error:"),
        "Should show final failure after retries: {}",
        stderr
    );
}
#[test]
#[serial]
fn test_devcontainer_feature_no_retry_by_default() {
    let output = run_picolayer(&["devcontainer-feature", "invalid-no-retry-test-reference"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("retrying") && !stderr.contains("attempt"),
        "Should not show retry attempts by default: {}",
        stderr
    );

    assert!(
        stderr.contains("Error:"),
        "Should show error without retries: {}",
        stderr
    );
}
