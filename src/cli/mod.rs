mod args;
mod handlers;

use anyhow::Result;

pub use args::{Cli, RetryConfig};

pub async fn run(cli: Cli) -> Result<()> {
    let retry_config = args::RetryConfig::from_cli(&cli);
    handlers::handle_command(cli.command, &retry_config).await
}
