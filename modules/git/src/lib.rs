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

        if let Some(info) = git_info {
            let mut parts = vec![info.branch.clone()];
            
            // Add ahead/behind info
            if let Some(ahead) = info.ahead {
                if ahead > 0 {
                    parts.push(format!("↑{}", ahead));
                }
            }
            if let Some(behind) = info.behind {
                if behind > 0 {
                    parts.push(format!("↓{}", behind));
                }
            }
            
            // Add status indicators
            if !info.status.is_clean() {
                parts.push(info.status.to_string());
            }
            
            // Add commit hash if available
            if let Some(ref hash) = info.commit_hash {
                parts.push(format!("@{}", hash));
            }
            
            // Add tag info if on a tag
            if let Some(ref tag) = info.tag {
                parts.push(format!("tag:{}", tag));
            }
            
            // Add stash count
            if info.stash_count > 0 {
                parts.push(format!("stash:{}", info.stash_count));
            }

            Ok(ModuleData {
                module: "git".to_string(),
                data: serde_json::json!({
                    "text": parts.join(" "),
                    "branch": info.branch,
                    "status": info.status.to_string(),
                    "ahead": info.ahead,
                    "behind": info.behind,
                    "commit_hash": info.commit_hash,
                    "tag": info.tag,
                    "stash_count": info.stash_count,
                    "modified": info.modified_count,
                    "staged": info.staged_count,
                    "untracked": info.untracked_count,
                    "conflicts": info.conflicts_count,
                    "remote": info.remote_name,
                }),
                cached: false,
            })
        } else {
            Ok(ModuleData {
                module: "git".to_string(),
                data: serde_json::json!({
                    "text": "",
                    "branch": null,
                    "status": null,
                }),
                cached: false,
            })
        }
    }

    fn get_git_info(path: &PathBuf) -> Result<Option<GitInfo>> {
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
        
        if branch.is_empty() {
            // Might be in detached HEAD state, check for tag
            let tag_output = Command::new("git")
                .args(&["describe", "--tags", "--exact-match", "HEAD"])
                .current_dir(path)
                .output();
            
            if let Ok(output) = tag_output {
                if output.status.success() {
                    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return Ok(Some(GitInfo {
                        branch: format!("HEAD@{}", tag),
                        status: GitStatus::Clean,
                        ahead: None,
                        behind: None,
                        commit_hash: Self::get_commit_hash(path)?,
                        tag: Some(tag),
                        stash_count: Self::get_stash_count(path)?,
                        modified_count: 0,
                        staged_count: 0,
                        untracked_count: 0,
                        conflicts_count: 0,
                        remote_name: None,
                    }));
                }
            }
            return Ok(None);
        }

        // Get status with detailed information
        let status_output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(path)
            .output()?;

        let (status, modified, staged, untracked, conflicts) = 
            Self::parse_status(&status_output.stdout);

        // Get ahead/behind info
        let (ahead, behind) = Self::get_ahead_behind(path, &branch)?;

        // Get commit hash
        let commit_hash = Self::get_commit_hash(path)?;

        // Get tag if on a tag
        let tag = Self::get_current_tag(path)?;

        // Get stash count
        let stash_count = Self::get_stash_count(path)?;

        // Get remote name
        let remote_name = Self::get_remote_name(path, &branch)?;

        Ok(Some(GitInfo {
            branch,
            status,
            ahead,
            behind,
            commit_hash,
            tag,
            stash_count,
            modified_count: modified,
            staged_count: staged,
            untracked_count: untracked,
            conflicts_count: conflicts,
            remote_name,
        }))
    }

    fn parse_status(output: &[u8]) -> (GitStatus, usize, usize, usize, usize) {
        let mut modified = 0;
        let mut staged = 0;
        let mut untracked = 0;
        let mut conflicts = 0;

        for line in String::from_utf8_lossy(output).lines() {
            if line.len() < 2 {
                continue;
            }
            let status = &line[..2];
            match status {
                "??" => untracked += 1,
                "UU" | "AA" | "DD" | "AU" | "UA" | "DU" | "UD" => conflicts += 1,
                _ => {
                    if status.chars().nth(0).map(|c| c != ' ').unwrap_or(false) {
                        staged += 1;
                    }
                    if status.chars().nth(1).map(|c| c != ' ').unwrap_or(false) {
                        modified += 1;
                    }
                }
            }
        }

        let status = if modified == 0 && staged == 0 && untracked == 0 && conflicts == 0 {
            GitStatus::Clean
        } else {
            GitStatus::Dirty
        };

        (status, modified, staged, untracked, conflicts)
    }

    fn get_ahead_behind(path: &PathBuf, branch: &str) -> Result<(Option<usize>, Option<usize>)> {
        // Get tracking branch
        let tracking_output = Command::new("git")
            .args(&["rev-list", "--left-right", "--count", 
                   &format!("{}...origin/{}", branch, branch)])
            .current_dir(path)
            .output();

        match tracking_output {
            Ok(output) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() == 2 {
                    let behind = parts[0].parse().ok();
                    let ahead = parts[1].parse().ok();
                    return Ok((ahead, behind));
                }
            }
            _ => {}
        }

        Ok((None, None))
    }

    fn get_commit_hash(path: &PathBuf) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .current_dir(path)
            .output()?;

        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    fn get_current_tag(path: &PathBuf) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(&["describe", "--tags", "--exact-match", "HEAD"])
            .current_dir(path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(Some(tag))
            }
            _ => Ok(None),
        }
    }

    fn get_stash_count(path: &PathBuf) -> Result<usize> {
        let output = Command::new("git")
            .args(&["stash", "list"])
            .current_dir(path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                Ok(String::from_utf8_lossy(&output.stdout).lines().count())
            }
            _ => Ok(0),
        }
    }

    fn get_remote_name(path: &PathBuf, branch: &str) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(&["config", &format!("branch.{}.remote", branch)])
            .current_dir(path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let remote = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !remote.is_empty() {
                    Ok(Some(remote))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

#[derive(Debug, Clone)]
struct GitInfo {
    branch: String,
    status: GitStatus,
    ahead: Option<usize>,
    behind: Option<usize>,
    commit_hash: Option<String>,
    tag: Option<String>,
    stash_count: usize,
    modified_count: usize,
    staged_count: usize,
    untracked_count: usize,
    conflicts_count: usize,
    remote_name: Option<String>,
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

