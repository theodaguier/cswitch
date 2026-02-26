use colored::Colorize;
use std::process::Command;

use crate::error::{CswitchError, Result};

const INSTALL_SCRIPT: &str = "https://raw.githubusercontent.com/theodaguier/cswitch/main/install.sh";

pub fn run() -> Result<()> {
    println!("Checking for updates...");

    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("curl -fsSL {INSTALL_SCRIPT} | sh"))
        .status()
        .map_err(|e| CswitchError::Config(format!("Failed to run updater: {e}")))?;

    if status.success() {
        println!("{} cswitch updated.", "✓".green().bold());
    } else {
        return Err(CswitchError::Config("Update failed".into()));
    }

    Ok(())
}
