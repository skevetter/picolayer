#[cfg(target_env = "musl")]
use crate::common::run_picolayer_with_retry;
#[cfg(target_env = "musl")]
use serial_test::serial;

#[test]
#[serial]
#[cfg(target_env = "musl")]
fn test_apk_installation() {
    let output = run_picolayer_with_retry(&["apk", "curl"]);

    assert!(
        output.status.success(),
        "apk installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[serial]
#[cfg(target_env = "musl")]
fn test_apk_multiple_packages() {
    let output = run_picolayer_with_retry(&["apk", "curl,git"]);

    assert!(
        output.status.success(),
        "apk multiple packages installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
