#![cfg(not(target_env = "musl"))]

use crate::common::run_picolayer_with_retry;
use serial_test::serial;
use std::process::Command;

#[test]
#[serial]
fn test_npm_installation() {
    let output = run_picolayer_with_retry(&["npm", "cowsay"]);

    assert!(
        output.status.success(),
        "npm installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify cowsay was installed and works
    let cowsay_output = Command::new("cowsay")
        .arg("Hello from npm!")
        .output()
        .expect("Failed to run cowsay");

    assert!(
        cowsay_output.status.success(),
        "cowsay execution failed: {}",
        String::from_utf8_lossy(&cowsay_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&cowsay_output.stdout);
    assert!(
        stdout.contains("Hello from npm!"),
        "cowsay output doesn't contain expected text: {}",
        stdout
    );
}

#[test]
#[serial]
fn test_npm_multiple_packages() {
    let output = run_picolayer_with_retry(&["npm", "cowsay,json"]);

    assert!(
        output.status.success(),
        "npm multiple packages installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify both packages were installed
    let cowsay_check = Command::new("cowsay")
        .arg("--version")
        .output()
        .expect("Failed to check cowsay");

    assert!(
        cowsay_check.status.success(),
        "cowsay not properly installed"
    );

    let json_check = Command::new("json")
        .arg("--version")
        .output()
        .expect("Failed to check json");

    assert!(json_check.status.success(), "json not properly installed");
}
