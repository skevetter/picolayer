use anyhow::{Context, Result};
use log::warn;
use std::process::{Command, Output};

/// Run a command, check its exit status, and log stderr/stdout on failure.
pub fn run_command(cmd: &mut Command, description: &str) -> Result<Output> {
    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute: {}", description))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            warn!("{} stdout:\n{}", description, stdout.trim());
        }
        if !stderr.is_empty() {
            warn!("{} stderr:\n{}", description, stderr.trim());
        }
        anyhow::bail!(
            "{} failed with exit code: {:?}",
            description,
            output.status.code()
        );
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_command_succeeds_on_true() {
        let output = run_command(&mut Command::new("true"), "true command").unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn run_command_fails_on_false() {
        let result = run_command(&mut Command::new("false"), "false command");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("false command failed with exit code"));
    }

    #[test]
    fn run_command_captures_stderr_in_error() {
        let result = run_command(
            Command::new("sh").args(["-c", "echo err >&2; exit 1"]),
            "stderr test",
        );
        assert!(result.is_err());
    }

    #[test]
    fn run_command_returns_output_on_success() {
        let output =
            run_command(Command::new("sh").args(["-c", "echo hello"]), "echo test").unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"));
    }
}
