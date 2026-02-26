use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cswitch", version, about = "Switch between Anthropic/Claude accounts")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new profile (interactive)
    Add {
        /// Profile name (prompted if omitted)
        name: Option<String>,
    },

    /// Switch to a profile (interactive selector)
    Use {
        /// Profile name (prompted if omitted)
        name: Option<String>,
    },

    /// List all profiles
    List,

    /// Show the current active profile
    Current,

    /// Remove a profile
    Remove {
        /// Profile name (prompted if omitted)
        name: Option<String>,
    },

    /// Import existing Claude Code credentials
    Import {
        /// Profile name to save as (prompted if omitted)
        name: Option<String>,
    },

    /// Initialize cswitch (create config directory)
    Init,

    /// Update cswitch to the latest version
    Update,

    /// [hidden] Emit the active API key for apiKeyHelper
    #[command(hide = true)]
    EmitKey,
}
