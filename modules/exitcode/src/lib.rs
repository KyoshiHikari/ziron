//! Exit code module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};

/// Exit code module implementation
pub struct ExitCodeModule;

impl ExitCodeModule {
    /// Fetch exit code information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        // Get last exit code from environment (if set)
        let exit_code = std::env::var("ZIRON_LAST_EXIT_CODE")
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        let text = if exit_code != 0 {
            format!("{}", exit_code)
        } else {
            String::new()
        };

        Ok(ModuleData {
            module: "exitcode".to_string(),
            data: serde_json::json!({
                "text": text,
                "code": exit_code,
            }),
            cached: false,
        })
    }
}

