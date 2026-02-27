use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::error::{CswitchError, Result};

const CLAUDE_CREDENTIALS_FILE: &str = ".credentials.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct CredentialStore {
    api_keys: HashMap<String, String>,
    oauth_tokens: HashMap<String, String>,
}

fn credentials_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| CswitchError::Keychain("Cannot determine config directory".into()))?;
    Ok(config_dir.join("cswitch").join("credentials.json"))
}

fn load_store() -> Result<CredentialStore> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(CredentialStore::default());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| CswitchError::Keychain(format!("Failed to read credentials: {e}")))?;
    serde_json::from_str(&data).map_err(|e| CswitchError::Keychain(format!("Invalid credentials file: {e}")))
}

fn save_store(store: &CredentialStore) -> Result<()> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(store)
        .map_err(|e| CswitchError::Keychain(format!("Failed to serialize credentials: {e}")))?;
    fs::write(&path, &data)
        .map_err(|e| CswitchError::Keychain(format!("Failed to write credentials: {e}")))?;
    // Restrict permissions to owner only (600)
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

fn claude_credentials_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| CswitchError::Keychain("Cannot determine home directory".into()))?;
    Ok(home.join(".claude").join(CLAUDE_CREDENTIALS_FILE))
}

// --- API keys ---

pub fn set_api_key(profile_name: &str, api_key: &str) -> Result<()> {
    let mut store = load_store()?;
    store.api_keys.insert(profile_name.to_string(), api_key.to_string());
    save_store(&store)
}

pub fn get_api_key(profile_name: &str) -> Result<String> {
    let store = load_store()?;
    store.api_keys.get(profile_name).cloned()
        .ok_or_else(|| CswitchError::Keychain(format!("No API key found for profile '{profile_name}'")))
}

pub fn delete_api_key(profile_name: &str) -> Result<()> {
    let mut store = load_store()?;
    store.api_keys.remove(profile_name);
    save_store(&store)
}

// --- OAuth tokens ---

pub fn set_oauth_token(profile_name: &str, token_json: &str) -> Result<()> {
    let mut store = load_store()?;
    store.oauth_tokens.insert(profile_name.to_string(), token_json.to_string());
    save_store(&store)
}

pub fn get_oauth_token(profile_name: &str) -> Result<String> {
    let store = load_store()?;
    store.oauth_tokens.get(profile_name).cloned()
        .ok_or_else(|| CswitchError::Keychain(format!("No OAuth token found for profile '{profile_name}'")))
}

pub fn delete_oauth_token(profile_name: &str) -> Result<()> {
    let mut store = load_store()?;
    store.oauth_tokens.remove(profile_name);
    save_store(&store)
}

// --- Claude Code credentials (read/write ~/.claude/.credentials.json) ---

pub fn get_claude_credentials() -> Result<String> {
    let path = claude_credentials_path()?;
    fs::read_to_string(&path)
        .map_err(|_| CswitchError::Keychain("No Claude Code credentials found at ~/.claude/.credentials.json".into()))
}

pub fn set_claude_credentials(token_json: &str) -> Result<()> {
    let path = claude_credentials_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, token_json)
        .map_err(|e| CswitchError::Keychain(format!("Failed to write Claude credentials: {e}")))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
