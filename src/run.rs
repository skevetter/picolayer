use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
#[cfg(not(target_os = "macos"))]
use std::env;
use std::path::{Path, PathBuf};

const PKGX_BIN_PATHS: [&str; 2] = ["/usr/local/bin/pkgx", "/usr/local/bin/pkgm"];
#[cfg(target_os = "macos")]
const PKGX_MACOS_DATA_PATHS: [&str; 2] =
    ["Library/Caches/pkgx", "Library/Application Support/pkgx"];

pub struct RunConfig<'a> {
    pub tool: &'a str,
    pub args: Vec<String>,
    pub working_dir: &'a str,
    pub env_vars: Vec<String>,
    pub delete: crate::DeleteOption,
}

struct PkgxBackup {
    binaries: Vec<(PathBuf, Vec<u8>)>,
    #[allow(dead_code)]
    data_dirs: Vec<PathBuf>,
}

fn uninstall_pkgx() -> Result<()> {
    info!("Uninstalling pkgx and removing all associated files...");
    let items_to_delete = collect_pkgx_paths()?;
    if items_to_delete.is_empty() {
        info!("No pkgx installation found to remove");
        return Ok(());
    }

    info!("Found {} pkgx items to remove", items_to_delete.len());
    let mut failed_items = Vec::new();

    for path in items_to_delete {
        if !path.exists() {
            continue;
        }

        let result = if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };

        match result {
            Ok(()) => {
                debug!("Removed: {}", path.display());
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                warn!("Failed to remove {}: {}", path.display(), error_msg);
                failed_items.push((path.display().to_string(), error_msg));
            }
        }
    }

    if !failed_items.is_empty() {
        warn!("Failed to remove {} items:", failed_items.len());
        for (path, error) in failed_items {
            warn!("  - {}: {}", path, error);
        }
    }

    Ok(())
}

/// Collect all pkgx-related paths to delete based on the operating system
fn collect_pkgx_paths() -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for bin_path in PKGX_BIN_PATHS {
        paths.push(PathBuf::from(bin_path));
    }

    if let Some(home_dir) = dirs_next::home_dir() {
        paths.push(home_dir.join(".pkgx"));
    }

    let platform_paths = get_platform_specific_paths()?;
    for path in platform_paths {
        paths.push(path);
    }

    let existing_paths: Vec<PathBuf> = paths.into_iter().filter(|path| path.exists()).collect();

    Ok(existing_paths)
}

fn get_platform_specific_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    if let Some(home_dir) = dirs_next::home_dir() {
        #[cfg(target_os = "macos")]
        {
            for path in PKGX_MACOS_DATA_PATHS {
                paths.push(home_dir.join(path));
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let cache_dir = if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
                PathBuf::from(xdg_cache)
            } else {
                home_dir.join(".cache")
            };
            paths.push(cache_dir.join("pkgx"));

            let data_dir = if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
                PathBuf::from(xdg_data)
            } else {
                home_dir.join(".local/share")
            };
            paths.push(data_dir.join("pkgx"));
        }
    }

    Ok(paths)
}

/// Backup existing pkgx installation before we make any changes
fn backup_existing_pkgx() -> Result<Option<PkgxBackup>> {
    info!("Checking for existing pkgx installation...");
    
    let mut binaries = Vec::new();
    let mut data_dirs = Vec::new();
    
    // Backup binary files
    for bin_path_str in PKGX_BIN_PATHS {
        let bin_path = PathBuf::from(bin_path_str);
        if bin_path.exists() && bin_path.is_file() {
            match std::fs::read(&bin_path) {
                Ok(content) => {
                    info!("Backing up existing binary: {}", bin_path.display());
                    binaries.push((bin_path, content));
                }
                Err(e) => {
                    warn!("Failed to backup {}: {}", bin_path.display(), e);
                }
            }
        }
    }
    
    // Check for existing data directories
    if let Some(home_dir) = dirs_next::home_dir() {
        let pkgx_home = home_dir.join(".pkgx");
        if pkgx_home.exists() && pkgx_home.is_dir() {
            info!("Found existing .pkgx directory: {}", pkgx_home.display());
            data_dirs.push(pkgx_home);
        }
    }
    
    let platform_paths = get_platform_specific_paths()?;
    for path in platform_paths {
        if path.exists() && path.is_dir() {
            info!("Found existing pkgx data directory: {}", path.display());
            data_dirs.push(path);
        }
    }
    
    if binaries.is_empty() && data_dirs.is_empty() {
        info!("No existing pkgx installation found");
        Ok(None)
    } else {
        info!("Backed up {} binaries and found {} data directories", binaries.len(), data_dirs.len());
        Ok(Some(PkgxBackup { binaries, data_dirs }))
    }
}

