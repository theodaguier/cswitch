use chrono::Utc;
use colored::Colorize;

use crate::error::{CswitchError, Result};
use crate::keychain;
use crate::profile::{Profile, ProfileStore, ProfileType};

/// Import existing Claude Code OAuth credentials from the macOS Keychain.
pub fn run(name: String, label: Option<String>) -> Result<()> {
    let mut store = ProfileStore::load()?;

    if store.profiles.contains_key(&name) {
        return Err(CswitchError::ProfileAlreadyExists(name));
    }

    // Read current Claude Code credentials
    let creds = keychain::get_claude_credentials().map_err(|_| {
        CswitchError::Keychain(
            "No existing Claude Code credentials found in Keychain. Make sure you're logged in to Claude Code first.".into(),
        )
    })?;

    // Store a copy under cswitch's own keychain entry
    keychain::set_oauth_token(&name, &creds)?;

    let profile = Profile {
        name: name.clone(),
        profile_type: ProfileType::OAuth,
        label,
        created_at: Utc::now(),
        last_used: None,
    };

    store.add_profile(profile)?;

    println!(
        "{} Imported Claude Code credentials as profile '{}'.",
        "✓".green().bold(),
        name
    );
    Ok(())
}
