use anyhow::{Context, Result};
use libpkgx::config::Config;
use libpkgx::{
    hydrate, install_multi::ProgressBarExt, pantry_db, resolve, sync, types::PackageReq,
};
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;

/// Resolve package dependencies using libpkgx
pub fn resolve_package_with_libpkgx(
    dependencies: &[String],
) -> Result<(HashMap<String, String>, Vec<libpkgx::types::Installation>)> {
    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create Tokio runtime for libpkgx operations")?;

    rt.block_on(async { resolve_dependencies_async(dependencies).await })
}

async fn resolve_dependencies_async(
    dependencies: &[String],
) -> Result<(HashMap<String, String>, Vec<libpkgx::types::Installation>)> {
    struct ToolProgressBar {
        bar: indicatif::ProgressBar,
    }

    impl ToolProgressBar {
        fn new() -> Self {
            let bar = indicatif::ProgressBar::new(0);
            bar.set_style(
                indicatif::ProgressStyle::with_template(
                    "{elapsed:.dim} ❲{wide_bar:.cyan/blue}❳ {percent}% {bytes_per_sec:.dim} {bytes:.dim}"
                ).unwrap()
                .progress_chars("██░")
            );
            Self { bar }
        }
    }

    impl ProgressBarExt for ToolProgressBar {
        fn inc(&self, n: u64) {
            self.bar.inc(n);
        }

        fn inc_length(&self, n: u64) {
            self.bar.inc_length(n);
        }
    }

    // PKGX_DIR and PKGX_PANTRY_DIR are set in the environment
    assert!(std::env::var("PKGX_DIR").is_ok());
    assert!(std::env::var("PKGX_PANTRY_DIR").is_ok());

    let config = Config::new().context("Failed to initialize libpkgx config")?;

    std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
    let mut conn = rusqlite::Connection::open(&config.pantry_db_file)?;

    if sync::should(&config).map_err(|e| anyhow::anyhow!("{}", e))? {
        info!("Syncing pkgx pantry database");
        sync::ensure(&config, &mut conn)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    let mut package_reqs = Vec::new();
    for dep in dependencies {
        match PackageReq::parse(dep) {
            Ok(req) => package_reqs.push(req),
            Err(e) => {
                eprintln!("Warning: Failed to parse dependency {}: {}", dep, e);
                continue;
            }
        }
    }

    if package_reqs.is_empty() {
        return Ok((HashMap::new(), Vec::new()));
    }

    let hydrated_packages = hydrate::hydrate(&package_reqs, |project| {
        pantry_db::deps_for_project(&project, &conn)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to hydrate dependencies: {}", e))?;

    let resolution = resolve::resolve(&hydrated_packages, &config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to resolve packages: {}", e))?;

    let mut installations = resolution.installed;
    if !resolution.pending.is_empty() {
        info!(
            "Installing {} packages with libpkgx",
            resolution.pending.len()
        );
        let progress_bar = ToolProgressBar::new();
        let installed = libpkgx::install_multi::install_multi(
            &resolution.pending,
            &config,
            Some(Arc::new(progress_bar)),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to install packages: {}", e))?;
        installations.extend(installed);
    }

    let env_map = libpkgx::env::map(&installations);
    let platform_env = libpkgx::env::mix(env_map);
    let runtime_env = libpkgx::env::mix_runtime(&platform_env, &installations, &conn)
        .map_err(|e| anyhow::anyhow!("Failed to mix runtime environment: {}", e))?;

    info!(
        "Successfully resolved {} packages with libpkgx",
        dependencies.len()
    );
    Ok((runtime_env, installations))
}

/// Query the pkgx pantry database to resolve a tool name to a project name
pub fn map_tool_to_project(tool_name: &str, conn: &rusqlite::Connection) -> Result<String> {
    let tool_name_string = tool_name.to_string();
    match libpkgx::pantry_db::projects_for_symbol(&tool_name_string, conn) {
        Ok(projects) if !projects.is_empty() => {
            if projects.len() == 1 {
                Ok(projects[0].clone())
            } else {
                info!(
                    "Multiple projects provide '{}': {:?}, using {}",
                    tool_name, projects, projects[0]
                );
                Ok(projects[0].clone())
            }
        }
        Ok(_) => {
            warn!(
                "No project found for tool '{}' in pantry database, using tool name as project",
                tool_name
            );
            Ok(tool_name.to_string())
        }
        Err(e) => {
            warn!(
                "Failed to query pantry database for tool '{}': {}, using tool name as project",
                tool_name, e
            );
            Ok(tool_name.to_string())
        }
    }
}

/// Resolve a tool name to a project name
pub fn resolve_tool_to_project(tool_name: &str) -> Result<String> {
    assert!(std::env::var("PKGX_DIR").is_ok());
    assert!(std::env::var("PKGX_PANTRY_DIR").is_ok());
    let config = Config::new().context("Failed to initialize libpkgx config")?;
    std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
    let mut conn = rusqlite::Connection::open(&config.pantry_db_file)?;

    // Sync if needed
    if sync::should(&config).map_err(|e| anyhow::anyhow!("{}", e))? {
        info!("Syncing pkgx pantry database");
        let rt =
            tokio::runtime::Runtime::new().context("Failed to create Tokio runtime for sync")?;
        rt.block_on(async {
            sync::ensure(&config, &mut conn)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))
        })?;
    }

    map_tool_to_project(tool_name, &conn)
}

/// Format tool spec for libpkgx (without + prefix)
pub fn format_tool_spec(project_name: &str, version_spec: &str) -> String {
    if version_spec == "latest" {
        project_name.to_string()
    } else {
        format!("{}@{}", project_name, version_spec)
    }
}

/// Format project arg for pkgx binary (with + prefix)
pub fn format_project_arg(project_name: &str, version_spec: &str) -> String {
    if version_spec == "latest" {
        format!("+{}", project_name)
    } else {
        format!("+{}@{}", project_name, version_spec)
    }
}

/// Check if the pkgx binary is available on the system
pub fn check_pkgx_binary() -> bool {
    use std::process::{Command, Stdio};

    Command::new("pkgx")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
