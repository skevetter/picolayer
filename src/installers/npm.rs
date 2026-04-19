use crate::utils;
use anyhow::Result;
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
    let mut cmd = utils::sudo::command("apt-get");
    cmd.args(["update"]);
    utils::subprocess::run_command(&mut cmd, "Update package lists")?;

    let mut cmd = utils::sudo::command("apt-get");
    cmd.args(["install", "-y", "nodejs", "npm"]);
    utils::subprocess::run_command(&mut cmd, "Install Node.js and npm")?;

    Ok(())
}

fn install_nodejs_alpine() -> Result<()> {
    debug!("Installing Node.js on Alpine Linux");
    let mut cmd = utils::sudo::command("apk");
    cmd.args(["add", "nodejs", "npm"]);
    utils::subprocess::run_command(&mut cmd, "Install Node.js and npm")?;

    Ok(())
}

fn install_packages(packages: &[String]) -> Result<()> {
    debug!("Installing npm packages: {:?}", packages);

    let mut cmd = Command::new("npm");
    cmd.args(["install", "-g"]);
    cmd.args(packages);
    utils::subprocess::run_command(&mut cmd, "Install npm packages")?;

    info!("Successfully installed npm packages: {:?}", packages);
    Ok(())
}
