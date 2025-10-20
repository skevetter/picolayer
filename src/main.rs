mod cli;
mod error;
mod installers;
mod utils;

use anyhow::{Context, Result};
use error::PicolayerError;
use log::info;
use std::process;

fn main() {
    if let Err(e) = run() {
        let picolayer_error: PicolayerError = e.into();
        eprintln!("{}", picolayer_error);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    utils::logging::init_logging().context("Failed to initialize logging")?;
    info!("Starting picolayer");
    cli::run()?;
    Ok(())
}
