use colored::Colorize;
use dialoguer::Confirm;

use crate::error::{CswitchError, Result};
use crate::keychain;
use crate::profile::{ProfileStore, ProfileType};

pub fn run(name: String, force: bool) -> Result<()> {
    let mut store = ProfileStore::load()?;

    // Check it exists before prompting
    let profile = store.get_profile(&name)?.clone();

    if !force {
        let confirmed = Confirm::new()
            .with_prompt(format!("Remove profile '{name}'?"))
            .default(false)
            .interact()
            .map_err(|e| CswitchError::Config(format!("Input error: {e}")))?;

        if !confirmed {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Delete from keychain
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
