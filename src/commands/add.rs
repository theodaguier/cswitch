use chrono::Utc;
use colored::Colorize;
use dialoguer::Password;

use crate::error::{CswitchError, Result};
use crate::keychain;
use crate::profile::{Profile, ProfileStore, ProfileType};

#[cfg(feature = "oauth")]
fn run_oauth(name: &str) -> Result<()> {
    crate::oauth::run_oauth_flow(name)
}

#[cfg(not(feature = "oauth"))]
fn run_oauth(_name: &str) -> Result<()> {
    eprintln!(
        "{} OAuth support requires the 'oauth' feature. Rebuild with: cargo build --features oauth",
        "Error:".red().bold()
    );
    Err(CswitchError::Config("OAuth feature not enabled".into()))
}

pub fn run(name: String, api_key: bool, oauth: bool, label: Option<String>) -> Result<()> {
    let mut store = ProfileStore::load()?;

    if store.profiles.contains_key(&name) {
        return Err(CswitchError::ProfileAlreadyExists(name));
    }

    let profile_type = if api_key {
        ProfileType::ApiKey
    } else if oauth {
        run_oauth(&name)?;
        ProfileType::OAuth
    } else {
        // Default to API key if neither flag is provided
        ProfileType::ApiKey
    };

    if profile_type == ProfileType::ApiKey {
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
    }

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
