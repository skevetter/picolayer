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
