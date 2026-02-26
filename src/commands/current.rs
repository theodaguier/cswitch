use colored::Colorize;

use crate::error::Result;
use crate::keychain;
use crate::profile::{ProfileStore, ProfileType};

pub fn run() -> Result<()> {
    let store = ProfileStore::load()?;
    let profile = store.get_active()?;

    let masked_credential = match profile.profile_type {
        ProfileType::ApiKey => {
            match keychain::get_api_key(&profile.name) {
                Ok(key) => {
                    if key.len() > 10 {
                        let start = &key[..7];
                        let end = &key[key.len() - 4..];
                        format!("{}...{}", start, end)
                    } else {
                        "***".to_string()
                    }
                }
                Err(_) => "key not found in keychain".to_string(),
            }
        }
        ProfileType::OAuth => "oauth token".to_string(),
    };

    let label_str = profile
        .label
        .as_deref()
        .map(|l| format!(", {l}"))
        .unwrap_or_default();

    println!(
        "{} {} ({}{}, {})",
        "Active:".bold(),
        profile.name.green().bold(),
        profile.profile_type,
        label_str,
        masked_credential.dimmed()
    );

    Ok(())
}
