//! Rust toolchain module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::fs;
use std::process::Command;

/// Rust toolchain module implementation
pub struct RustModule;

impl RustModule {
    /// Fetch Rust toolchain version information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let rust_info = Self::get_rust_info(&context.current_dir)?;

        let mut parts = vec![];
        
        if let Some(ref version) = rust_info.version {
            parts.push(version.clone());
        }
        
        if rust_info.is_workspace {
            parts.push("workspace".to_string());
        }

        Ok(ModuleData {
            module: "rust".to_string(),
            data: serde_json::json!({
                "text": parts.join(" "),
                "version": rust_info.version,
                "is_workspace": rust_info.is_workspace,
                "has_cargo_toml": rust_info.has_cargo_toml,
            }),
            cached: false,
        })
    }

    fn get_rust_info(path: &PathBuf) -> Result<RustInfo> {
        // Check for rust-toolchain.toml
        let version_from_file = Self::read_rust_toolchain(path)?;
        
        // Get version from system if not in file
        let version = version_from_file.or_else(|| {
            Command::new("rustc")
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        let version = String::from_utf8_lossy(&o.stdout)
                            .trim()
                            .to_string();
                        // Extract just the version number (e.g., "rustc 1.70.0" -> "1.70.0")
                        version.split_whitespace().nth(1).map(|s| s.to_string())
                    } else {
                        None
                    }
                })
        });

        // Check for Cargo.toml
        let cargo_toml = path.join("Cargo.toml");
        let has_cargo_toml = cargo_toml.exists();
        
        // Check if it's a workspace
        let is_workspace = if has_cargo_toml {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                content.contains("[workspace]")
            } else {
                false
            }
        } else {
            false
        };

        Ok(RustInfo {
            version,
            is_workspace,
            has_cargo_toml,
        })
    }

    fn read_rust_toolchain(path: &PathBuf) -> Result<Option<String>> {
        // Check rust-toolchain.toml
        let toolchain_file = path.join("rust-toolchain.toml");
        if toolchain_file.exists() {
            if let Ok(content) = fs::read_to_string(&toolchain_file) {
                // Simple parsing for toolchain.channel
                for line in content.lines() {
                    if line.trim().starts_with("channel") {
                        if let Some(channel) = line.split('=').nth(1) {
                            let version = channel.trim().trim_matches('"').to_string();
                            if !version.is_empty() {
                                return Ok(Some(version));
                            }
                        }
                    }
                }
            }
        }

        // Check rust-toolchain (legacy format)
        let toolchain_legacy = path.join("rust-toolchain");
        if toolchain_legacy.exists() {
            if let Ok(content) = fs::read_to_string(&toolchain_legacy) {
                let version = content.trim().to_string();
                if !version.is_empty() {
                    return Ok(Some(version));
                }
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct RustInfo {
    version: Option<String>,
    is_workspace: bool,
    has_cargo_toml: bool,
}

