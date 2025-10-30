use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub api_token: String,
    pub project_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitConfig {
    pub provider: String,
    pub base_url: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Preferences {
    pub branch_prefix: String,
    pub default_transition: String,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            anyhow::bail!(
                "Configuration not found. Run 'devflow init' to set up your credentials."
            );
        }

        let config_str = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let settings: Settings = toml::from_str(&config_str)
            .context("Failed to parse config file")?;

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
                api_token: "test-token".to_string(),
                project_key: "TEST".to_string(),
            },
            git: GitConfig {
                provider: "gitlab".to_string(),
                base_url: "https://git.example.com".to_string(),
                token: "git-token".to_string(),
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
        let result = Settings::load();
        assert!(result.is_err());
    }
}