/// Restore pkgx installation from backup
fn restore_pkgx_from_backup(backup: Option<PkgxBackup>) -> Result<()> {
    if let Some(backup) = backup {
        info!("Restoring pkgx from backup...");
        
        // Restore binaries
        for (path, content) in backup.binaries {
            match std::fs::write(&path, content) {
                Ok(_) => {
                    info!("Restored binary: {}", path.display());
                    // Ensure the binary is executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(mut perms) = std::fs::metadata(&path).map(|m| m.permissions()) {
                            perms.set_mode(0o755);
                            let _ = std::fs::set_permissions(&path, perms);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to restore {}: {}", path.display(), e);
                }
            }
        }
        
        // Note: We don't restore data directories as they were just markers
        // The actual data is preserved by not deleting them
        info!("Pkgx restoration complete");
    }
    
    Ok(())
}

pub fn execute(input: &RunConfig) -> Result<()> {
    // Backup existing pkgx installation before making any changes
    let pkgx_backup = backup_existing_pkgx()?;
    let had_existing_pkgx = pkgx_backup.is_some();

    let working_path = Path::new(input.working_dir);
    if !working_path.exists() {
        anyhow::bail!("Working directory does not exist: {}", input.working_dir);
    }
    let (tool_name, version_spec) = parse_tool_spec(input.tool);
    info!("Working directory: {}", input.working_dir);
    info!("Tool: {} ({})", tool_name, version_spec);
    info!("Command: {}", input.args.join(" "));

    let mut env_map = Vec::new();
    for env_var in &input.env_vars {
        if let Some((key, value)) = env_var.split_once('=') {
            env_map.push((key.to_string(), value.to_string()));
        } else {
            anyhow::bail!(
                "Invalid environment variable format: {} (expected key=value)",
                env_var
            );
        }
    }

    // Determine whether to keep packages based on delete option
    let keep_package = !matches!(input.delete, crate::DeleteOption::Package);

    let exec_result = if crate::utils::pkgx::check_pkgx_binary() {
        execute_with_pkgx_binary(
            &tool_name,
            &version_spec,
            &input.args,
            working_path,
            &env_map,
        )
    } else {
        execute_with_pkgx_library(
            &tool_name,
            &version_spec,
            &input.args,
            working_path,
            &env_map,
            keep_package,
        )
    };

    // Handle cleanup based on delete option
    match input.delete {
        crate::DeleteOption::Pkgx => {
            if had_existing_pkgx {
                info!("Existing pkgx installation detected, restoring from backup instead of deleting");
                restore_pkgx_from_backup(pkgx_backup)?;
            } else {
                info!("No existing pkgx found, proceeding with deletion");
                uninstall_pkgx()?;
            }
        }
        crate::DeleteOption::None => {
            info!("Keeping all installations (both pkgx and packages)");
            // If there was an existing pkgx, we don't need to do anything
            // as we never deleted it in the first place
        }
        crate::DeleteOption::Package => {
            // Packages are cleaned up within the execution functions
            info!("Package cleanup handled during execution");
        }
    }

    match exec_result {
        Ok(()) => {}
        Err(e) => {
            warn!("Command failed: {}", e);
        }
    }

    Ok(())
}

fn parse_tool_spec(tool: &str) -> (String, String) {
    if let Some((name, version)) = tool.split_once('@') {
        (name.to_string(), version.to_string())
    } else {
        (tool.to_string(), "latest".to_string())
    }
}

fn execute_with_pkgx_library(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    keep_package: bool,
) -> Result<()> {
    info!("Using pkgx library integration...");

    match try_libpkgx_execution(
        tool_name,
        version_spec,
        args,
        working_path,
        env_map,
        keep_package,
    ) {
        Ok(()) => {
            info!("Command executed with pkgx library!");
            Ok(())
        }
        Err(e) => {
            warn!("pkgx library execution failed: {}", e);
            info!("Falling back to pkgx binary execution");
            execute_with_pkgx_binary(tool_name, version_spec, args, working_path, env_map)
        }
    }
}

fn try_libpkgx_execution(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    keep_package: bool,
) -> Result<()> {
    use std::env;

    if args.is_empty() {
        anyhow::bail!("No arguments provided for tool: {}", tool_name);
    }

    let (project_name, tool_spec) =
        crate::utils::pkgx::resolve_tool_to_project(tool_name, version_spec)?;

    info!("Resolving package: {}", tool_spec);
    let mut cmd_env = HashMap::new();
    for (key, value) in env::vars() {
        cmd_env.insert(key, value);
    }

    for (key, value) in env_map {
        cmd_env.insert(key.clone(), value.clone());
    }

    let mut paths_to_cleanup = Vec::new();

    let execution_result = match crate::utils::pkgx::resolve_package_with_libpkgx(&[tool_spec]) {
        Ok((pkgx_env, installations)) => {
            for (key, value) in pkgx_env {
                cmd_env.insert(key, value);
            }

            for installation in &installations {
                if installation.pkg.project == project_name {
                    info!("Package installed at: {}", installation.path.display());

                    if !keep_package {
                        paths_to_cleanup.push(installation.path.clone());
                    }

                    let bin_paths = vec!["bin", "sbin"];
                    for bin_dir in bin_paths {
                        let executable_path = installation.path.join(bin_dir).join(tool_name);
                        if executable_path.exists() {
                            info!("Executable found at: {}", executable_path.display());
                            break;
                        }
                    }
                }
            }

            info!("Resolved package with libpkgx");
            std::process::Command::new(tool_name)
                .args(args.iter().map(|arg| arg.as_str()).collect::<Vec<&str>>())
                .current_dir(working_path.to_str().context("Invalid working directory")?)
                .envs(
                    cmd_env
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect::<Vec<(&str, &str)>>(),
                )
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .context("Failed to execute command with libpkgx")?;

            Ok(())
        }
        Err(e) => {
            warn!("Failed to resolve package with libpkgx");
            Err(e)
        }
    };

    if !keep_package && !paths_to_cleanup.is_empty() {
        for path in paths_to_cleanup {
            if let Err(e) = cleanup_installation(&path) {
                warn!("Failed to cleanup {}: {}", path.display(), e);
            } else {
                info!("Removed: {}", path.display());
            }
        }
        info!("Cleanup completed");
    }

    execution_result
}

