#![cfg(not(target_env = "musl"))]

use crate::common::run_picolayer_with_retry;
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
fn test_devcontainer_feature_with_options() {
    let output = run_picolayer_with_retry(&[
        "devcontainer-feature",
        "ghcr.io/devcontainers/features/common-utils:2",
        "--option",
        "installZsh=false",
        "--option",
        "installOhMyZsh=false",
    ]);

    // This test may fail due to network/registry issues, but should not crash
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic") && !stderr.contains("thread panicked"),
        "Should not panic on feature installation: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_devcontainer_feature_with_env_vars() {
    let output = run_picolayer_with_retry(&[
        "devcontainer-feature",
        "ghcr.io/devcontainers/features/common-utils:2",
        "--env",
        "TEST_VAR=test_value",
        "--remote-user",
        "vscode",
    ]);

    // This test may fail due to network/registry issues, but should not crash
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic") && !stderr.contains("thread panicked"),
        "Should not panic with environment variables: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_devcontainer_feature_custom_script() {
    let output = run_picolayer_with_retry(&[
        "devcontainer-feature",
        "ghcr.io/devcontainers/features/common-utils:2",
        "--script",
        "install.sh",
        "--user",
        "root",
    ]);

    // This test may fail due to network/registry issues, but should not crash
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic") && !stderr.contains("thread panicked"),
        "Should not panic with custom script: {}",
        stderr
    );
}
#[test]
#[serial]
fn test_devcontainer_feature_retry_functionality() {
    use crate::common::run_picolayer;

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

    // Should show retry attempts in the logs
    assert!(
        stderr.contains("failed (attempt 1/2)") || stderr.contains("retrying in 100ms"),
        "Should show retry attempts in logs: {}",
        stderr
    );

    // Should eventually fail after retries
    assert!(
        stderr.contains("failed after 2 attempts") || stderr.contains("Error:"),
        "Should show final failure after retries: {}",
        stderr
    );
}
#[test]
#[serial]
fn test_devcontainer_feature_no_retry_by_default() {
    use crate::common::run_picolayer;

    let output = run_picolayer(&["devcontainer-feature", "invalid-no-retry-test-reference"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should NOT show retry attempts (no retries by default)
    assert!(
        !stderr.contains("retrying") && !stderr.contains("attempt"),
        "Should not show retry attempts by default: {}",
        stderr
    );

    // Should fail immediately
    assert!(
        stderr.contains("Error:"),
        "Should show error without retries: {}",
        stderr
    );
}
