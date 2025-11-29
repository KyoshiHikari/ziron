//! Timer module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};

/// Timer module implementation
pub struct TimerModule;

impl TimerModule {
    /// Fetch timer information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        // Get command execution time from environment (if set)
        let duration_ms = std::env::var("ZIRON_CMD_DURATION_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let text = if duration_ms > 0 {
            if duration_ms < 1000 {
                format!("{}ms", duration_ms)
            } else {
                format!("{:.2}s", duration_ms as f64 / 1000.0)
            }
        } else {
            String::new()
        };

        Ok(ModuleData {
            module: "timer".to_string(),
            data: serde_json::json!({
                "text": text,
                "duration_ms": duration_ms,
            }),
            cached: false,
        })
    }
}

