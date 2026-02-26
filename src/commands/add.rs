use chrono::Utc;
use colored::Colorize;
use dialoguer::{Input, Password, Select};

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
    let auth_options = &["API Key", "OAuth"];
    let auth_choice = Select::new()
        .with_prompt("Authentication type")
        .items(auth_options)
        .default(0)
        .interact()
        .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

    let profile_type = match auth_choice {
        0 => ProfileType::ApiKey,
        _ => ProfileType::OAuth,
    };

    // 3. Get credentials
    match profile_type {
        ProfileType::ApiKey => {
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
        ProfileType::OAuth => {
            run_oauth(&name)?;
        }
    }

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
