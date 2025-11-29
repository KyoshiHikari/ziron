//! Configuration loading and management

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
    #[serde(default)]
    pub completion: CompletionConfig,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub theme: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shell: ShellConfig::default(),
            performance: PerformanceConfig::default(),
            completion: CompletionConfig::default(),
            modules: vec![],
            theme: Some("default".to_string()), // Standard-Theme: ziron-default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(default = "default_shell")]
    pub default: String,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            default: default_shell(),
        }
    }
}

fn default_shell() -> String {
    "zsh".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_ttl_ms: default_cache_ttl(),
        }
    }
}

fn default_cache_ttl() -> u64 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionConfig {
    #[serde(default = "default_partial_completion")]
    pub partial_completion: bool,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            partial_completion: default_partial_completion(),
        }
    }
}

fn default_partial_completion() -> bool {
    true // Enable by default
}

impl Config {
    /// Load configuration from the default location (~/.config/ziron/config.toml)
    pub fn load() -> Result<Self> {
        let config_path = Self::default_path()?;
        Self::load_from(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// Get the default configuration path
    pub fn default_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| Error::Config("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home).join(".config").join("ziron").join("config.toml"))
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_path()?;
        self.save_to(&config_path)
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Config(format!("Failed to create config directory: {}", e)))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.shell.default, "zsh");
        assert_eq!(config.performance.cache_ttl_ms, 50);
    }

    #[test]
    fn test_load_save_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            shell: ShellConfig {
                default: "bash".to_string(),
            },
            performance: PerformanceConfig { cache_ttl_ms: 100 },
            completion: CompletionConfig::default(),
            modules: vec!["git".to_string(), "sysinfo".to_string()],
            theme: Some("default".to_string()),
        };

        config.save_to(&config_path).unwrap();
        let loaded = Config::load_from(&config_path).unwrap();

        assert_eq!(loaded.shell.default, "bash");
        assert_eq!(loaded.performance.cache_ttl_ms, 100);
        assert_eq!(loaded.modules.len(), 2);
        assert_eq!(loaded.theme, Some("default".to_string()));
    }
}

