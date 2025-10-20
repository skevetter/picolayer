#![cfg(not(target_env = "musl"))]

use crate::common::run_picolayer_with_retry;
use serial_test::serial;
use std::process::Command;

#[test]
#[serial]
fn test_pipx_installation() {
    let output = run_picolayer_with_retry(&["pipx", "pycowsay"]);

    assert!(
        output.status.success(),
        "pipx installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify pycowsay was installed and works
    let cowsay_output = Command::new("pycowsay")
        .arg("Hello from pipx!")
        .output()
        .expect("Failed to run pycowsay");

    assert!(
        cowsay_output.status.success(),
        "pycowsay execution failed: {}",
        String::from_utf8_lossy(&cowsay_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&cowsay_output.stdout);
    assert!(
        stdout.contains("Hello from pipx!"),
        "pycowsay output doesn't contain expected text: {}",
        stdout
    );
}

#[test]
#[serial]
fn test_pipx_multiple_packages() {
    let output = run_picolayer_with_retry(&["pipx", "pycowsay,httpie"]);

    assert!(
        output.status.success(),
        "pipx multiple packages installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify both packages were installed
    let cowsay_check = Command::new("pycowsay")
        .arg("--version")
        .output()
        .expect("Failed to check pycowsay");

    assert!(
        cowsay_check.status.success(),
        "pycowsay not properly installed"
    );

    let http_check = Command::new("http")
        .arg("--version")
        .output()
        .expect("Failed to check httpie");

    assert!(http_check.status.success(), "httpie not properly installed");
}
