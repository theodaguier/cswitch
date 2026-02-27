use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::error::{CswitchError, Result};

const CLAUDE_KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

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

fn claude_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string())
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

// --- Claude Code credentials (via macOS `security` CLI, no dialog) ---

pub fn get_claude_credentials() -> Result<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", CLAUDE_KEYCHAIN_SERVICE, "-a", &claude_user(), "-w"])
        .output()
        .map_err(|e| CswitchError::Keychain(format!("Failed to run security: {e}")))?;

    if !output.status.success() {
        return Err(CswitchError::Keychain("No Claude Code credentials found in Keychain".into()));
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| CswitchError::Keychain(format!("Invalid credentials encoding: {e}")))
}

pub fn set_claude_credentials(token_json: &str) -> Result<()> {
    // Delete existing entry first (security add fails if it exists)
    let _ = std::process::Command::new("security")
        .args(["delete-generic-password", "-s", CLAUDE_KEYCHAIN_SERVICE, "-a", &claude_user()])
        .output();

    let status = std::process::Command::new("security")
        .args(["add-generic-password", "-s", CLAUDE_KEYCHAIN_SERVICE, "-a", &claude_user(), "-w", token_json, "-U"])
        .status()
        .map_err(|e| CswitchError::Keychain(format!("Failed to run security: {e}")))?;

    if !status.success() {
        return Err(CswitchError::Keychain("Failed to write credentials to Keychain".into()));
    }
    Ok(())
}
