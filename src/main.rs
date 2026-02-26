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
        Commands::Add { name } => commands::add::run(name),
        Commands::Use { name } => commands::use_profile::run(name),
        Commands::List => commands::list::run(),
        Commands::Current => commands::current::run(),
        Commands::Remove { name } => commands::remove::run(name),
        Commands::Import { name } => commands::import::run(name),
        Commands::Init => commands::init::run(),
        Commands::Update => commands::update::run(),
        Commands::EmitKey => commands::emit_key::run(),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", "Error:".red().bold());
        std::process::exit(1);
    }
}
