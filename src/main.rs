mod cli;
mod claude_config;
mod commands;
mod error;
mod keychain;
#[cfg(feature = "oauth")]
mod oauth;
mod profile;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Add {
            name,
            api_key,
            oauth,
            label,
        } => commands::add::run(name, api_key, oauth, label),
        Commands::Use { name } => commands::use_profile::run(name),
        Commands::List => commands::list::run(),
        Commands::Current => commands::current::run(),
        Commands::Remove { name, force } => commands::remove::run(name, force),
        Commands::Import { name, label } => commands::import::run(name, label),
        Commands::Init => commands::init::run(),
        Commands::EmitKey => commands::emit_key::run(),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", "Error:".red().bold());
        std::process::exit(1);
    }
}
