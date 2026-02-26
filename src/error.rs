use thiserror::Error;

#[derive(Error, Debug)]
pub enum CswitchError {
    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),

    #[error("Profile '{0}' already exists")]
    ProfileAlreadyExists(String),

    #[error("No active profile set")]
    NoActiveProfile,

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Failed to read/write config: {0}")]
    Config(String),

    #[error("Failed to read/write Claude settings: {0}")]
    ClaudeSettings(String),

    #[allow(dead_code)]
    #[error("Invalid API key format")]
    InvalidApiKey,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "oauth")]
    #[error("OAuth error: {0}")]
    OAuth(String),
}

pub type Result<T> = std::result::Result<T, CswitchError>;
