//! Conda environment module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};

/// Conda environment module implementation
pub struct CondaModule;

impl CondaModule {
    /// Fetch Conda environment information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        let env_name = Self::get_conda_env()?;

        let text = env_name.unwrap_or_default();

        Ok(ModuleData {
            module: "conda".to_string(),
            data: serde_json::json!({
                "text": text,
                "active": !text.is_empty(),
            }),
            cached: false,
        })
    }

    fn get_conda_env() -> Result<Option<String>> {
        // Check CONDA_DEFAULT_ENV environment variable
        if let Ok(env_name) = std::env::var("CONDA_DEFAULT_ENV") {
            if !env_name.is_empty() && env_name != "base" {
                return Ok(Some(env_name));
            }
        }
        Ok(None)
    }
}

