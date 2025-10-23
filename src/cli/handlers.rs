use super::RetryConfig;
use super::args::{Commands, normalize_package_list, parse_key_value_pairs};
use crate::installers;
use crate::utils;
use anyhow::Result;

pub fn handle_command(command: Commands, retry_config: &RetryConfig) -> Result<()> {
    match command {
        Commands::AptGet { packages, ppa_args } => {
            anyhow::ensure!(
                utils::os::is_debian_like(),
                "apt-get command is only supported on Debian/Ubuntu systems. Use 'apk' on Alpine Linux."
            );
            let pkg_list = normalize_package_list(&packages);
            let ppa_list = ppa_args.ppas.as_ref().map(|p| normalize_package_list(p));

            installers::package_manager::install_apt_get(
                &installers::package_manager::PackageManagerConfig {
                    packages: &pkg_list,
                    ppas: ppa_list.as_deref(),
                    force_ppas_on_non_ubuntu: ppa_args.force_ppas_on_non_ubuntu,
                },
            )
        }

        Commands::Apt { packages, ppa_args } => {
            anyhow::ensure!(
                utils::os::is_debian_like(),
                "apt command is only supported on Debian/Ubuntu systems. Use 'apk' on Alpine Linux."
            );
            let pkg_list = normalize_package_list(&packages);
            let ppa_list = ppa_args.ppas.as_ref().map(|p| normalize_package_list(p));

            installers::package_manager::install_apt(
                &installers::package_manager::PackageManagerConfig {
                    packages: &pkg_list,
                    ppas: ppa_list.as_deref(),
                    force_ppas_on_non_ubuntu: ppa_args.force_ppas_on_non_ubuntu,
                },
            )
        }

        Commands::Aptitude { packages } => {
            anyhow::ensure!(
                utils::os::is_debian_like(),
                "aptitude command is only supported on Debian/Ubuntu systems. Use 'apk' on Alpine Linux."
            );
            let pkg_list = normalize_package_list(&packages);
            installers::package_manager::install_aptitude(&pkg_list)
        }

        Commands::Apk { packages } => {
            anyhow::ensure!(
                utils::os::is_alpine(),
                "apk command is only supported on Alpine Linux. Use 'apt-get' on Debian/Ubuntu systems."
            );
            let pkg_list = normalize_package_list(&packages);
            installers::package_manager::install_apk(&pkg_list)
        }

        Commands::Brew { packages } => {
            anyhow::ensure!(
                utils::os::is_macos(),
                "brew command is only supported on macOS. Use 'apt-get' on Debian/Ubuntu or 'apk' on Alpine Linux."
            );
            let pkg_list = normalize_package_list(&packages);
            installers::package_manager::install_brew(&pkg_list)
        }

        Commands::Npm { packages } => {
            let pkg_list = normalize_package_list(&packages);
            installers::npm::install(&pkg_list)
        }

        Commands::Pipx { packages, python } => {
            let pkg_list = normalize_package_list(&packages);
            installers::pipx::install(&pkg_list, python.as_deref())
        }

        Commands::DevcontainerFeature {
            feature,
            option,
            remote_user,
            env,
            script,
            user,
            registry_username,
            registry_password,
            registry_token,
        } => {
            anyhow::ensure!(
                utils::os::is_linux(),
                "devcontainer-feature command is only supported on Linux systems."
            );
            let options = parse_key_value_pairs(&option);
            let envs = parse_key_value_pairs(&env);

            let config = installers::devcontainer_feature::DevcontainerFeatureConfig {
                feature_ref: &feature,
                options,
                remote_user: remote_user.as_deref(),
                envs,
                script_name: &script,
                user: user.as_deref(),
                registry_username: registry_username.as_deref(),
                registry_password: registry_password.as_deref(),
                registry_token: registry_token.as_deref(),
            };

            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(installers::devcontainer_feature::install_async(
                &config,
                retry_config,
            ))
        }

        Commands::GhRelease {
            owner,
            repo,
            binary,
            version,
            install_dir,
            filter,
            verify_checksum,
            checksum_text,
            gpg_key,
            include_prerelease,
        } => {
            anyhow::ensure!(
                utils::os::is_debian_like(),
                "gh-release command is only supported on Debian/Ubuntu systems."
            );
            let binary_list = normalize_package_list(&binary.unwrap_or_else(|| repo.clone()));

            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(installers::gh_release::install(
                &installers::gh_release::GhReleaseConfig {
                    owner: &owner,
                    repo: &repo,
                    binary_names: &binary_list,
                    version: &version,
                    install_dir: &install_dir,
                    filter: filter.as_deref(),
                    verify_checksum,
                    checksum_text: checksum_text.as_deref(),
                    gpg_key: gpg_key.as_deref(),
                    include_prerelease,
                },
                retry_config,
            ))
        }
        Commands::Pkgx {
            tool,
            version,
            args,
            working_dir,
            env,
        } => {
            let config = installers::pkgx::PkgxConfig {
                tool: &tool,
                version: &version,
                args,
                working_dir: &working_dir,
                env_vars: env,
            };
            installers::pkgx::execute(&config)
        }
    }
}
