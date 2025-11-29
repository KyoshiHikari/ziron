//! AWS module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::fs;

/// AWS module implementation
pub struct AwsModule;

impl AwsModule {
    /// Fetch AWS profile information
    pub fn fetch_data(_context: &ModuleContext) -> Result<ModuleData> {
        let aws_info = Self::get_aws_info()?;

        if let Some(info) = aws_info {
            let mut parts = vec![];
            
            if let Some(ref profile) = info.profile {
                parts.push(profile.clone());
            }
            
            if let Some(ref region) = info.region {
                parts.push(region.clone());
            }

            Ok(ModuleData {
                module: "aws".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "profile": info.profile,
                    "region": info.region,
                    "account_id": info.account_id,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "aws".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "profile": null,
                }),
                cached: false,
            })
        }
    }

    fn get_aws_info() -> Result<Option<AwsInfo>> {
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };
        let aws_dir = PathBuf::from(&home).join(".aws");
        
        if !aws_dir.exists() {
            return Ok(None);
        }

        // Get profile from environment or default
        let profile = std::env::var("AWS_PROFILE")
            .or_else(|_| std::env::var("AWS_DEFAULT_PROFILE"))
            .ok()
            .or_else(|| Some("default".to_string()));

        // Read config file
        let config_file = aws_dir.join("config");
        let region = if config_file.exists() {
            Self::read_region_from_config(&config_file, profile.as_deref())?
        } else {
            None
        };

        // Try to get account ID from credentials or environment
        let account_id = std::env::var("AWS_ACCOUNT_ID").ok();

        if profile.is_some() || region.is_some() {
            Ok(Some(AwsInfo {
                profile,
                region,
                account_id,
            }))
        } else {
            Ok(None)
        }
    }

    fn read_region_from_config(config_path: &PathBuf, profile: Option<&str>) -> Result<Option<String>> {
        let content = fs::read_to_string(config_path)?;
        let profile_name = profile.unwrap_or("default");
        let profile_section = if profile_name == "default" {
            "[default]"
        } else {
            &format!("[profile {}]", profile_name)
        };

        let mut in_section = false;
        for line in content.lines() {
            if line.trim() == profile_section {
                in_section = true;
                continue;
            }
            if line.starts_with('[') {
                in_section = false;
                continue;
            }
            if in_section && line.trim().starts_with("region") {
                if let Some(region) = line.split('=').nth(1) {
                    return Ok(Some(region.trim().to_string()));
                }
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct AwsInfo {
    profile: Option<String>,
    region: Option<String>,
    account_id: Option<String>,
}