fn execute_with_pkgx_binary(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
) -> Result<()> {
    let pkgx_available = crate::utils::pkgx::check_pkgx_binary();

    if !pkgx_available {
        anyhow::bail!("pkgx is not available. Install pkgx from https://pkgx.sh.");
    }

    let (project_name, _) = crate::utils::pkgx::resolve_tool_to_project(tool_name, version_spec)?;

    let project_arg = if version_spec == "latest" {
        format!("+{}", project_name)
    } else {
        format!("+{}@{}", project_name, version_spec)
    };

    std::process::Command::new("pkgx")
        .arg(&project_arg)
        .arg(tool_name)
        .args(args.iter().map(|arg| arg.as_str()).collect::<Vec<&str>>())
        .current_dir(working_path.to_str().context("Invalid working directory")?)
        .envs(
            env_map
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect::<Vec<(&str, &str)>>(),
        )
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to execute command with pkgx")?;

    Ok(())
}

fn cleanup_installation(path: &PathBuf) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            std::fs::remove_dir_all(path)
                .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
        } else {
            std::fs::remove_file(path)
                .with_context(|| format!("Failed to remove file: {}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_spec_with_version() {
        let (name, version) = parse_tool_spec("python@3.11");
        assert_eq!(name, "python");
        assert_eq!(version, "3.11");
    }

    #[test]
    fn test_parse_tool_spec_without_version() {
        let (name, version) = parse_tool_spec("python");
        assert_eq!(name, "python");
        assert_eq!(version, "latest");
    }

    #[test]
    fn test_parse_tool_spec_complex_version() {
        let (name, version) = parse_tool_spec("node@18.16.0");
        assert_eq!(name, "node");
        assert_eq!(version, "18.16.0");
    }

    #[test]
    fn test_map_tool_to_project_python() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("python", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_node() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("node", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_go() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("go", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_rust() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("cargo", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_unknown() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("unknown-tool-xyz-not-real", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project, "unknown-tool-xyz-not-real");
    }

    #[test]
    fn test_resolve_tool_to_project() {
        let result = crate::utils::pkgx::resolve_tool_to_project("node", "latest");
        match &result {
            Ok(_) => {
                let (project, spec) = result.unwrap();
                assert!(!project.is_empty());
                assert_eq!(spec, project);
            }
            Err(e) => {
                eprintln!("Skipping test due to sync error: {}", e);
                if e.to_string().contains("403 Forbidden") || e.to_string().contains("HTTP") {
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }

        // Test with version
        let result = crate::utils::pkgx::resolve_tool_to_project("python", "3.11");
        match &result {
            Ok(_) => {
                let (project, spec) = result.unwrap();
                assert!(!project.is_empty());
                assert!(spec.contains("@3.11"));
            }
            Err(e) => {
                eprintln!("Skipping test due to sync error: {}", e);
                if e.to_string().contains("403 Forbidden") || e.to_string().contains("HTTP") {
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    fn test_query_various_tools() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let tools = vec!["bash", "git", "curl", "wget", "make"];
        for tool in tools {
            let result = crate::utils::pkgx::map_tool_to_project(tool, &conn);
            assert!(result.is_ok(), "Failed to query tool: {}", tool);
            let project = result.unwrap();
            assert!(!project.is_empty(), "Empty project for tool: {}", tool);
        }
    }

    #[test]
    fn test_collect_pkgx_paths() {
        let paths = collect_pkgx_paths();

        assert!(paths.is_ok());

        let paths = paths.unwrap();
        let path_strings: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();

        let _has_bin_paths = path_strings
            .iter()
            .any(|p| p.contains("/usr/local/bin/pkgx"));
        let _has_home_path = path_strings.iter().any(|p| p.contains(".pkgx"));
    }

    #[test]
    fn test_get_platform_specific_paths() {
        let paths = get_platform_specific_paths();

        assert!(paths.is_ok());

        let paths = paths.unwrap();

        assert!(paths.len() >= 1);

        for path in paths {
            assert!(path.to_string_lossy().contains("pkgx"));
        }
    }
}
