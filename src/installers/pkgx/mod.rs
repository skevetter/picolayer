mod resolver;

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::Path;
use std::{collections::HashMap, env};
use tempfile::TempDir;

pub struct PkgxConfig<'a> {
    pub tool: &'a str,
    pub version: &'a str,
    pub args: Vec<String>,
    pub working_dir: &'a str,
    pub env_vars: Vec<String>,
}

struct PkgxEnv {
    pkgx_dir: String,
    pantry_dir: String,
    _temp_dir: TempDir,
}

impl PkgxEnv {
    fn new() -> Result<Self> {
        let temp_dir =
            TempDir::with_prefix("picolayer_").context("Failed to create temporary directory")?;

        let pkgx_dir = temp_dir.path().join("pkgx").join("tools");
        let pantry_dir = temp_dir.path().join("pkgx").join("pantry");

        std::fs::create_dir_all(&pkgx_dir).context("Failed to create pkgx directory")?;
        std::fs::create_dir_all(&pantry_dir).context("Failed to create pantry directory")?;

        Ok(Self {
            pkgx_dir: pkgx_dir
                .to_str()
                .context("Failed to convert pkgx directory path to string")?
                .to_string(),
            pantry_dir: pantry_dir
                .to_str()
                .context("Failed to convert pantry directory path to string")?
                .to_string(),
            _temp_dir: temp_dir,
        })
    }
}

pub fn execute(input: &PkgxConfig) -> Result<()> {
    validate_working_directory(input.working_dir)?;
    debug!("Working directory: {}", input.working_dir);
    debug!("Tool: {} ({})", input.tool, input.version);
    debug!("Command: {}", input.args.join(" "));

    let env_map = parse_env_vars(&input.env_vars)?;
    let exec_env = PkgxEnv::new()?;

    debug!("Using pkgx virtual environment: {}", exec_env.pkgx_dir);
    debug!("Using pantry directory: {}", exec_env.pantry_dir);

    let working_path = Path::new(input.working_dir);

    // Store original environment variables
    let original_pkgx_dir = env::var("PKGX_DIR").ok();
    let original_pkgx_pantry_dir = env::var("PKGX_PANTRY_DIR").ok();

    // Set temporary environment variables
    unsafe {
        env::set_var("PKGX_DIR", &exec_env.pkgx_dir);
        env::set_var("PKGX_PANTRY_DIR", &exec_env.pantry_dir);
    }

    let result = execute_with_pkgx_library(
        input.tool,
        input.version,
        &input.args,
        working_path,
        &env_map,
        &exec_env,
    );

    // Restore original environment variables
    unsafe {
        match original_pkgx_dir {
            Some(val) => env::set_var("PKGX_DIR", val),
            None => env::remove_var("PKGX_DIR"),
        }
        match original_pkgx_pantry_dir {
            Some(val) => env::set_var("PKGX_PANTRY_DIR", val),
            None => env::remove_var("PKGX_PANTRY_DIR"),
        }
    }

    result
}

fn validate_working_directory(working_dir: &str) -> Result<()> {
    let working_path = Path::new(working_dir);
    if !working_path.exists() {
        anyhow::bail!("Working directory does not exist: {}", working_dir);
    }
    Ok(())
}

fn parse_env_vars(env_vars: &[String]) -> Result<Vec<(String, String)>> {
    env_vars
        .iter()
        .map(|env_var| {
            env_var
                .split_once('=')
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .context(format!(
                    "Invalid environment variable format: {} (expected key=value)",
                    env_var
                ))
        })
        .collect()
}

fn create_command_env(
    env_map: &[(String, String)],
    pkgx_dir: &str,
    pantry_dir: &str,
) -> HashMap<String, String> {
    let mut cmd_env: HashMap<String, String> = env::vars().collect();

    // https://docs.pkgx.sh/pkgx/pkgx#virtual-environments
    cmd_env.insert("PKGX_DIR".to_string(), pkgx_dir.to_string());

    // Set pantry directory for complete isolation
    cmd_env.insert("PKGX_PANTRY_DIR".to_string(), pantry_dir.to_string());

    // User provided environment variables
    for (key, value) in env_map {
        cmd_env.insert(key.clone(), value.clone());
    }

    cmd_env
}

fn execute_with_pkgx_library(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    exec_env: &PkgxEnv,
) -> Result<()> {
    info!("Using pkgx library integration with virtual environment");

    let project_name = resolver::resolve_tool_to_project(tool_name)
        .context("Failed to resolve tool to project using pkgx")?;

    let tool_spec = resolver::format_tool_spec(&project_name, version_spec);
    info!("Resolving package: {}", tool_spec);

    match resolver::resolve_package_with_libpkgx(&[tool_spec]) {
        Ok((pkgx_env, installations)) => {
            let mut cmd_env = create_command_env(env_map, &exec_env.pkgx_dir, &exec_env.pantry_dir);
            cmd_env.extend(pkgx_env);

            log_installations(&installations, &project_name, tool_name);

            debug!("Resolved package with libpkgx");
            let status = std::process::Command::new(tool_name)
                .args(args)
                .current_dir(working_path.to_str().context("Invalid working directory")?)
                .envs(&cmd_env)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .context("Failed to execute command with libpkgx")?;

            if status.success() {
                debug!("Command executed successfully with pkgx library!");
                Ok(())
            } else {
                anyhow::bail!("Command failed with exit code: {:?}", status.code());
            }
        }
        Err(e) => {
            if resolver::check_pkgx_binary() {
                warn!(
                    "Failed to resolve package with libpkgx, falling back to pkgx binary: {}",
                    e
                );
                execute_with_pkgx_binary(
                    tool_name,
                    version_spec,
                    args,
                    working_path,
                    env_map,
                    exec_env,
                )
            } else {
                anyhow::bail!(
                    "Failed to resolve package with libpkgx and no pkgx binary available: {}",
                    e
                );
            }
        }
    }
}

fn log_installations(
    installations: &[libpkgx::types::Installation],
    project_name: &str,
    tool_name: &str,
) {
    for installation in installations {
        if installation.pkg.project == project_name {
            info!("Package installed at: {}", installation.path.display());

            for bin_dir in ["bin", "sbin"] {
                let executable_path = installation.path.join(bin_dir).join(tool_name);
                if executable_path.exists() {
                    info!("Executable found at: {}", executable_path.display());
                    break;
                }
            }
        }
    }
}

fn execute_with_pkgx_binary(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    exec_env: &PkgxEnv,
) -> Result<()> {
    if !resolver::check_pkgx_binary() {
        anyhow::bail!("pkgx is not available. Install pkgx from https://pkgx.sh.");
    }

    let project_name = resolver::resolve_tool_to_project(tool_name)
        .context("Failed to resolve tool to project using pkgx")?;

    let project_arg = resolver::format_project_arg(&project_name, version_spec);

    info!("Using pkgx binary with virtual environment");

    let mut cmd = std::process::Command::new("pkgx");
    cmd.arg(&project_arg)
        .arg(tool_name)
        .args(args)
        .current_dir(working_path.to_str().context("Invalid working directory")?)
        .env("PKGX_DIR", &exec_env.pkgx_dir)
        .env("PKGX_PANTRY_DIR", &exec_env.pantry_dir)
        .envs(env_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));

    let status = cmd
        .status()
        .context("Failed to execute command with pkgx")?;

    if status.success() {
        info!("Command executed successfully with pkgx binary!");
        Ok(())
    } else {
        anyhow::bail!("Command failed with exit code: {:?}", status.code());
    }
}
