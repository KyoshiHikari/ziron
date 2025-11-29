//! Mercurial module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::process::Command;

/// Mercurial module implementation
pub struct MercurialModule;

impl MercurialModule {
    /// Fetch Mercurial status information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let hg_info = Self::get_hg_info(&context.current_dir)?;

        if let Some(info) = hg_info {
            let mut parts = vec![info.branch.clone()];
            
            if !info.status.is_clean() {
                parts.push(info.status.to_string());
            }
            
            if info.modified_count > 0 {
                parts.push(format!("M:{}", info.modified_count));
            }

            Ok(ModuleData {
                module: "mercurial".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "branch": info.branch,
                    "revision": info.revision,
                    "status": info.status.to_string(),
                    "modified": info.modified_count,
                    "bookmark": info.bookmark,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "mercurial".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "branch": null,
                    "revision": null,
                }),
                cached: false,
            })
        }
    }

    fn get_hg_info(path: &PathBuf) -> Result<Option<HgInfo>> {
        // Check if directory is a Mercurial repository
        let hg_dir = path.join(".hg");
        if !hg_dir.exists() {
            return Ok(None);
        }

        // Get branch
        let branch_output = Command::new("hg")
            .args(&["branch"])
            .current_dir(path)
            .output()?;

        if !branch_output.status.success() {
            return Ok(None);
        }

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        // Get revision
        let rev_output = Command::new("hg")
            .args(&["id", "-n"])
            .current_dir(path)
            .output()?;

        let revision = if rev_output.status.success() {
            String::from_utf8_lossy(&rev_output.stdout)
                .trim()
                .to_string()
        } else {
            String::new()
        };

        // Get bookmark if any
        let bookmark_output = Command::new("hg")
            .args(&["bookmark", "--active"])
            .current_dir(path)
            .output();

        let bookmark = bookmark_output
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .filter(|s| !s.is_empty());

        // Get status
        let status_output = Command::new("hg")
            .args(&["status", "--quiet"])
            .current_dir(path)
            .output()?;

        let (status, modified) = Self::parse_status(&status_output.stdout);

        Ok(Some(HgInfo {
            branch,
            revision,
            status,
            modified_count: modified,
            bookmark,
        }))
    }

    fn parse_status(output: &[u8]) -> (HgStatus, usize) {
        let mut modified = 0;

        for line in String::from_utf8_lossy(output).lines() {
            if !line.is_empty() {
                modified += 1;
            }
        }

        let status = if modified == 0 {
            HgStatus::Clean
        } else {
            HgStatus::Dirty
        };

        (status, modified)
    }
}

#[derive(Debug, Clone)]
struct HgInfo {
    branch: String,
    revision: String,
    status: HgStatus,
    modified_count: usize,
    bookmark: Option<String>,
}

#[derive(Debug, Clone)]
enum HgStatus {
    Clean,
    Dirty,
}

impl HgStatus {
    fn is_clean(&self) -> bool {
        matches!(self, HgStatus::Clean)
    }
}

impl ToString for HgStatus {
    fn to_string(&self) -> String {
        match self {
            HgStatus::Clean => "✓".to_string(),
            HgStatus::Dirty => "✗".to_string(),
        }
    }
}

