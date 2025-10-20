use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::feature::Feature;
use super::{DevcontainerFeatureConfig, client};

const ORDERED_BASE_USERS: &[&str] = &["vscode", "node", "codespace"];

pub async fn install_async(
    config: &DevcontainerFeatureConfig<'_>,
    retry_config: &crate::cli::RetryConfig,
) -> Result<()> {
    info!("Installing devcontainer feature: {}", config.feature_ref);

    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;

    info!("Downloading and extracting feature");
    client::download_and_extract_layer(
        config.feature_ref,
        temp_dir.path(),
        config.registry_username,
        config.registry_password,
        config.registry_token,
        retry_config,
    )
    .await?;

    let feature = load_feature_metadata(temp_dir.path())?;
    info!(
        "Feature: {} v{}",
        feature.id,
        feature.version.as_deref().unwrap_or("unknown")
    );

    let (remote_user_name, remote_user_home) =
        resolve_remote_user(config.remote_user.or(config.user))?;
    info!(
        "Installing for user: {} (home: {})",
        remote_user_name, remote_user_home
    );

    let resolved_options = feature.resolve_options(config.options.clone());
    debug!("Resolved options: {:?}", resolved_options);

    let mut env_vars = config.envs.clone().unwrap_or_default();
    env_vars.insert("_REMOTE_USER".to_string(), remote_user_name.clone());
    env_vars.insert("_REMOTE_USER_HOME".to_string(), remote_user_home.clone());

    for (key, value) in resolved_options {
        env_vars.insert(key.to_uppercase(), value);
    }

    execute_install_script(temp_dir.path(), &env_vars, config.script_name)?;
    set_container_env(&feature)?;
    execute_entrypoint(&feature)?;

    info!("Devcontainer feature installation completed successfully");
    Ok(())
}

fn load_feature_metadata(feature_dir: &Path) -> Result<Feature> {
    let metadata_path = feature_dir.join("devcontainer-feature.json");

    if !metadata_path.exists() {
        anyhow::bail!("Feature metadata file not found: devcontainer-feature.json");
    }

    let metadata_content =
        fs::read_to_string(&metadata_path).context("Failed to read feature metadata")?;

    let feature: Feature =
        serde_json::from_str(&metadata_content).context("Failed to parse feature metadata")?;

    Ok(feature)
}

fn resolve_remote_user(remote_user: Option<&str>) -> Result<(String, String)> {
    if let Some(user) = remote_user
        && let Ok(output) = Command::new("id").arg("-u").arg(user).output()
        && output.status.success()
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok((user.to_string(), home));
        }
        if let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg(format!("eval echo ~{}", user))
            .output()
            && output.status.success()
        {
            let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok((user.to_string(), home));
        }

        warn!("User '{}' not found, attempting fallback", user);
    }

    for user in ORDERED_BASE_USERS {
        if let Ok(output) = Command::new("id").arg("-u").arg(user).output()
            && output.status.success()
            && let Ok(output) = Command::new("sh")
                .arg("-c")
                .arg(format!("eval echo ~{}", user))
                .output()
            && output.status.success()
        {
            let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok((user.to_string(), home));
        }
    }

    // Fallback to user 1000
    if let Ok(output) = Command::new("id").arg("-un").arg("1000").output()
        && output.status.success()
    {
        let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg(format!("eval echo ~{}", user))
            .output()
            && output.status.success()
        {
            let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok((user, home));
        }
    }

    Ok(("root".to_string(), "/root".to_string()))
}

fn execute_install_script(
    feature_dir: &Path,
    env_vars: &HashMap<String, String>,
    script_name: &str,
) -> Result<()> {
    let install_script = feature_dir.join(script_name);
    if !install_script.exists() {
        anyhow::bail!("Feature script not found: {}", script_name);
    }

    info!("Executing feature script: {}", script_name);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&install_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_script, perms)?;
    }

    let env_string: Vec<String> = env_vars
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v.replace("\"", "\\\"")))
        .collect();

    let env_prefix = env_string.join(" ");
    let command = format!(
        "cd {} && {} bash -i +H -x ./{}",
        feature_dir.display(),
        env_prefix,
        script_name
    );

    debug!("Executing: {}", command);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .context("Failed to execute install script")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        warn!("Script output:\n{}", stdout);
        warn!("Script errors:\n{}", stderr);
        anyhow::bail!(
            "Feature installation script failed with exit code: {:?}",
            output.status.code()
        );
    }

    info!("Feature installation script completed successfully");
    Ok(())
}

fn set_container_env(feature: &Feature) -> Result<()> {
    const PROFILE_DIR: &str = "/etc/profile.d";

    if feature.container_env.is_none() {
        return Ok(());
    }

    let profile_dir = Path::new(PROFILE_DIR);
    fs::create_dir_all(profile_dir).context("Failed to create profile directory")?;

    let profile_file = profile_dir.join(format!("picolayer-{}.sh", feature.id));

    let mut content = String::new();
    if profile_file.exists() {
        content = fs::read_to_string(&profile_file)?;
    }

    if let Some(container_env) = &feature.container_env {
        for (key, value) in container_env {
            let statement = format!("export {}={}\n", key, value);
            if !content.contains(&statement) {
                content.push_str(&statement);
            }
        }
    }

    fs::write(&profile_file, content).context("Failed to write profile file")?;

    Ok(())
}

fn execute_entrypoint(feature: &Feature) -> Result<()> {
    if let Some(entrypoint) = &feature.entrypoint {
        info!("Executing feature entrypoint: {}", entrypoint);
        let output = Command::new("sh")
            .arg("-c")
            .arg(entrypoint)
            .output()
            .context("Failed to execute entrypoint")?;

        if !output.status.success() {
            warn!(
                "Entrypoint failed but continuing: {:?}",
                output.status.code()
            );
        }
    }
    Ok(())
}
