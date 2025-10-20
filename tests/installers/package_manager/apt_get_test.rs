#[cfg(all(target_os = "linux", not(target_env = "musl")))]
use crate::common::run_picolayer_with_retry;
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
use serial_test::serial;

#[test]
#[serial]
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
fn test_apt_get_installation() {
    // Perform update before installing any package to ensure lists are populated
    std::process::Command::new("sudo")
        .arg("apt-get")
        .arg("update")
        .status()
        .expect("Failed to run apt-get update");

    // Expect /var/lib/apt/lists to exist
    let lists_path = std::path::Path::new("/var/lib/apt/lists");
    assert!(lists_path.exists(), "/var/lib/apt/lists does not exist");

    // Expect files in the lists directory
    let entries = std::fs::read_dir(lists_path).unwrap();
    assert!(entries.count() > 0, "/var/lib/apt/lists is empty");

    let output = run_picolayer_with_retry(&["apt-get", "file"]);

    // Expect the command to succeed
    assert!(
        output.status.success(),
        "apt-get installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
