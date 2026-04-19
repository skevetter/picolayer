use crate::utils;
use anyhow::Result;
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
    let mut cmd = utils::sudo::command("apt-get");
    cmd.args(["update"]);
    utils::subprocess::run_command(&mut cmd, "Update package lists")?;

    let mut cmd = utils::sudo::command("apt-get");
    cmd.args(["install", "-y", "pipx"]);
    utils::subprocess::run_command(&mut cmd, "Install pipx")?;

    Ok(())
}

fn install_pipx_alpine() -> Result<()> {
    debug!("Installing pipx on Alpine Linux");
    let mut cmd = utils::sudo::command("apk");
    cmd.args(["add", "py3-pip", "python3"]);
    utils::subprocess::run_command(&mut cmd, "Install Python and pip")?;

    let mut cmd = Command::new("pip3");
    cmd.args(["install", "--user", "pipx"]);
    utils::subprocess::run_command(&mut cmd, "Install pipx via pip")?;

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

        utils::subprocess::run_command(&mut cmd, &format!("Install pipx package: {}", package))?;
    }

    info!("Successfully installed pipx packages: {:?}", packages);
    Ok(())
}
