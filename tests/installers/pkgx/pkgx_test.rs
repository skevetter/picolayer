#![cfg(not(target_env = "musl"))]

use crate::common::run_picolayer_with_retry;
use anyhow::Result;
use serial_test::serial;
#[cfg(target_os = "linux")]
use std::env;
use std::path::PathBuf;

fn get_platform_pkgx_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    if let Some(home_dir) = dirs_next::home_dir() {
        #[cfg(target_os = "macos")]
        {
            for path in ["Library/Caches/pkgx", "Library/Application Support/pkgx"] {
                paths.push(home_dir.join(path));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(xdg_cache_home) = env::var("XDG_CACHE_HOME") {
                paths.push(PathBuf::from(xdg_cache_home).join("pkgx"));
            } else {
                paths.push(home_dir.join(".cache/pkgx"));
            }

            if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
                paths.push(PathBuf::from(xdg_data_home).join("pkgx"));
            } else {
                paths.push(home_dir.join(".local/share/pkgx"));
            }
        }
    }

    Ok(paths)
}

#[test]
#[serial]
fn test_pkgx_without_existing_pkgx_cache() {
    let paths = get_platform_pkgx_paths().unwrap_or_default();
    assert!(!paths.is_empty(), "No pkgx paths found for this platform");

    for path in &paths {
        if path.exists() {
            std::fs::remove_dir_all(path).ok();
        }
    }

    for path in &paths {
        assert!(!path.exists(), "Path should not exist: {}", path.display());
    }

    let output = run_picolayer_with_retry(&["pkgx", "--tool", "python", "--", "--version"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Python"),
        "Expected Python version output, got: {}",
        stdout
    );
}

#[test]
#[serial]
fn test_pkgx_python_version() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "python",
        "--version",
        "3.11",
        "--",
        "--version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python 3.11"));
}

#[test]
#[serial]
fn test_pkgx_node_version() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "node",
        "--version",
        "18",
        "--",
        "--version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v18"));
}

#[test]
#[serial]
fn test_pkgx_with_working_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_str().unwrap();

    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "python",
        "--working-dir",
        working_dir,
        "--",
        "-c",
        "import os; print(os.getcwd())",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(working_dir));
}

#[test]
#[serial]
fn test_pkgx_dependency_detection() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let package_json = temp_dir.path().join("package.json");
    std::fs::write(&package_json, r#"{"dependencies": {"lodash": "^4.17.21"}}"#)
        .expect("Failed to write package.json");

    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "node",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "--",
        "--version",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v"));
}

#[test]
#[serial]
fn test_pkgx_python_with_requirements() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let requirements_txt = temp_dir.path().join("requirements.txt");
    std::fs::write(&requirements_txt, "requests==2.28.1\n")
        .expect("Failed to write requirements.txt");

    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "python",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "--",
        "--version",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python"));
}

#[test]
#[serial]
fn test_pkgx_go_with_mod() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let go_mod = temp_dir.path().join("go.mod");
    std::fs::write(&go_mod, "module test\n\ngo 1.19\n").expect("Failed to write go.mod");

    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "go",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "--",
        "version",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("go version"));
}

#[test]
#[serial]
fn test_pkgx_python_with_version_simple() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "python",
        "--version",
        "3.11",
        "--",
        "--version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python"));
}

#[test]
#[serial]
fn test_pkgx_python_latest() {
    let output = run_picolayer_with_retry(&["pkgx", "--tool", "python", "--", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python"));
}

#[test]
#[serial]
fn test_pkgx_python_script() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("test.py");
    std::fs::write(&script_path, "print('Hello from Python!')").expect("Failed to write script");

    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "python",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "--",
        script_path.to_str().unwrap(),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Python!"));
}

#[test]
#[serial]
fn test_pkgx_node_with_version_simple() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "node",
        "--version",
        "18",
        "--",
        "--version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v18."));
}

#[test]
#[serial]
fn test_pkgx_python_inline_code() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "python",
        "--",
        "-c",
        "print('Hello from Python!')",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Python!"));
}

#[test]
#[serial]
fn test_pkgx_node_inline_code() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "node",
        "--",
        "-e",
        "console.log('Hello from Node.js!')",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Node.js!"));
}

#[test]
#[serial]
fn test_pkgx_go_with_version() {
    let output =
        run_picolayer_with_retry(&["pkgx", "--tool", "go", "--version", "1.21", "--", "version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("go version"));
}

#[test]
#[serial]
fn test_pkgx_ruby_inline() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "ruby",
        "--",
        "-e",
        "puts 'Hello from Ruby!'",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Ruby!"));
}

#[test]
#[serial]
fn test_pkgx_with_env_vars() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "python",
        "--env",
        "TEST_VAR=hello",
        "--",
        "-c",
        "import os; print(os.environ.get('TEST_VAR', 'not found'))",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"));
}

#[test]
#[serial]
fn test_pkgx_rust_with_version() {
    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "rustc",
        "--version",
        "1.70",
        "--",
        "--version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rustc"));
}

#[test]
#[serial]
fn test_pkgx_multiple_args() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    std::fs::write(&file1, "content1").expect("Failed to write file1");
    std::fs::write(&file2, "content2").expect("Failed to write file2");

    let output = run_picolayer_with_retry(&[
        "pkgx",
        "--tool",
        "python",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "--",
        "-c",
        "
import os
with open('file1.txt', 'r') as f1, open('file2.txt', 'r') as f2:
    print(f1.read().strip())
    print(f2.read().strip())
        ",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("content1"));
    assert!(stdout.contains("content2"));
}
