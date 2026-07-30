mod cli;
mod launchd;
mod model;
mod privilege;
mod web;

use clap::Parser;
use cli::{handle_cli, Cli, CliAction};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing / log subscriber
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match handle_cli(cli)? {
        CliAction::Executed => {}
        CliAction::StartWeb { host, port, open } => {
            web::start_web_server(&host, port, open).await?;
        }
    }

    Ok(())
}
