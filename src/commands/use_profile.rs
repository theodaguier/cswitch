use colored::Colorize;

use crate::claude_config;
use crate::error::Result;
use crate::keychain;
use crate::profile::{ProfileStore, ProfileType};

pub fn run(name: String) -> Result<()> {
    let mut store = ProfileStore::load()?;
    let profile = store.get_profile(&name)?.clone();

    // Warn if ANTHROPIC_API_KEY is set
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        eprintln!(
            "{} ANTHROPIC_API_KEY env var is set and will override cswitch. Consider unsetting it.",
            "Warning:".yellow().bold()
        );
    }

    match profile.profile_type {
        ProfileType::ApiKey => {
            // Ensure the key exists in keychain
            let _ = keychain::get_api_key(&name)?;
            // Set apiKeyHelper in settings.json
            claude_config::enable_api_key_helper()?;
        }
        ProfileType::OAuth => {
            // Get the stored OAuth token and swap it into Claude Code credentials
            let token = keychain::get_oauth_token(&name)?;
            keychain::set_claude_credentials(&token)?;
            // Remove apiKeyHelper so Claude Code uses OAuth
            claude_config::disable_api_key_helper()?;
        }
    }

    store.set_active(&name)?;

    println!(
        "{} Switched to '{}' ({}).",
        "✓".green().bold(),
        name,
        profile.profile_type
    );
    Ok(())
}
