use crate::utils;
use anyhow::Result;
use log::{info, warn};

use super::PackageManagerConfig;

const PPA_SUPPORT_PACKAGES: &[&str] = &["software-properties-common"];
const PPA_SUPPORT_PACKAGES_DEBIAN: &[&str] = &["python3-launchpadlib"];

pub(super) fn install(tool: &str, config: &PackageManagerConfig) -> Result<()> {
    anyhow::ensure!(
        which::which(tool).is_ok(),
        "{} command not found in PATH",
        tool
    );

    let mut ppas = config.ppas.map(|p| p.to_vec()).unwrap_or_default();
    if !ppas.is_empty() && !utils::os::is_ubuntu() && !config.force_ppas_on_non_ubuntu {
        warn!(
            "PPAs are ignored on non-Ubuntu distros. Use --force-ppas-on-non-ubuntu to include them anyway."
        );
        ppas.clear();
    }

    update_repositories()?;

    if !ppas.is_empty() {
        install_ppa_support()?;
        add_ppas(&ppas)?;
        update_repositories()?;
    }

    install_packages(tool, config.packages)?;
    cleanup()?;

    Ok(())
}

pub(super) fn install_aptitude(packages: &[String]) -> Result<()> {
    update_repositories()?;
    install_aptitude_tool()?;
    install_packages_aptitude(packages)?;
    cleanup_aptitude()?;

    Ok(())
}

fn update_repositories() -> Result<()> {
    info!("Updating repositories");
    let mut cmd = utils::sudo::command("apt-get");
    cmd.args(["update", "-y"]);
    utils::subprocess::run_command(&mut cmd, "Update repositories")?;
    Ok(())
}

fn install_ppa_support() -> Result<()> {
    info!("Installing PPA support packages");
    let mut cmd = utils::sudo::command("apt-get");
    cmd.args(["install", "-y", "--no-install-recommends"])
        .args(PPA_SUPPORT_PACKAGES);
    utils::subprocess::run_command(&mut cmd, "Install PPA support packages")?;

    if utils::os::is_debian() {
        let mut cmd = utils::sudo::command("apt-get");
        cmd.args(["install", "-y", "--no-install-recommends"])
            .args(PPA_SUPPORT_PACKAGES_DEBIAN);
        utils::subprocess::run_command(&mut cmd, "Install Debian PPA support packages")?;
    }
    Ok(())
}

fn add_ppas(ppas: &[String]) -> Result<()> {
    for ppa in ppas {
        info!("Adding PPA: {}", ppa);
        let mut cmd = utils::sudo::command("add-apt-repository");
        cmd.args(["-y", ppa]);
        utils::subprocess::run_command(&mut cmd, &format!("Add PPA: {}", ppa))?;
    }
    Ok(())
}

fn install_packages(tool: &str, packages: &[String]) -> Result<()> {
    info!("Installing packages with {}: {:?}", tool, packages);
    let mut cmd = utils::sudo::command(tool);
    cmd.args(["install", "-y", "--no-install-recommends"])
        .args(packages);
    utils::subprocess::run_command(&mut cmd, "Install packages")?;
    Ok(())
}

fn install_aptitude_tool() -> Result<()> {
    info!("Installing aptitude");
    let mut cmd = utils::sudo::command("apt-get");
    cmd.args(["install", "-y", "--no-install-recommends", "aptitude"]);
    utils::subprocess::run_command(&mut cmd, "Install aptitude")?;
    Ok(())
}

fn install_packages_aptitude(packages: &[String]) -> Result<()> {
    info!("Installing packages with aptitude: {:?}", packages);
    let mut cmd = utils::sudo::command("aptitude");
    cmd.args(["install", "-y"]).args(packages);
    utils::subprocess::run_command(&mut cmd, "Install packages with aptitude")?;
    Ok(())
}

fn cleanup() -> Result<()> {
    info!("Cleaning package cache");
    let mut cmd = utils::sudo::command("apt-get");
    cmd.args(["clean"]);
    utils::subprocess::run_command(&mut cmd, "Clean package cache")?;
    Ok(())
}

fn cleanup_aptitude() -> Result<()> {
    info!("Cleaning aptitude cache");
    let mut cmd = utils::sudo::command("aptitude");
    cmd.args(["clean"]);
    utils::subprocess::run_command(&mut cmd, "Clean aptitude cache")?;
    Ok(())
}
