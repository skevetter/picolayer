//! Common utilities for integration tests

use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Path to the picolayer binary for testing
#[allow(dead_code)]
pub const PICOLAYER_BIN: &str = env!("CARGO_BIN_EXE_picolayer");

/// Run picolayer with the given arguments and return the output
#[allow(dead_code)]
pub fn run_picolayer(args: &[&str]) -> std::process::Output {
    println!("=== Running picolayer with args: {:?} ===", args);

    let output = Command::new(PICOLAYER_BIN)
        .args(args)
        .output()
        .expect("Failed to execute picolayer");

    println!("Exit status: {}", output.status);
    println!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
    println!("=== End picolayer execution ===\n");

    output
}

/// Run picolayer as root using sudo
#[allow(dead_code)]
pub fn run_picolayer_as_root(args: &[&str]) -> std::process::Output {
    println!("=== Running picolayer with sudo and args: {:?} ===", args);

    let mut cmd = std::process::Command::new("sudo");
    cmd.arg(PICOLAYER_BIN);
    cmd.args(args);

    let output = cmd.output().expect("Failed to execute picolayer with sudo");

    println!(
        "Exit status: {} (code: {:?})",
        if output.status.success() {
            "SUCCESS"
        } else {
            "FAILED"
        },
        output.status.code()
    );
    println!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
    println!("=== End picolayer execution ===\n");

    output
}

/// Run picolayer with retry and exponential backoff for transient errors
#[allow(dead_code)]
pub fn run_picolayer_with_retry(args: &[&str]) -> std::process::Output {
    run_picolayer_with_retry_impl(args, false)
}

/// Run picolayer with sudo and retry for devcontainer features
#[allow(dead_code)]
pub fn run_picolayer_with_retry_as_root(args: &[&str]) -> std::process::Output {
    run_picolayer_with_retry_impl(args, true)
}

fn run_picolayer_with_retry_impl(args: &[&str], use_sudo: bool) -> std::process::Output {
    const MAX_RETRIES: u32 = 5;
    const BASE_DELAY_MS: u64 = 5000;

    for attempt in 0..MAX_RETRIES {
        println!("=== Attempt {}/{} ===", attempt + 1, MAX_RETRIES);
        let output = if use_sudo {
            run_picolayer_as_root(args)
        } else {
            run_picolayer(args)
        };

        if output.status.success() {
            return output;
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !is_transient_error(&stderr) {
            panic!("Non-transient error detected, not retrying");
        }

        if attempt < MAX_RETRIES - 1 {
            let delay = BASE_DELAY_MS * 2_u64.pow(attempt);
            println!(
                "Transient error detected, retrying in {}ms (attempt {}/{})",
                delay,
                attempt + 1,
                MAX_RETRIES
            );
            thread::sleep(Duration::from_millis(delay));
        }
    }

    // Return the last attempt's output
    println!("=== Final attempt ===");
    run_picolayer(args)
}

/// Check if an error message indicates a transient error that should be retried or ignored
#[allow(dead_code)]
pub fn is_transient_error(stderr: &str) -> bool {
    stderr.contains("403")
        || stderr.contains("500")
        || stderr.contains("rate limit")
        || stderr.contains("connection")
        || stderr.contains("timeout")
        || stderr.contains("network")
}

/// Check if a binary exists at the given path
#[allow(dead_code)]
pub fn binary_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Check if a binary exists and optionally verify it contains expected version info
#[allow(dead_code)]
pub fn check_binary_version(binary_path: &str, expected_substring: Option<&str>) -> bool {
    if !binary_exists(binary_path) {
        return false;
    }

    let output = Command::new(binary_path).arg("--version").output();

    if let Ok(output) = output {
        let version_str = String::from_utf8_lossy(&output.stdout);
        if let Some(expected) = expected_substring {
            version_str.contains(expected)
        } else {
            !version_str.is_empty()
        }
    } else {
        false
    }
}
