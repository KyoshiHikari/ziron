//! Theme system for prompt rendering

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(rename = "theme")]
    pub config: ThemeConfig,
    #[serde(default, rename = "segments")]
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    /// Background color as hex code (e.g., "#15161e")
    #[serde(default)]
    pub background: Option<String>,
    /// Multi-line prompt support
    #[serde(default)]
    pub multiline: Option<bool>,
    /// Right-side prompt segments
    #[serde(default, rename = "right_segments")]
    pub right_segments: Option<Vec<Segment>>,
    /// Show prompt timing information
    #[serde(default)]
    pub show_timing: Option<bool>,
    /// Custom color palette
    #[serde(default)]
    pub color_palette: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub module: String,
    pub color: Option<String>,
    pub separator: Option<String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub condition: String,
    pub value: serde_json::Value,
}

impl Theme {
    /// Load a theme from a TOML file
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Theme(format!("Failed to read theme file: {}", e)))?;

        let theme: Theme = toml::from_str(&content)
            .map_err(|e| Error::Theme(format!("Failed to parse theme: {}", e)))?;
        
        Ok(theme)
    }

    /// Get the default theme path
    pub fn default_path() -> Result<PathBuf> {
        Ok(PathBuf::from("themes").join("default").join("theme.toml"))
    }
}

