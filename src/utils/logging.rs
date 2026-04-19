use anyhow::{Context, Result};
use log::LevelFilter;
use std::fs;
use std::path::PathBuf;

pub fn init_logging(verbose: u8, quiet: bool) -> Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(get_log_level(verbose, quiet));

    if let Ok(log_file_path) = std::env::var("PICOLAYER_LOG_FILE")
        && !log_file_path.is_empty()
    {
        setup_file_logging(&mut builder, &log_file_path)?;
    }

    builder.init();
    Ok(())
}

fn get_log_level(verbose: u8, quiet: bool) -> LevelFilter {
    // CLI flags take precedence
    if quiet {
        return LevelFilter::Error;
    }
    if verbose > 0 {
        return match verbose {
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };
    }

    // Fall back to env vars
    if let Ok(level_str) = std::env::var("PICOLAYER_LOG_LEVEL")
        && let Ok(level) = level_str.parse()
    {
        return level;
    }

    if let Ok(level_str) = std::env::var("RUST_LOG")
        && let Ok(level) = level_str.parse()
    {
        return level;
    }

    LevelFilter::Warn
}

fn setup_file_logging(builder: &mut env_logger::Builder, log_file_path: &str) -> Result<()> {
    let path = PathBuf::from(log_file_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create log directory: {}", parent.display()))?;
    }

    let file = fs::File::create(&path)
        .with_context(|| format!("Failed to create log file: {}", path.display()))?;

    builder.target(env_logger::Target::Pipe(Box::new(file)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use serial_test::serial;

    #[test]
    #[serial]
    fn get_log_level_defaults_to_warn() {
        // Clear relevant env vars
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::remove_var("PICOLAYER_LOG_LEVEL");
            std::env::remove_var("RUST_LOG");
        }
        assert_eq!(get_log_level(0, false), LevelFilter::Warn);
    }

    #[test]
    #[serial]
    fn get_log_level_respects_picolayer_env() {
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::set_var("PICOLAYER_LOG_LEVEL", "debug");
            std::env::remove_var("RUST_LOG");
        }
        let level = get_log_level(0, false);
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::remove_var("PICOLAYER_LOG_LEVEL");
        }
        assert_eq!(level, LevelFilter::Debug);
    }

    #[test]
    #[serial]
    fn get_log_level_picolayer_overrides_rust_log() {
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::set_var("PICOLAYER_LOG_LEVEL", "error");
            std::env::set_var("RUST_LOG", "info");
        }
        let level = get_log_level(0, false);
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::remove_var("PICOLAYER_LOG_LEVEL");
            std::env::remove_var("RUST_LOG");
        }
        assert_eq!(level, LevelFilter::Error);
    }

    #[test]
    #[serial]
    fn get_log_level_falls_back_to_rust_log() {
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::remove_var("PICOLAYER_LOG_LEVEL");
            std::env::set_var("RUST_LOG", "info");
        }
        let level = get_log_level(0, false);
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::remove_var("RUST_LOG");
        }
        assert_eq!(level, LevelFilter::Info);
    }

    #[test]
    #[serial]
    fn get_log_level_ignores_invalid_values() {
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::set_var("PICOLAYER_LOG_LEVEL", "not_a_level");
            std::env::remove_var("RUST_LOG");
        }
        let level = get_log_level(0, false);
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::remove_var("PICOLAYER_LOG_LEVEL");
        }
        // Falls through to default when parse fails
        assert_eq!(level, LevelFilter::Warn);
    }

    #[test]
    fn get_log_level_quiet_returns_error() {
        assert_eq!(get_log_level(0, true), LevelFilter::Error);
    }

    #[test]
    fn get_log_level_verbose_1_returns_info() {
        assert_eq!(get_log_level(1, false), LevelFilter::Info);
    }

    #[test]
    fn get_log_level_verbose_2_returns_debug() {
        assert_eq!(get_log_level(2, false), LevelFilter::Debug);
    }

    #[test]
    fn get_log_level_verbose_3_returns_trace() {
        assert_eq!(get_log_level(3, false), LevelFilter::Trace);
    }

    #[test]
    #[serial]
    fn get_log_level_cli_verbose_overrides_env() {
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::set_var("PICOLAYER_LOG_LEVEL", "error");
        }
        let level = get_log_level(2, false);
        unsafe {
            std::env::remove_var("PICOLAYER_LOG_LEVEL");
        }
        assert_eq!(level, LevelFilter::Debug);
    }
}
