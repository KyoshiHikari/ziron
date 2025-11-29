//! Git module for Ziron

use ziron_core::error::Result;
use ziron_core::module::{ModuleContext, ModuleData};
use std::path::PathBuf;
use std::process::Command;

/// Git module implementation
pub struct GitModule;

impl GitModule {
    /// Fetch git status information
    pub fn fetch_data(context: &ModuleContext) -> Result<ModuleData> {
        let git_info = Self::get_git_info(&context.current_dir)?;

        let (text, branch, status_str) = if let Some((branch, status)) = &git_info {
            let mut parts = vec![branch.clone()];
            if !status.is_clean() {
                parts.push(status.to_string());
            }
            (parts.join(" "), Some(branch.clone()), Some(status.to_string()))
        } else {
            (String::new(), None, None)
        };

        Ok(ModuleData {
            module: "git".to_string(),
            data: serde_json::json!({
                "text": text,
                "branch": branch,
                "status": status_str,
            }),
            cached: false,
        })
    }

    fn get_git_info(path: &PathBuf) -> Result<Option<(String, GitStatus)>> {
        // Check if directory is a git repository
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Ok(None);
        }

        // Get current branch
        let branch_output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(path)
            .output()?;

        if !branch_output.status.success() {
            return Ok(None);
        }

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        // Get status
        let status_output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(path)
            .output()?;

        let status = if status_output.stdout.is_empty() {
            GitStatus::Clean
        } else {
            GitStatus::Dirty
        };

        Ok(Some((branch, status)))
    }
}

#[derive(Debug, Clone)]
enum GitStatus {
    Clean,
    Dirty,
}

impl GitStatus {
    fn is_clean(&self) -> bool {
        matches!(self, GitStatus::Clean)
    }
}

impl ToString for GitStatus {
    fn to_string(&self) -> String {
        match self {
            GitStatus::Clean => "✓".to_string(),
            GitStatus::Dirty => "✗".to_string(),
        }
    }
}

