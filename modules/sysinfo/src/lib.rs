//! System info module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};

/// System info module implementation
pub struct SysInfoModule;

impl SysInfoModule {
    /// Fetch system information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let text = format!("{}@{}", context.user, context.hostname);

        Ok(ModuleData {
            module: "sysinfo".to_string(),
            data: serde_json::json!({
                "text": text,
                "user": context.user.clone(),
                "hostname": context.hostname.clone(),
            }),
            cached: false,
        })
    }
}

