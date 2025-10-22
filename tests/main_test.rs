mod common;

use crate::common::run_picolayer;
use serial_test::serial;

#[test]
#[serial]
fn test_main_help() {
    let output = run_picolayer(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("picolayer"));
    assert!(stdout.contains("Ensures minimal container layers"));
}

#[test]
#[serial]
fn test_main_version() {
    let output = run_picolayer(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("picolayer"));
}

#[test]
#[serial]
fn test_apt_get_help() {
    let output = run_picolayer(&["apt-get", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("apt-get"));
}

#[test]
#[serial]
fn test_apk_help() {
    let output = run_picolayer(&["apk", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("apk"));
}

#[test]
#[serial]
fn test_brew_help() {
    let output = run_picolayer(&["brew", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("brew"));
}

#[test]
#[serial]
fn test_gh_release_help() {
    let output = run_picolayer(&["gh-release", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("gh-release"));
}

#[test]
#[serial]
fn test_run_help() {
    let output = run_picolayer(&["pkgx", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pkgx"));
}

#[test]
#[serial]
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
fn test_error_handling_github_not_found() {
    let output = run_picolayer(&[
        "gh-release",
        "--owner",
        "nonexistent-owner-12345",
        "--repo",
        "nonexistent-repo-12345",
        "--binary",
        "test",
        "--install-dir",
        "/tmp",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("Repository not found or not accessible"),
        "Should show user-friendly GitHub error: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_error_handling_devcontainer_feature() {
    let output = run_picolayer(&["devcontainer-feature", "invalid-feature-reference-12345"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("Failed to download container feature"),
        "Should show user-friendly devcontainer feature error: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_error_handling_shows_debug_hint() {
    let output = run_picolayer(&["devcontainer-feature", "invalid-feature-reference-12345"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("For technical details, set PICOLAYER_DEBUG=1"),
        "Should not show debug hint for known errors: {}",
        stderr
    );
}
