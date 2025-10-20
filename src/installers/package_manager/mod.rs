mod apk;
mod apt_based;
mod brew;

use anyhow::Result;

pub struct PackageManagerConfig<'a> {
    pub packages: &'a [String],
    pub ppas: Option<&'a [String]>,
    pub force_ppas_on_non_ubuntu: bool,
}

pub fn install_apt_get(config: &PackageManagerConfig) -> Result<()> {
    apt_based::install("apt-get", config)
}

pub fn install_apt(config: &PackageManagerConfig) -> Result<()> {
    apt_based::install("apt", config)
}

pub fn install_aptitude(packages: &[String]) -> Result<()> {
    apt_based::install_aptitude(packages)
}

pub fn install_apk(packages: &[String]) -> Result<()> {
    apk::install(packages)
}

pub fn install_brew(packages: &[String]) -> Result<()> {
    brew::install(packages)
}
