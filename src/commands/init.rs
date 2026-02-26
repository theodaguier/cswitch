use colored::Colorize;
use std::fs;

use crate::error::Result;
use crate::profile::ProfileStore;

pub fn run() -> Result<()> {
    let path = ProfileStore::config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        println!("Config already exists at {}", path.display());
    } else {
        let store = ProfileStore::default();
        store.save()?;
        println!(
            "{} Initialized cswitch config at {}",
            "✓".green().bold(),
            path.display()
        );
    }

    Ok(())
}
