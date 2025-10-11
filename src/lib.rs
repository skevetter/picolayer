pub mod apk;
pub mod apt;
pub mod apt_get;
pub mod aptitude;
pub mod brew;
pub mod devcontainer_feature;
pub mod gh_release;
pub mod run;
pub mod utils;

pub use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum DeleteOption {
    /// Don't delete anything (keep both pkgx and packages)
    None,
    /// Delete only the installed package
    Package,
    /// Delete the entire pkgx installation
    Pkgx,
}
