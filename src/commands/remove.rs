use colored::Colorize;
use dialoguer::{Confirm, Select};

use crate::error::{CswitchError, Result};
use crate::keychain;
use crate::profile::{ProfileStore, ProfileType};

pub fn run(name: Option<String>) -> Result<()> {
    let mut store = ProfileStore::load()?;

    if store.profiles.is_empty() {
        println!("No profiles to remove.");
        return Ok(());
    }

    // If no name provided, show interactive selector
    let name = match name {
        Some(n) => n,
        None => {
            let mut profiles: Vec<_> = store.profiles.values().collect();
            profiles.sort_by(|a, b| a.name.cmp(&b.name));

            let items: Vec<String> = profiles
                .iter()
                .map(|p| {
                    let label = p.label.as_deref().unwrap_or("");
                    format!("{} ({}) {}", p.name, p.profile_type, label)
                })
                .collect();

            let choice = Select::new()
                .with_prompt("Remove which profile")
                .items(&items)
                .interact()
                .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

            profiles[choice].name.clone()
        }
    };

    let profile = store.get_profile(&name)?.clone();

    let confirmed = Confirm::new()
        .with_prompt(format!("Remove profile '{name}'?"))
        .default(false)
        .interact()
        .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

    if !confirmed {
        println!("Aborted.");
        return Ok(());
    }

    match profile.profile_type {
        ProfileType::ApiKey => {
            let _ = keychain::delete_api_key(&name);
        }
        ProfileType::OAuth => {
            let _ = keychain::delete_oauth_token(&name);
        }
    }

    store.remove_profile(&name)?;

    println!("{} Profile '{}' removed.", "✓".green().bold(), name);
    Ok(())
}
