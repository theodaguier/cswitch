use chrono::Utc;
use colored::Colorize;
use dialoguer::{Input, Password, Select};

use crate::error::{CswitchError, Result};
use crate::keychain;
use crate::profile::{Profile, ProfileStore, ProfileType};


pub fn run(name: Option<String>) -> Result<()> {
    let mut store = ProfileStore::load()?;

    // 1. Ask for name
    let name = match name {
        Some(n) => n,
        None => Input::new()
            .with_prompt("Profile name")
            .interact_text()
            .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?,
    };

    if store.profiles.contains_key(&name) {
        return Err(CswitchError::ProfileAlreadyExists(name));
    }

    // 2. Ask for auth type
    let auth_options = vec![
        "API Key",
        "OAuth (login via browser)",
        "Import from Claude Code (existing login)",
    ];

    let auth_choice = Select::new()
        .with_prompt("Authentication type")
        .items(&auth_options)
        .default(0)
        .interact()
        .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

    let selected = auth_options[auth_choice];

    // 3. Get credentials
    let profile_type = if selected == "API Key" {
        let key = Password::new()
            .with_prompt("Anthropic API key")
            .interact()
            .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

        if !key.starts_with("sk-ant-") {
            eprintln!(
                "{} Key doesn't start with 'sk-ant-'. Storing anyway.",
                "Warning:".yellow().bold()
            );
        }

        keychain::set_api_key(&name, &key)?;
        ProfileType::ApiKey
    } else if selected.starts_with("Import") {
        let creds = keychain::get_claude_credentials().map_err(|_| {
            CswitchError::Keychain(
                "No Claude Code credentials found in Keychain. Log in to Claude Code first.".into(),
            )
        })?;
        keychain::set_oauth_token(&name, &creds)?;
        ProfileType::OAuth
    } else {
        crate::oauth::run_oauth_flow(&name)?;
        ProfileType::OAuth
    };

    // 4. Ask for optional label
    let label: String = Input::new()
        .with_prompt("Label (optional)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

    let label = if label.is_empty() { None } else { Some(label) };

    let profile = Profile {
        name: name.clone(),
        profile_type,
        label,
        created_at: Utc::now(),
        last_used: None,
    };

    store.add_profile(profile)?;

    println!("{} Profile '{}' added.", "✓".green().bold(), name);
    Ok(())
}
