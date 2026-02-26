use chrono::Utc;
use colored::Colorize;
use dialoguer::Input;

use crate::error::{CswitchError, Result};
use crate::keychain;
use crate::profile::{Profile, ProfileStore, ProfileType};

pub fn run(name: Option<String>) -> Result<()> {
    let mut store = ProfileStore::load()?;

    let name = match name {
        Some(n) => n,
        None => Input::new()
            .with_prompt("Profile name for imported credentials")
            .interact_text()
            .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?,
    };

    if store.profiles.contains_key(&name) {
        return Err(CswitchError::ProfileAlreadyExists(name));
    }

    let creds = keychain::get_claude_credentials().map_err(|_| {
        CswitchError::Keychain(
            "No existing Claude Code credentials found in Keychain. Make sure you're logged in to Claude Code first.".into(),
        )
    })?;

    keychain::set_oauth_token(&name, &creds)?;

    let label: String = Input::new()
        .with_prompt("Label (optional)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

    let label = if label.is_empty() { None } else { Some(label) };

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
