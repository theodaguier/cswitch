use colored::Colorize;

use crate::error::Result;
use crate::profile::ProfileStore;

pub fn run() -> Result<()> {
    let store = ProfileStore::load()?;

    if store.profiles.is_empty() {
        println!("No profiles configured. Run 'cswitch add <name> --api-key' to add one.");
        return Ok(());
    }

    let mut profiles: Vec<_> = store.profiles.values().collect();
    profiles.sort_by(|a, b| a.name.cmp(&b.name));

    for profile in profiles {
        let is_active = store.active.as_deref() == Some(&profile.name);
        let marker = if is_active {
            "*".green().bold().to_string()
        } else {
            " ".to_string()
        };

        let label = profile
            .label
            .as_deref()
            .unwrap_or("")
            .to_string();

        println!(
            "{} {:<12} {:<10} {}",
            marker,
            if is_active {
                profile.name.bold().to_string()
            } else {
                profile.name.clone()
            },
            profile.profile_type.to_string().dimmed(),
            label.dimmed()
        );
    }

    Ok(())
}
