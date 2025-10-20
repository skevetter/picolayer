use crate::utils;
use anyhow::{Context, Result};
use log::{debug, info};
use std::process::Command;

pub fn install(packages: &[String]) -> Result<()> {
    ensure_npm_available()?;
    install_packages(packages)?;
    Ok(())
}

fn ensure_npm_available() -> Result<()> {
    if Command::new("npm").arg("--version").output().is_ok() {
        debug!("npm is already available");
        return Ok(());
    }

    info!("npm not found, installing Node.js");
    install_nodejs()?;
    Ok(())
}

fn install_nodejs() -> Result<()> {
    if utils::os::is_debian_like() {
        install_nodejs_debian()
    } else if utils::os::is_alpine() {
        install_nodejs_alpine()
    } else {
        anyhow::bail!("Unsupported OS for automatic Node.js installation")
    }
}

fn install_nodejs_debian() -> Result<()> {
    debug!("Installing Node.js on Debian-like system");
    utils::sudo::command("apt-get")
        .args(["update"])
        .output()
        .context("Failed to update package lists")?;

    utils::sudo::command("apt-get")
        .args(["install", "-y", "nodejs", "npm"])
        .output()
        .context("Failed to install Node.js and npm")?;

    Ok(())
}

fn install_nodejs_alpine() -> Result<()> {
    debug!("Installing Node.js on Alpine Linux");
    utils::sudo::command("apk")
        .args(["add", "nodejs", "npm"])
        .output()
        .context("Failed to install Node.js and npm")?;

    Ok(())
}

fn install_packages(packages: &[String]) -> Result<()> {
    debug!("Installing npm packages: {:?}", packages);

    let mut cmd = Command::new("npm");
    cmd.args(["install", "-g"]);
    cmd.args(packages);

    cmd.output().context("Failed to install npm packages")?;

    info!("Successfully installed npm packages: {:?}", packages);
    Ok(())
}
