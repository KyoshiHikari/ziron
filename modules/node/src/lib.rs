//! Node.js module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::process::Command;

/// Node.js module implementation
pub struct NodeModule;

impl NodeModule {
    /// Fetch Node.js version information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        let version = Self::get_node_version()?;

        let text = version.unwrap_or_default();

        Ok(ModuleData {
            module: "node".to_string(),
            data: serde_json::json!({
                "text": text,
                "version": text.clone(),
            }),
            cached: false,
        })
    }

    fn get_node_version() -> Result<Option<String>> {
        let output = Command::new("node")
            .arg("--version")
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(Some(version))
            }
            _ => Ok(None),
        }
    }
}

