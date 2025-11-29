//! Module registry and plugin system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{Error, Result};

/// Module manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub hooks: Vec<String>,
}

/// Module registry
#[derive(Debug, Default, Clone)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleManifest>,
    module_paths: HashMap<String, PathBuf>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a module from a manifest file
    pub fn register(&mut self, manifest_path: &PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(manifest_path)
            .map_err(|e| Error::Module(format!("Failed to read manifest: {}", e)))?;

        let manifest: ModuleManifest = toml::from_str(&content)
            .map_err(|e| Error::Module(format!("Failed to parse manifest: {}", e)))?;

        let module_dir = manifest_path
            .parent()
            .ok_or_else(|| Error::Module("Manifest path has no parent".to_string()))?;

        self.modules.insert(manifest.name.clone(), manifest.clone());
        self.module_paths.insert(manifest.name.clone(), module_dir.to_path_buf());

        Ok(())
    }

    /// Get a module manifest by name
    pub fn get(&self, name: &str) -> Option<&ModuleManifest> {
        self.modules.get(name)
    }

    /// Get all registered modules
    pub fn all(&self) -> &HashMap<String, ModuleManifest> {
        &self.modules
    }

    /// Get the path for a module
    pub fn get_path(&self, name: &str) -> Option<&PathBuf> {
        self.module_paths.get(name)
    }

    /// Check if a module is registered
    pub fn contains(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }
}

/// Module context passed to plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleContext {
    pub current_dir: PathBuf,
    pub shell: String,
    pub user: String,
    pub hostname: String,
    pub exit_code: Option<i32>,
}

impl ModuleContext {
    /// Create a new module context from the current environment
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            current_dir: std::env::current_dir()
                .map_err(|e| Error::Io(e))?,
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
            user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            hostname: std::env::var("HOSTNAME").unwrap_or_else(|_| {
                hostname::get()
                    .ok()
                    .and_then(|h| h.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "unknown".to_string())
            }),
            exit_code: None,
        })
    }
}

/// Module data returned by plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleData {
    pub module: String,
    pub data: serde_json::Value,
    pub cached: bool,
}

