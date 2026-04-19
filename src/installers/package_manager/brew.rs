use crate::utils;
use anyhow::Result;
use log::info;

pub(super) fn install(packages: &[String]) -> Result<()> {
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
    let mut cmd = std::process::Command::new("brew");
    cmd.arg("update");
    utils::subprocess::run_command(&mut cmd, "Update Homebrew")?;
    Ok(())
}

fn install_packages(packages: &[String]) -> Result<()> {
    info!("Installing Homebrew packages: {:?}", packages);
    let mut cmd = std::process::Command::new("brew");
    cmd.args(["install"]).args(packages);
    utils::subprocess::run_command(&mut cmd, "Install Homebrew packages")?;
    Ok(())
}

fn cleanup() -> Result<()> {
    info!("Cleaning up Homebrew cache");
    let mut cmd = std::process::Command::new("brew");
    cmd.arg("cleanup");
    utils::subprocess::run_command(&mut cmd, "Clean up Homebrew cache")?;
    Ok(())
}
