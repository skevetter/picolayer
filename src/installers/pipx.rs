use crate::utils;
use anyhow::{Context, Result};
use log::{debug, info};
use std::process::Command;

pub fn install(packages: &[String], python_version: Option<&str>) -> Result<()> {
    ensure_pipx_available()?;
    install_packages(packages, python_version)?;
    Ok(())
}

fn ensure_pipx_available() -> Result<()> {
    if Command::new("pipx").arg("--version").output().is_ok() {
        debug!("pipx is already available");
        return Ok(());
    }

    info!("pipx not found, installing pipx");
    install_pipx()?;
    Ok(())
}

fn install_pipx() -> Result<()> {
    if utils::os::is_debian_like() {
        install_pipx_debian()
    } else if utils::os::is_alpine() {
        install_pipx_alpine()
    } else {
        anyhow::bail!("Unsupported OS for automatic pipx installation")
    }
}

fn install_pipx_debian() -> Result<()> {
    debug!("Installing pipx on Debian-like system");
    utils::sudo::command("apt-get")
        .args(["update"])
        .output()
        .context("Failed to update package lists")?;

    utils::sudo::command("apt-get")
        .args(["install", "-y", "pipx"])
        .output()
        .context("Failed to install pipx")?;

    Ok(())
}

fn install_pipx_alpine() -> Result<()> {
    debug!("Installing pipx on Alpine Linux");
    utils::sudo::command("apk")
        .args(["add", "py3-pip", "python3"])
        .output()
        .context("Failed to install Python and pip")?;

    Command::new("pip3")
        .args(["install", "--user", "pipx"])
        .output()
        .context("Failed to install pipx via pip")?;

    Ok(())
}

fn install_packages(packages: &[String], python_version: Option<&str>) -> Result<()> {
    debug!("Installing pipx packages: {:?}", packages);

    for package in packages {
        let mut cmd = Command::new("pipx");
        cmd.args(["install", package]);

        if let Some(version) = python_version {
            cmd.args(["--python", version]);
        }

        cmd.output()
            .with_context(|| format!("Failed to install pipx package: {}", package))?;
    }

    info!("Successfully installed pipx packages: {:?}", packages);
    Ok(())
}
