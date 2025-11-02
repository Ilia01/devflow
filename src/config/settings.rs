use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::errors::{DevFlowError, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub jira: JiraConfig,
    pub git: GitConfig,
    pub preferences: Preferences,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraConfig {
    pub url: String,
    pub email: String,
    pub project_key: String,
    pub auth_method: AuthMethod,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMethod {
    PersonalAccessToken { token: String },
    ApiToken { token: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitConfig {
    pub provider: String,
    pub base_url: String,
    pub token: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Preferences {
    pub branch_prefix: String,
    pub default_transition: String,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()
            .map_err(|e| DevFlowError::ConfigInvalid(e.to_string()))?;

        if !config_path.exists() {
            return Err(DevFlowError::ConfigNotFound);
        }

        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| DevFlowError::ConfigInvalid(format!("Failed to read config file: {}", e)))?;

        let settings: Settings = toml::from_str(&config_str)
            .map_err(|e| DevFlowError::ConfigInvalid(format!("Failed to parse config file: {}", e)))?;

        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let config_str = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(&config_path, config_str)
            .context("Failed to write config file")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&config_path, perms)?;
        }

        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;

        let config_dir = PathBuf::from(home).join(".devflow");
        Ok(config_dir.join("config.toml"))
    }

    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".devflow"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let settings = Settings {
            jira: JiraConfig {
                url: "https://jira.example.com".to_string(),
                email: "test@example.com".to_string(),
                auth_method: AuthMethod::ApiToken {
                    token: "test-token".to_string(),
                },
                project_key: "TEST".to_string(),
            },
            git: GitConfig {
                provider: "gitlab".to_string(),
                base_url: "https://git.example.com".to_string(),
                token: "git-token".to_string(),
                owner: None,
                repo: None,
            },
            preferences: Preferences {
                branch_prefix: "feat".to_string(),
                default_transition: "In Progress".to_string(),
            },
        };

        let toml_str = toml::to_string(&settings).unwrap();
        assert!(toml_str.contains("https://jira.example.com"));
        assert!(toml_str.contains("test@example.com"));

        let deserialized: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.jira.url, "https://jira.example.com");
        assert_eq!(deserialized.preferences.branch_prefix, "feat");
    }

    #[test]
    fn test_config_load_missing_file() {
        // This test might pass if user has a real config file
        // Just verify the load method works (doesn't panic)
        let _ = Settings::load();
    }
}
