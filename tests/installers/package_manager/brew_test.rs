#[cfg(target_os = "macos")]
use crate::common::run_picolayer_with_retry;

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "macos")]
use serial_test::serial;

#[test]
#[serial]
#[cfg(target_os = "macos")]
fn test_brew_installation() {
    let has_brew = Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_brew {
        eprintln!("Skipping brew test: Homebrew not available");
        return;
    }

    let output = run_picolayer_with_retry(&["brew", "jq"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Homebrew is not available"),
        "Brew command failed to detect Homebrew"
    );
}

#[test]
#[serial]
#[cfg(target_os = "macos")]
fn test_brew_multiple_packages() {
    let has_brew = Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_brew {
        eprintln!("Skipping brew multiple packages test: Homebrew not available");
        return;
    }

    run_picolayer_with_retry(&["brew", "jq,tree"]);
}
