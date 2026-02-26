use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{CswitchError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileType {
    ApiKey,
    OAuth,
}

impl std::fmt::Display for ProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileType::ApiKey => write!(f, "api-key"),
            ProfileType::OAuth => write!(f, "oauth"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub profile_type: ProfileType,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileStore {
    pub active: Option<String>,
    pub profiles: HashMap<String, Profile>,
}

impl ProfileStore {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CswitchError::Config("Cannot determine config directory".into()))?;
        let dir = config_dir.join("cswitch");
        Ok(dir.join("profiles.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(&path)?;
        let store: Self = serde_json::from_str(&data)?;
        Ok(store)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;
        Ok(())
    }

    pub fn add_profile(&mut self, profile: Profile) -> Result<()> {
        if self.profiles.contains_key(&profile.name) {
            return Err(CswitchError::ProfileAlreadyExists(profile.name.clone()));
        }
        self.profiles.insert(profile.name.clone(), profile);
        self.save()
    }

    pub fn remove_profile(&mut self, name: &str) -> Result<Profile> {
        let profile = self
            .profiles
            .remove(name)
            .ok_or_else(|| CswitchError::ProfileNotFound(name.into()))?;
        if self.active.as_deref() == Some(name) {
            self.active = None;
        }
        self.save()?;
        Ok(profile)
    }

    pub fn set_active(&mut self, name: &str) -> Result<()> {
        if !self.profiles.contains_key(name) {
            return Err(CswitchError::ProfileNotFound(name.into()));
        }
        self.active = Some(name.to_string());
        if let Some(profile) = self.profiles.get_mut(name) {
            profile.last_used = Some(Utc::now());
        }
        self.save()
    }

    pub fn get_active(&self) -> Result<&Profile> {
        let name = self
            .active
            .as_deref()
            .ok_or(CswitchError::NoActiveProfile)?;
        self.profiles
            .get(name)
            .ok_or_else(|| CswitchError::ProfileNotFound(name.into()))
    }

    pub fn get_profile(&self, name: &str) -> Result<&Profile> {
        self.profiles
            .get(name)
            .ok_or_else(|| CswitchError::ProfileNotFound(name.into()))
    }
}
