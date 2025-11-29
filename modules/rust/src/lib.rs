//! Rust toolchain module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::process::Command;

/// Rust toolchain module implementation
pub struct RustModule;

impl RustModule {
    /// Fetch Rust toolchain version information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        let version = Self::get_rust_version()?;

        let text = version.unwrap_or_default();

        Ok(ModuleData {
            module: "rust".to_string(),
            data: serde_json::json!({
                "text": text,
                "version": text.clone(),
            }),
            cached: false,
        })
    }

    fn get_rust_version() -> Result<Option<String>> {
        let output = Command::new("rustc")
            .arg("--version")
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                // Extract just the version number (e.g., "rustc 1.70.0" -> "1.70.0")
                let version = version.split_whitespace().nth(1).unwrap_or(&version).to_string();
                Ok(Some(version))
            }
            _ => Ok(None),
        }
    }
}

