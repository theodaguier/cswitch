use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{CswitchError, Result};

const OAUTH_TOKEN_ENDPOINT: &str = "https://console.anthropic.com/v1/oauth/token";
const OAUTH_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

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

// --- OAuth token refresh ---

/// Check if the stored OAuth token is expired and refresh it if needed.
/// Returns the (possibly refreshed) token JSON string ready to write to Keychain.
pub fn refresh_oauth_token_if_needed(profile_name: &str) -> Result<String> {
    let token_json = get_oauth_token(profile_name)?;

    let mut creds: Value = serde_json::from_str(&token_json)
        .map_err(|e| CswitchError::OAuth(format!("Invalid token JSON: {e}")))?;

    // Extract values from the immutable borrow before mutating
    let (expires_at, refresh_token) = {
        let oauth = creds
            .get("claudeAiOauth")
            .ok_or_else(|| CswitchError::OAuth("Missing claudeAiOauth in token".into()))?;

        let expires_at = oauth
            .get("expiresAt")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| CswitchError::OAuth("Missing expiresAt in token".into()))?;

        let refresh_token = oauth
            .get("refreshToken")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        (expires_at, refresh_token)
    };

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    // Still valid (with 5-minute buffer) → return as-is
    if now_ms < expires_at - 300_000 {
        return Ok(token_json);
    }

    let refresh_token = refresh_token.ok_or_else(|| {
        CswitchError::OAuth(
            "No refresh token available. Re-authenticate with 'cswitch add --oauth'.".into(),
        )
    })?;

    eprintln!("  Token expired, refreshing…");

    let body = format!(
        "grant_type=refresh_token&client_id={OAUTH_CLIENT_ID}&refresh_token={refresh_token}"
    );

    let output = std::process::Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            OAUTH_TOKEN_ENDPOINT,
            "-H",
            "Content-Type: application/x-www-form-urlencoded",
            "-d",
            &body,
        ])
        .output()
        .map_err(|e| CswitchError::OAuth(format!("Failed to call refresh endpoint: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CswitchError::OAuth(format!(
            "Token refresh request failed: {stderr}. Re-authenticate with 'cswitch add --oauth'."
        )));
    }

    let response: Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        CswitchError::OAuth(format!(
            "Invalid refresh response: {e}. Re-authenticate with 'cswitch add --oauth'."
        ))
    })?;

    // Check for error in response body
    if let Some(error) = response.get("error") {
        return Err(CswitchError::OAuth(format!(
            "Token refresh failed: {}. Re-authenticate with 'cswitch add --oauth'.",
            error.as_str().unwrap_or("unknown error")
        )));
    }

    let new_access_token = response
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CswitchError::OAuth("No access_token in refresh response".into()))?;

    let new_refresh_token = response
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or(&refresh_token);

    let expires_in = response
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);

    let new_expires_at = now_ms + (expires_in * 1000);

    // Update token in-place, preserving scopes/subscriptionType/rateLimitTier
    if let Some(oauth_mut) = creds.get_mut("claudeAiOauth") {
        oauth_mut["accessToken"] = Value::String(new_access_token.to_string());
        oauth_mut["refreshToken"] = Value::String(new_refresh_token.to_string());
        oauth_mut["expiresAt"] = Value::Number(serde_json::Number::from(new_expires_at));
    }

    let new_token_json = serde_json::to_string(&creds)
        .map_err(|e| CswitchError::OAuth(format!("Failed to serialize refreshed token: {e}")))?;

    // Persist the refreshed token so future switches don't need to refresh again
    set_oauth_token(profile_name, &new_token_json)?;

    eprintln!("  Token refreshed successfully.");

    Ok(new_token_json)
}
