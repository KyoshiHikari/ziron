//! Python virtual environment module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::fs;

/// Python virtual environment module implementation
pub struct VenvModule;

impl VenvModule {
    /// Fetch virtual environment information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let venv_info = Self::detect_venv(&context.current_dir)?;

        let mut parts = vec![];
        
        if let Some(ref venv) = venv_info.venv_name {
            parts.push(venv.clone());
        }
        
        if let Some(ref version) = venv_info.python_version {
            parts.push(version.clone());
        }
        
        if let Some(ref tool) = venv_info.tool {
            parts.push(tool.clone());
        }

        Ok(ModuleData {
            module: "venv".to_string(),
            data: serde_json::json!({
                "text": parts.join(" "),
                "venv": venv_info.venv_name,
                "python_version": venv_info.python_version,
                "tool": venv_info.tool,
                "active": venv_info.venv_name.is_some(),
            }),
            cached: false,
        })
    }

    fn detect_venv(path: &PathBuf) -> Result<VenvInfo> {
        let mut venv_name = None;
        let mut tool = None;

        // Check VIRTUAL_ENV environment variable
        if let Ok(venv_path) = std::env::var("VIRTUAL_ENV") {
            let venv_path = PathBuf::from(venv_path);
            if let Some(name) = venv_path.file_name().and_then(|n| n.to_str()) {
                venv_name = Some(name.to_string());
            }
        }

        // Check for .python-version file
        let python_version = Self::read_python_version(path)?;

        // Check for pipenv
        if path.join("Pipfile").exists() {
            tool = Some("pipenv".to_string());
        }
        
        // Check for poetry
        if path.join("pyproject.toml").exists() {
            if let Ok(content) = fs::read_to_string(path.join("pyproject.toml")) {
                if content.contains("[tool.poetry]") {
                    tool = Some("poetry".to_string());
                }
            }
        }

        // Check for common virtual environment directories if not found yet
        if venv_name.is_none() {
            let current = path.clone();
            let mut check_path = current.as_path();
            
            loop {
                let venv_dirs = ["venv", ".venv", "env", ".env", "virtualenv"];
                for dir in &venv_dirs {
                    let venv_path = check_path.join(dir);
                    if venv_path.exists() && venv_path.is_dir() {
                        if let Some(name) = venv_path.file_name().and_then(|n| n.to_str()) {
                            venv_name = Some(name.to_string());
                            break;
                        }
                    }
                }
                
                if venv_name.is_some() {
                    break;
                }
                
                if let Some(parent) = check_path.parent() {
                    check_path = parent;
                } else {
                    break;
                }
            }
        }

        Ok(VenvInfo {
            venv_name,
            python_version,
            tool,
        })
    }

    fn read_python_version(path: &PathBuf) -> Result<Option<String>> {
        // Check .python-version
        let python_version_file = path.join(".python-version");
        if python_version_file.exists() {
            if let Ok(content) = fs::read_to_string(&python_version_file) {
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
struct VenvInfo {
    venv_name: Option<String>,
    python_version: Option<String>,
    tool: Option<String>,
}

