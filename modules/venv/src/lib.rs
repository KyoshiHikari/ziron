//! Python virtual environment module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;

/// Python virtual environment module implementation
pub struct VenvModule;

impl VenvModule {
    /// Fetch virtual environment information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let venv_name = Self::detect_venv(&context.current_dir)?;

        let text = venv_name.unwrap_or_default();

        Ok(ModuleData {
            module: "venv".to_string(),
            data: serde_json::json!({
                "text": text,
                "active": !text.is_empty(),
            }),
            cached: false,
        })
    }

    fn detect_venv(path: &PathBuf) -> Result<Option<String>> {
        // Check VIRTUAL_ENV environment variable
        if let Ok(venv_path) = std::env::var("VIRTUAL_ENV") {
            let venv_path = PathBuf::from(venv_path);
            if let Some(name) = venv_path.file_name().and_then(|n| n.to_str()) {
                return Ok(Some(name.to_string()));
            }
        }

        // Check for common virtual environment directories
        let current = path.clone();
        let mut check_path = current.as_path();
        
        loop {
            let venv_dirs = ["venv", ".venv", "env", ".env", "virtualenv"];
            for dir in &venv_dirs {
                let venv_path = check_path.join(dir);
                if venv_path.exists() && venv_path.is_dir() {
                    if let Some(name) = venv_path.file_name().and_then(|n| n.to_str()) {
                        return Ok(Some(name.to_string()));
                    }
                }
            }
            
            if let Some(parent) = check_path.parent() {
                check_path = parent;
            } else {
                break;
            }
        }

        Ok(None)
    }
}

