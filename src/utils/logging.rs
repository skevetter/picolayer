use anyhow::{Context, Result};
use log::LevelFilter;
use std::fs;
use std::path::PathBuf;

pub fn init_logging() -> Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(get_log_level());

    if let Ok(log_file_path) = std::env::var("PICOLAYER_LOG_FILE")
        && !log_file_path.is_empty()
    {
        setup_file_logging(&mut builder, &log_file_path)?;
    }

    builder.init();
    Ok(())
}

fn get_log_level() -> LevelFilter {
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
        assert_eq!(get_log_level(), LevelFilter::Warn);
    }

    #[test]
    #[serial]
    fn get_log_level_respects_picolayer_env() {
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::set_var("PICOLAYER_LOG_LEVEL", "debug");
            std::env::remove_var("RUST_LOG");
        }
        let level = get_log_level();
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
        let level = get_log_level();
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
        let level = get_log_level();
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
        let level = get_log_level();
        // SAFETY: serialized via #[serial] so no concurrent env access
        unsafe {
            std::env::remove_var("PICOLAYER_LOG_LEVEL");
        }
        // Falls through to default when parse fails
        assert_eq!(level, LevelFilter::Warn);
    }
}
