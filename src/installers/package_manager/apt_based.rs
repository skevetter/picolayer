use crate::utils;
use anyhow::{Context, Result};
use log::{info, warn};

use super::PackageManagerConfig;

const PPA_SUPPORT_PACKAGES: &[&str] = &["software-properties-common"];
const PPA_SUPPORT_PACKAGES_DEBIAN: &[&str] = &["python3-launchpadlib"];

pub fn install(tool: &str, config: &PackageManagerConfig) -> Result<()> {
    anyhow::ensure!(
        which::which(tool).is_ok(),
        "{} command not found in PATH",
        tool
    );

    let mut ppas = config.ppas.map(|p| p.to_vec()).unwrap_or_default();
    if !ppas.is_empty() && !utils::os::is_ubuntu() && !config.force_ppas_on_non_ubuntu {
        warn!("PPAs are ignored on non-Ubuntu distros!");
        info!("Use --force-ppas-on-non-ubuntu to include them anyway.");
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

pub fn install_aptitude(packages: &[String]) -> Result<()> {
    update_repositories()?;
    install_aptitude_tool()?;
    install_packages_aptitude(packages)?;
    cleanup_aptitude()?;

    Ok(())
}

fn update_repositories() -> Result<()> {
    info!("Updating repositories");
    utils::sudo::command("apt-get")
        .args(["update", "-y"])
        .output()
        .context("Failed to update repositories")?;
    Ok(())
}

fn install_ppa_support() -> Result<()> {
    info!("Installing PPA support packages");
    utils::sudo::command("apt-get")
        .args(["install", "-y", "--no-install-recommends"])
        .args(PPA_SUPPORT_PACKAGES)
        .output()
        .context("Failed to install PPA support packages")?;

    if utils::os::is_debian() {
        utils::sudo::command("apt-get")
            .args(["install", "-y", "--no-install-recommends"])
            .args(PPA_SUPPORT_PACKAGES_DEBIAN)
            .output()
            .context("Failed to install Debian PPA support packages")?;
    }
    Ok(())
}

fn add_ppas(ppas: &[String]) -> Result<()> {
    for ppa in ppas {
        info!("Adding PPA: {}", ppa);
        utils::sudo::command("add-apt-repository")
            .args(["-y", ppa])
            .output()
            .with_context(|| format!("Failed to add PPA: {}", ppa))?;
    }
    Ok(())
}

fn install_packages(tool: &str, packages: &[String]) -> Result<()> {
    info!("Installing packages with {}: {:?}", tool, packages);
    utils::sudo::command(tool)
        .args(["install", "-y", "--no-install-recommends"])
        .args(packages)
        .output()
        .context("Failed to install packages")?;
    Ok(())
}

fn install_aptitude_tool() -> Result<()> {
    info!("Installing aptitude");
    utils::sudo::command("apt-get")
        .args(["install", "-y", "--no-install-recommends", "aptitude"])
        .output()
        .context("Failed to install aptitude")?;
    Ok(())
}

fn install_packages_aptitude(packages: &[String]) -> Result<()> {
    info!("Installing packages with aptitude: {:?}", packages);
    utils::sudo::command("aptitude")
        .args(["install", "-y"])
        .args(packages)
        .output()
        .context("Failed to install packages with aptitude")?;
    Ok(())
}

fn cleanup() -> Result<()> {
    info!("Cleaning package cache");
    utils::sudo::command("apt-get")
        .args(["clean"])
        .output()
        .context("Failed to clean package cache")?;
    Ok(())
}

fn cleanup_aptitude() -> Result<()> {
    info!("Cleaning aptitude cache");
    utils::sudo::command("aptitude")
        .args(["clean"])
        .output()
        .context("Failed to clean aptitude cache")?;
    Ok(())
}
