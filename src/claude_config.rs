use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::error::{CswitchError, Result};

const API_KEY_HELPER_CMD: &str = "cswitch emit-key";

/// Return the path to ~/.claude/settings.json
pub fn settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| CswitchError::ClaudeSettings("Cannot determine home directory".into()))?;
    Ok(home.join(".claude").join("settings.json"))
}

/// Read settings.json as a serde_json::Value, or return an empty object if it doesn't exist.
pub fn read_settings() -> Result<Value> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| CswitchError::ClaudeSettings(format!("read error: {e}")))?;
    let val: Value = serde_json::from_str(&data)
        .map_err(|e| CswitchError::ClaudeSettings(format!("parse error: {e}")))?;
    Ok(val)
}

/// Write settings.json, creating the directory if needed.
fn write_settings(val: &Value) -> Result<()> {
    let path = settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(val)
        .map_err(|e| CswitchError::ClaudeSettings(format!("serialize error: {e}")))?;
    fs::write(&path, data)
        .map_err(|e| CswitchError::ClaudeSettings(format!("write error: {e}")))?;
    Ok(())
}

/// Set `apiKeyHelper` in settings.json to point to `cswitch emit-key`.
pub fn enable_api_key_helper() -> Result<()> {
    let mut settings = read_settings()?;
    let obj = settings
        .as_object_mut()
        .ok_or_else(|| CswitchError::ClaudeSettings("settings.json is not an object".into()))?;
    obj.insert(
        "apiKeyHelper".to_string(),
        Value::String(API_KEY_HELPER_CMD.to_string()),
    );
    write_settings(&settings)
}

/// Remove `apiKeyHelper` from settings.json.
pub fn disable_api_key_helper() -> Result<()> {
    let mut settings = read_settings()?;
    let obj = settings
        .as_object_mut()
        .ok_or_else(|| CswitchError::ClaudeSettings("settings.json is not an object".into()))?;
    obj.remove("apiKeyHelper");
    write_settings(&settings)
}

/// Check if `apiKeyHelper` is currently set.
#[allow(dead_code)]
pub fn has_api_key_helper() -> Result<bool> {
    let settings = read_settings()?;
    Ok(settings.get("apiKeyHelper").is_some())
}
