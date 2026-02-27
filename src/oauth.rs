// Login to Claude by delegating to `claude` CLI.

use colored::Colorize;
use std::process::Command;

use crate::error::{CswitchError, Result};
use crate::keychain;

/// Run `claude login` then capture the credentials from the Keychain.
pub fn run_oauth_flow(profile_name: &str) -> Result<()> {
    // Check that `claude` is installed
    let has_claude = Command::new("claude")
        .arg("--version")
        .output()
        .is_ok();

    if !has_claude {
        return Err(CswitchError::OAuth(
            "Claude Code CLI not found. Install it first: https://docs.anthropic.com/en/docs/claude-code".into(),
        ));
    }

    println!(
        "{} Running 'claude login'...",
        "→".blue().bold()
    );

    let status = Command::new("claude")
        .arg("login")
        .status()
        .map_err(|e| CswitchError::OAuth(format!("Failed to run 'claude login': {e}")))?;

    if !status.success() {
        return Err(CswitchError::OAuth("'claude login' failed".into()));
    }

    // Grab the fresh credentials from the Keychain
    let creds = keychain::get_claude_credentials().map_err(|_| {
        CswitchError::OAuth(
            "Login succeeded but credentials not found in Keychain".into(),
        )
    })?;

    keychain::set_oauth_token(profile_name, &creds)?;

    println!("{} Authentication successful.", "✓".green().bold());
    Ok(())
}
