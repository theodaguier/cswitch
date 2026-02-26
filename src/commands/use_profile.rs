use colored::Colorize;
use dialoguer::Select;

use crate::claude_config;
use crate::error::{CswitchError, Result};
use crate::keychain;
use crate::profile::{ProfileStore, ProfileType};

pub fn run(name: Option<String>) -> Result<()> {
    let mut store = ProfileStore::load()?;

    if store.profiles.is_empty() {
        println!("No profiles configured. Run 'cswitch add' to create one.");
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
                    let active = if store.active.as_deref() == Some(&p.name) {
                        "* "
                    } else {
                        "  "
                    };
                    let label = p.label.as_deref().unwrap_or("");
                    format!("{}{} ({}) {}", active, p.name, p.profile_type, label)
                })
                .collect();

            let default = profiles
                .iter()
                .position(|p| store.active.as_deref() == Some(&p.name))
                .unwrap_or(0);

            let choice = Select::new()
                .with_prompt("Switch to")
                .items(&items)
                .default(default)
                .interact()
                .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

            profiles[choice].name.clone()
        }
    };

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
            let _ = keychain::get_api_key(&name)?;
            claude_config::enable_api_key_helper()?;
        }
        ProfileType::OAuth => {
            let token = keychain::get_oauth_token(&name)?;
            keychain::set_claude_credentials(&token)?;
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
