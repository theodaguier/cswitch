use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cswitch", version, about = "Switch between Anthropic/Claude accounts")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new profile
    Add {
        /// Profile name
        name: String,

        /// Add an API key profile
        #[arg(long, group = "profile_type")]
        api_key: bool,

        /// Add an OAuth profile
        #[arg(long, group = "profile_type")]
        oauth: bool,

        /// Optional label for this profile
        #[arg(long, short)]
        label: Option<String>,
    },

    /// Switch to a profile
    Use {
        /// Profile name to switch to
        name: String,
    },

    /// List all profiles
    List,

    /// Show the current active profile
    Current,

    /// Remove a profile
    Remove {
        /// Profile name to remove
        name: String,

        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,
    },

    /// Import existing Claude Code credentials
    Import {
        /// Profile name to save as
        name: String,

        /// Optional label
        #[arg(long, short)]
        label: Option<String>,
    },

    /// Initialize cswitch (create config directory)
    Init,

    /// [hidden] Emit the active API key for apiKeyHelper
    #[command(hide = true)]
    EmitKey,
}
