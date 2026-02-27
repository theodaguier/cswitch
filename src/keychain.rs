use keyring::Entry;

use crate::error::{CswitchError, Result};

const SERVICE_NAME: &str = "cswitch";
const CLAUDE_SERVICE: &str = "Claude Code-credentials";

fn claude_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string())
}

fn keychain_error(e: keyring::Error) -> CswitchError {
    CswitchError::Keychain(e.to_string())
}

/// Store an API key for a profile in the OS keychain.
pub fn set_api_key(profile_name: &str, api_key: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, profile_name).map_err(keychain_error)?;
    entry.set_password(api_key).map_err(keychain_error)?;
    Ok(())
}

/// Retrieve an API key for a profile from the OS keychain.
pub fn get_api_key(profile_name: &str) -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, profile_name).map_err(keychain_error)?;
    entry.get_password().map_err(keychain_error)
}

/// Delete an API key for a profile from the OS keychain.
pub fn delete_api_key(profile_name: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, profile_name).map_err(keychain_error)?;
    entry.delete_credential().map_err(keychain_error)?;
    Ok(())
}

/// Store an OAuth token JSON for a profile in the OS keychain.
pub fn set_oauth_token(profile_name: &str, token_json: &str) -> Result<()> {
    let key = format!("{}-oauth", profile_name);
    let entry = Entry::new(SERVICE_NAME, &key).map_err(keychain_error)?;
    entry.set_password(token_json).map_err(keychain_error)?;
    Ok(())
}

/// Retrieve an OAuth token JSON for a profile from the OS keychain.
pub fn get_oauth_token(profile_name: &str) -> Result<String> {
    let key = format!("{}-oauth", profile_name);
    let entry = Entry::new(SERVICE_NAME, &key).map_err(keychain_error)?;
    entry.get_password().map_err(keychain_error)
}

/// Delete an OAuth token for a profile from the OS keychain.
pub fn delete_oauth_token(profile_name: &str) -> Result<()> {
    let key = format!("{}-oauth", profile_name);
    let entry = Entry::new(SERVICE_NAME, &key).map_err(keychain_error)?;
    entry.delete_credential().map_err(keychain_error)?;
    Ok(())
}

/// Read the current Claude Code credentials from the Keychain.
pub fn get_claude_credentials() -> Result<String> {
    let entry = Entry::new(CLAUDE_SERVICE, &claude_user()).map_err(keychain_error)?;
    entry.get_password().map_err(keychain_error)
}

/// Write OAuth credentials to the Claude Code Keychain entry.
pub fn set_claude_credentials(token_json: &str) -> Result<()> {
    let entry = Entry::new(CLAUDE_SERVICE, &claude_user()).map_err(keychain_error)?;
    entry.set_password(token_json).map_err(keychain_error)?;
    Ok(())
}
