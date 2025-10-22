use crate::utils;
use anyhow::{Context, Result};
use log::info;

pub fn install(packages: &[String]) -> Result<()> {
    if std::process::Command::new("which")
        .arg("apk")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        anyhow::bail!("apk command not found in PATH");
    }

    update_repositories()?;
    install_packages(packages)?;
    cleanup()?;

    Ok(())
}

fn update_repositories() -> Result<()> {
    info!("Updating apk repositories");
    utils::sudo::command("apk")
        .args(["update"])
        .output()
        .context("Failed to update apk repositories")?;
    Ok(())
}

fn install_packages(packages: &[String]) -> Result<()> {
    info!("Installing apk packages: {:?}", packages);
    utils::sudo::command("apk")
        .args(["add", "--no-cache"])
        .args(packages)
        .output()
        .context("Failed to install apk packages")?;
    Ok(())
}

fn cleanup() -> Result<()> {
    info!("Cleaning up apk cache");
    utils::sudo::command("apk")
        .args(["cache", "clean"])
        .output()
        .context("Failed to clean apk cache")?;
    Ok(())
}
