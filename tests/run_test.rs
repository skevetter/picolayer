mod common;

use common::{binary_exists, run_picolayer};
use serial_test::serial;

use crate::common::is_transient_error;

#[test]
#[serial]
fn test_run_python_version() {
    let output = run_picolayer(&["run", "python@3.11", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Python 3.11"));
}

#[test]
#[serial]
fn test_run_node_version() {
    let output = run_picolayer(&["run", "node@18", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("v18"));
}

#[test]
#[serial]
fn test_run_with_working_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_str().unwrap();
    let script_path = temp_dir.path().join("test_script.py");
    std::fs::write(&script_path, "print('Hello from script')").expect("Failed to write script");

    let output = run_picolayer(&[
        "run",
        "--working-dir",
        working_dir,
        "python",
        "test_script.py",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Hello from script"));
}

#[test]
#[serial]
fn test_run_dependency_detection() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let package_json = temp_dir.path().join("package.json");
    std::fs::write(&package_json, r#"{"name": "test", "version": "1.0.0"}"#)
        .expect("Failed to write package.json");
    let output = run_picolayer(&[
        "run",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "node",
        "--version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("v"));
}

#[test]
#[serial]
fn test_run_python_with_requirements() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let requirements_txt = temp_dir.path().join("requirements.txt");
    std::fs::write(&requirements_txt, "requests==2.28.0")
        .expect("Failed to write requirements.txt");

    let output = run_picolayer(&[
        "run",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "python",
        "--version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Python"));
}

#[test]
#[serial]
fn test_run_go_with_mod() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let go_mod = temp_dir.path().join("go.mod");
    std::fs::write(&go_mod, "module test\n\ngo 1.19").expect("Failed to write go.mod");

    let output = run_picolayer(&[
        "run",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "go",
        "version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("go version"));
}

#[test]
#[serial]
fn test_run_python_with_version_simple() {
    let output = run_picolayer(&["run", "python@3.10", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Python 3.10"));
}

#[test]
#[serial]
fn test_run_python_latest() {
    let output = run_picolayer(&["run", "python", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Python"));
}

#[test]
#[serial]
fn test_run_python_script() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("test.py");
    std::fs::write(&script_path, "print('Hello from Python!')").expect("Failed to write script");

    let output = run_picolayer(&[
        "run",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "python",
        script_path.to_str().unwrap(),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Hello from Python!"));
}

#[test]
#[serial]
fn test_run_node_with_version_simple() {
    let output = run_picolayer(&["run", "node@18", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("v18."));
}

#[test]
#[serial]
fn test_run_python_inline_code() {
    let output = run_picolayer(&["run", "python", "--", "-c", "print('Hello from Python!')"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Hello from Python!"));
}

#[test]
#[serial]
fn test_run_node_inline_code() {
    let output = run_picolayer(&[
        "run",
        "node",
        "--",
        "-e",
        "console.log('Hello from Node.js!')",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Hello from Node.js!"));
}

#[test]
#[serial]
fn test_run_go_with_version() {
    let output = run_picolayer(&["run", "go@1.21", "version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("go1.21"));
}

#[test]
#[serial]
fn test_run_ruby_inline() {
    let output = run_picolayer(&["run", "ruby", "-e", "puts 'Hello from Ruby!'"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Hello from Ruby!"));
}

#[test]
#[serial]
fn test_run_with_env_vars() {
    let output = run_picolayer(&[
        "run",
        "--env",
        "TEST_VAR=hello_world",
        "python",
        "--",
        "-c",
        "import os; print(f'TEST_VAR={os.environ.get(\"TEST_VAR\", \"not found\")}')",
    ]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Output: {}", stdout);

    assert!(stdout.contains("TEST_VAR=hello_world"));
}

#[test]
#[serial]
fn test_run_with_delete_none() {
    let output = run_picolayer(&["run", "--delete", "none", "bash@5.1", "-c", "echo 'hello world'"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("hello world"));
}

#[test]
#[serial]
fn test_run_with_delete_package() {
    let output = run_picolayer(&["run", "--delete", "package", "python", "-c", "print('test')"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("test"));
}

#[test]
#[serial]
fn test_run_with_delete_pkgx() {
    let output = run_picolayer(&["run", "--delete", "pkgx", "bash@5.1", "-c", "echo 'hello world'"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("hello world"));
}

#[test]
#[serial]
fn test_run_rust_with_version() {
    let output = run_picolayer(&["run", "rustc@1.70", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);
    println!("Output: {}", stderr);

    assert!(stdout.contains("rustc 1.70"));
}

#[test]
#[serial]
fn test_run_multiple_args() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    std::fs::write(&file1, "content1").expect("Failed to write file1");
    std::fs::write(&file2, "content2").expect("Failed to write file2");

    let output = run_picolayer(&[
        "run",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "python",
        "-c",
        &format!(
            "
import os
with open('{}', 'r') as f1, open('{}', 'r') as f2:
    print(f1.read().strip())
    print(f2.read().strip())
        ",
            file1.file_name().unwrap().to_str().unwrap(),
            file2.file_name().unwrap().to_str().unwrap()
        ),
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("content1"));
    assert!(stdout.contains("content2"));
}

#[test]
#[serial]
fn test_pkgx_xz_installation_end_to_end() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_picolayer"))
        .args([
            "gh-release",
            "pkgxdev/pkgx",
            "pkgx",
            "--version",
            "v2.7.0",
            "--install-dir",
            bin_location,
            "--filter",
            &format!("{}.*x86-64\\.tar\\.xz", os),
        ])
        .output()
        .expect("Failed to execute picolayer");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
            return;
        }
    };

    assert!(
        output.status.success(),
        "pkgx XZ installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let binary_path = format!("{}/pkgx", bin_location);
    assert!(
        binary_exists(&binary_path),
        "pkgx binary was not installed at {}",
        binary_path
    );
}
