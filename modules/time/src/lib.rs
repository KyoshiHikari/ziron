//! Time module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};

/// Time module implementation
pub struct TimeModule;

impl TimeModule {
    /// Fetch time information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        use std::time::SystemTime;
        
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        
        let secs = now.as_secs();
        // Simple time formatting without chrono for now
        let text = format!("{:02}:{:02}:{:02}", 
            (secs / 3600) % 24,
            (secs / 60) % 60,
            secs % 60
        );

        Ok(ModuleData {
            module: "time".to_string(),
            data: serde_json::json!({
                "text": text,
                "timestamp": secs,
            }),
            cached: false,
        })
    }
}

