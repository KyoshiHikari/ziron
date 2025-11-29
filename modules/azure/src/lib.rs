//! Azure module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::process::Command;

/// Azure module implementation
pub struct AzureModule;

impl AzureModule {
    /// Fetch Azure subscription information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        let azure_info = Self::get_azure_info()?;

        if let Some(info) = azure_info {
            let mut parts = vec![];
            
            if let Some(ref subscription) = info.subscription {
                parts.push(subscription.clone());
            }
            
            if let Some(ref account) = info.account {
                parts.push(format!("@{}", account));
            }

            Ok(ModuleData {
                module: "azure".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "subscription": info.subscription,
                    "account": info.account,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "azure".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "subscription": null,
                }),
                cached: false,
            })
        }
    }

    fn get_azure_info() -> Result<Option<AzureInfo>> {
        // Try to get from az CLI
        let subscription_output = Command::new("az")
            .args(&["account", "show", "--query", "name", "-o", "tsv"])
            .output();

        let subscription = subscription_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let sub = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !sub.is_empty() {
                        Some(sub)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        // Get account
        let account_output = Command::new("az")
            .args(&["account", "show", "--query", "user.name", "-o", "tsv"])
            .output();

        let account = account_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let acc = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !acc.is_empty() {
                        Some(acc)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        if subscription.is_some() || account.is_some() {
            Ok(Some(AzureInfo {
                subscription,
                account,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
struct AzureInfo {
    subscription: Option<String>,
    account: Option<String>,
}

