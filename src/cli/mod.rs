mod args;
mod handlers;

use anyhow::Result;
use clap::Parser;

pub use args::{Cli, RetryConfig};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let retry_config = args::RetryConfig::from_cli(&cli);
    handlers::handle_command(cli.command, &retry_config)
}
