use anyhow::{Context, Result};
use log::info;

pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        which::which("brew").is_ok(),
        "Homebrew not installed or not in PATH"
    );

    update()?;
    install_packages(packages)?;
    cleanup()?;

    Ok(())
}

fn update() -> Result<()> {
    info!("Updating Homebrew");
    std::process::Command::new("brew")
        .arg("update")
        .output()
        .context("Failed to update Homebrew")?;
    Ok(())
}

fn install_packages(packages: &[String]) -> Result<()> {
    info!("Installing Homebrew packages: {:?}", packages);
    std::process::Command::new("brew")
        .args(["install"])
        .args(packages)
        .output()
        .context("Failed to install Homebrew packages")?;
    Ok(())
}

fn cleanup() -> Result<()> {
    info!("Cleaning up Homebrew cache");
    std::process::Command::new("brew")
        .arg("cleanup")
        .output()
        .context("Failed to clean up Homebrew cache")?;
    Ok(())
}
