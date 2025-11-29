//! Terraform module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::process::Command;

/// Terraform module implementation
pub struct TerraformModule;

impl TerraformModule {
    /// Fetch Terraform workspace information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let tf_info = Self::get_terraform_info(&context.current_dir)?;

        if let Some(info) = tf_info {
            let mut parts = vec![];
            
            if let Some(ref workspace) = info.workspace {
                parts.push(workspace.clone());
            }

            Ok(ModuleData {
                module: "terraform".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "workspace": info.workspace,
                    "version": info.version,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "terraform".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "workspace": null,
                }),
                cached: false,
            })
        }
    }

    fn get_terraform_info(path: &PathBuf) -> Result<Option<TerraformInfo>> {
        // Check for .terraform directory
        let tf_dir = path.join(".terraform");
        if !tf_dir.exists() {
            return Ok(None);
        }

        // Get workspace
        let workspace_output = Command::new("terraform")
            .args(&["workspace", "show"])
            .current_dir(path)
            .output();

        let workspace = workspace_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let ws = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !ws.is_empty() && ws != "default" {
                        Some(ws)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        // Get version
        let version_output = Command::new("terraform")
            .args(&["version", "-json"])
            .current_dir(path)
            .output();

        let version = version_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    // Parse JSON to get version
                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&o.stdout) {
                        json.get("terraform_version")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        if workspace.is_some() {
            Ok(Some(TerraformInfo {
                workspace,
                version,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
struct TerraformInfo {
    workspace: Option<String>,
    version: Option<String>,
}

