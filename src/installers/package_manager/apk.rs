use crate::utils;
use anyhow::Result;
use log::info;

pub(super) fn install(packages: &[String]) -> Result<()> {
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
    let mut cmd = utils::sudo::command("apk");
    cmd.args(["update"]);
    utils::subprocess::run_command(&mut cmd, "Update apk repositories")?;
    Ok(())
}

fn install_packages(packages: &[String]) -> Result<()> {
    info!("Installing apk packages: {:?}", packages);
    let mut cmd = utils::sudo::command("apk");
    cmd.args(["add", "--no-cache"]).args(packages);
    utils::subprocess::run_command(&mut cmd, "Install apk packages")?;
    Ok(())
}

fn cleanup() -> Result<()> {
    info!("Cleaning up apk cache");
    let mut cmd = utils::sudo::command("apk");
    cmd.args(["cache", "clean"]);
    utils::subprocess::run_command(&mut cmd, "Clean apk cache")?;
    Ok(())
}
