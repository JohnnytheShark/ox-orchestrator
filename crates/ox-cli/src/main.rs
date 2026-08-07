mod cli_args;
mod commands;
mod config;
mod tui;

use clap::Parser;
use cli_args::{Cli, Commands};
use config::ConfigResolver;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    let workspace_root = ConfigResolver::find_workspace_root(cli.workspace);
    let config_file = ConfigResolver::load_hierarchical_config(&workspace_root);

    let provider_config = ConfigResolver::resolve_provider_config(
        cli.model.as_deref(),
        cli.provider.as_deref(),
        cli.base_url.as_deref(),
        &config_file,
    );

    let auto_approve = cli.auto_approve || config_file.get_auto_approve().unwrap_or(false);
    let default_max_turns = config_file.get_max_turns().unwrap_or(25);

    match cli.command {
        Some(Commands::Chat { session, prompt }) => {
            commands::run_chat(
                provider_config,
                workspace_root,
                session,
                prompt,
                auto_approve,
                config_file,
            )
            .await?;
        }
        Some(Commands::Run { prompt, max_turns }) => {
            commands::run_prompt(
                provider_config,
                workspace_root,
                prompt,
                max_turns.unwrap_or(default_max_turns),
                auto_approve,
                config_file,
            )
            .await?;
        }
        Some(Commands::Session { command }) => {
            commands::handle_session_command(command, &workspace_root)?;
        }
        Some(Commands::Tools) => {
            commands::handle_tools_command();
        }
        None => {
            // Default to starting interactive chat REPL
            commands::run_chat(
                provider_config,
                workspace_root,
                None,
                None,
                auto_approve,
                config_file,
            )
            .await?;
        }
    }

    Ok(())
}
