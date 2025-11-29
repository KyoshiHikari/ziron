//! Go module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::fs;
use std::process::Command;

/// Go module implementation
pub struct GoModule;

impl GoModule {
    /// Fetch Go module information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let go_info = Self::get_go_info(&context.current_dir)?;

        if let Some(info) = go_info {
            let mut parts = vec![];
            
            if let Some(ref version) = info.version {
                parts.push(version.clone());
            }
            
            if let Some(ref module) = info.module_name {
                parts.push(module.clone());
            }

            Ok(ModuleData {
                module: "go".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "version": info.version,
                    "module": info.module_name,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "go".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "version": null,
                }),
                cached: false,
            })
        }
    }

    fn get_go_info(path: &PathBuf) -> Result<Option<GoInfo>> {
        // Check for go.mod
        let go_mod = path.join("go.mod");
        if !go_mod.exists() {
            return Ok(None);
        }

        // Read go.mod to get module name and Go version
        let content = fs::read_to_string(&go_mod)?;
        let mut module_name = None;
        let mut go_version = None;

        for line in content.lines() {
            if line.starts_with("module ") {
                module_name = line.split_whitespace().nth(1).map(|s| s.to_string());
            } else if line.starts_with("go ") {
                go_version = line.split_whitespace().nth(1).map(|s| s.to_string());
            }
        }

        // Get Go version from system if not in go.mod
        let version = go_version.or_else(|| {
            Command::new("go")
                .args(&["version"])
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        let output = String::from_utf8_lossy(&o.stdout);
                        // Extract version like "go1.21.0" -> "1.21.0"
                        output
                            .split_whitespace()
                            .nth(2)
                            .and_then(|v| v.strip_prefix("go"))
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
        });

        if module_name.is_some() || version.is_some() {
            Ok(Some(GoInfo {
                version,
                module_name,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
struct GoInfo {
    version: Option<String>,
    module_name: Option<String>,
}

