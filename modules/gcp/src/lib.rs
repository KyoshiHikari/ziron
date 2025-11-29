//! Google Cloud Platform module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::process::Command;

/// GCP module implementation
pub struct GcpModule;

impl GcpModule {
    /// Fetch GCP configuration information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        let gcp_info = Self::get_gcp_info()?;

        if let Some(info) = gcp_info {
            let mut parts = vec![];
            
            if let Some(ref project) = info.project {
                parts.push(project.clone());
            }
            
            if let Some(ref account) = info.account {
                parts.push(format!("@{}", account));
            }

            Ok(ModuleData {
                module: "gcp".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "project": info.project,
                    "account": info.account,
                    "region": info.region,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "gcp".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "project": null,
                }),
                cached: false,
            })
        }
    }

    fn get_gcp_info() -> Result<Option<GcpInfo>> {
        // Get GCP project from environment
        let project = std::env::var("GCP_PROJECT")
            .or_else(|_| std::env::var("GOOGLE_CLOUD_PROJECT"))
            .ok();

        // Try to get from gcloud CLI
        let gcloud_project = Command::new("gcloud")
            .args(&["config", "get-value", "project"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let proj = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !proj.is_empty() {
                        Some(proj)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        let project = project.or(gcloud_project);

        // Get account
        let account = Command::new("gcloud")
            .args(&["config", "get-value", "account"])
            .output()
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

        // Get region
        let region = std::env::var("GCP_REGION")
            .or_else(|_| std::env::var("GOOGLE_CLOUD_REGION"))
            .ok();

        if project.is_some() || account.is_some() {
            Ok(Some(GcpInfo {
                project,
                account,
                region,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
struct GcpInfo {
    project: Option<String>,
    account: Option<String>,
    region: Option<String>,
}

