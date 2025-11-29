//! Node.js module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::fs;
use std::process::Command;

/// Node.js module implementation
pub struct NodeModule;

impl NodeModule {
    /// Fetch Node.js version information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let node_info = Self::get_node_info(&context.current_dir)?;

        let mut parts = vec![];
        
        if let Some(ref version) = node_info.version {
            parts.push(version.clone());
        }
        
        if let Some(ref pm) = node_info.package_manager {
            parts.push(pm.clone());
        }

        Ok(ModuleData {
            module: "node".to_string(),
            data: serde_json::json!({
                "text": parts.join(" "),
                "version": node_info.version,
                "package_manager": node_info.package_manager,
                "has_package_json": node_info.has_package_json,
            }),
            cached: false,
        })
    }

    fn get_node_info(path: &PathBuf) -> Result<NodeInfo> {
        // Check for .nvmrc or .node-version
        let version_from_file = Self::read_version_file(path)?;
        
        // Get version from system if not in file
        let version = version_from_file.or_else(|| {
            Command::new("node")
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
        });

        // Detect package manager
        let package_manager = Self::detect_package_manager(path)?;
        
        // Check for package.json
        let has_package_json = path.join("package.json").exists();

        Ok(NodeInfo {
            version,
            package_manager,
            has_package_json,
        })
    }

    fn read_version_file(path: &PathBuf) -> Result<Option<String>> {
        // Check .nvmrc
        let nvmrc = path.join(".nvmrc");
        if nvmrc.exists() {
            if let Ok(content) = fs::read_to_string(&nvmrc) {
                let version = content.trim().to_string();
                if !version.is_empty() {
                    return Ok(Some(format!("v{}", version)));
                }
            }
        }

        // Check .node-version
        let node_version = path.join(".node-version");
        if node_version.exists() {
            if let Ok(content) = fs::read_to_string(&node_version) {
                let version = content.trim().to_string();
                if !version.is_empty() {
                    return Ok(Some(format!("v{}", version)));
                }
            }
        }

        Ok(None)
    }

    fn detect_package_manager(path: &PathBuf) -> Result<Option<String>> {
        // Check for lock files
        if path.join("yarn.lock").exists() {
            return Ok(Some("yarn".to_string()));
        }
        if path.join("pnpm-lock.yaml").exists() {
            return Ok(Some("pnpm".to_string()));
        }
        if path.join("package-lock.json").exists() {
            return Ok(Some("npm".to_string()));
        }
        
        // Check for package.json to see if it's a workspace
        let package_json = path.join("package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if content.contains("\"workspaces\"") || content.contains("\"workspace\"") {
                    return Ok(Some("workspace".to_string()));
                }
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct NodeInfo {
    version: Option<String>,
    package_manager: Option<String>,
    has_package_json: bool,
}

