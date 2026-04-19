mod cli;
mod error;
mod installers;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use error::PicolayerError;
use log::info;
use std::process;

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default CryptoProvider");

    if let Err(e) = run().await {
        let picolayer_error: PicolayerError = e.into();
        eprintln!("{}", picolayer_error);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    utils::logging::init_logging(cli.verbose, cli.quiet).context("Failed to initialize logging")?;
    info!("Starting picolayer");
    cli::run(cli).await?;
    Ok(())
}
