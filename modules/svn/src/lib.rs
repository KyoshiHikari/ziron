//! SVN (Subversion) module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::process::Command;

/// SVN module implementation
pub struct SvnModule;

impl SvnModule {
    /// Fetch SVN status information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let svn_info = Self::get_svn_info(&context.current_dir)?;

        if let Some(info) = svn_info {
            let mut parts = vec![info.branch_or_path.clone()];
            
            if !info.status.is_clean() {
                parts.push(info.status.to_string());
            }
            
            if info.modified_count > 0 {
                parts.push(format!("M:{}", info.modified_count));
            }
            
            if info.conflicts_count > 0 {
                parts.push(format!("C:{}", info.conflicts_count));
            }

            Ok(ModuleData {
                module: "svn".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "branch": info.branch_or_path,
                    "revision": info.revision,
                    "status": info.status.to_string(),
                    "modified": info.modified_count,
                    "conflicts": info.conflicts_count,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "svn".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "branch": null,
                    "revision": null,
                }),
                cached: false,
            })
        }
    }

    fn get_svn_info(path: &PathBuf) -> Result<Option<SvnInfo>> {
        // Check if directory is an SVN repository
        let svn_dir = path.join(".svn");
        if !svn_dir.exists() {
            return Ok(None);
        }

        // Get SVN info
        let info_output = Command::new("svn")
            .args(&["info", "--show-item", "revision", "url", "relative-url"])
            .current_dir(path)
            .output()?;

        if !info_output.status.success() {
            return Ok(None);
        }

        let info_text = String::from_utf8_lossy(&info_output.stdout);
        let lines: Vec<&str> = info_text.lines().collect();
        
        let revision = lines.get(0).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let url = lines.get(1).unwrap_or(&"").to_string();
        
        // Extract branch/tag/trunk from URL
        let branch_or_path = Self::extract_branch_from_url(&url);

        // Get status
        let status_output = Command::new("svn")
            .args(&["status", "--quiet"])
            .current_dir(path)
            .output()?;

        let (status, modified, conflicts) = Self::parse_status(&status_output.stdout);

        Ok(Some(SvnInfo {
            branch_or_path,
            revision,
            status,
            modified_count: modified,
            conflicts_count: conflicts,
        }))
    }

    fn extract_branch_from_url(url: &str) -> String {
        // Try to extract branch/tag/trunk from URL
        if url.contains("/trunk") {
            "trunk".to_string()
        } else if url.contains("/branches/") {
            url.split("/branches/")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .unwrap_or("branch")
                .to_string()
        } else if url.contains("/tags/") {
            url.split("/tags/")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .unwrap_or("tag")
                .to_string()
        } else {
            // Just use the last part of the URL
            url.split('/').last().unwrap_or("svn").to_string()
        }
    }

    fn parse_status(output: &[u8]) -> (SvnStatus, usize, usize) {
        let mut modified = 0;
        let mut conflicts = 0;

        for line in String::from_utf8_lossy(output).lines() {
            if line.is_empty() {
                continue;
            }
            let status_char = line.chars().next().unwrap_or(' ');
            match status_char {
                'M' | 'A' | 'D' | 'R' => modified += 1,
                'C' => conflicts += 1,
                _ => {}
            }
        }

        let status = if modified == 0 && conflicts == 0 {
            SvnStatus::Clean
        } else {
            SvnStatus::Dirty
        };

        (status, modified, conflicts)
    }
}

#[derive(Debug, Clone)]
struct SvnInfo {
    branch_or_path: String,
    revision: u64,
    status: SvnStatus,
    modified_count: usize,
    conflicts_count: usize,
}

#[derive(Debug, Clone)]
enum SvnStatus {
    Clean,
    Dirty,
}

impl SvnStatus {
    fn is_clean(&self) -> bool {
        matches!(self, SvnStatus::Clean)
    }
}

impl ToString for SvnStatus {
    fn to_string(&self) -> String {
        match self {
            SvnStatus::Clean => "✓".to_string(),
            SvnStatus::Dirty => "✗".to_string(),
        }
    }
}

