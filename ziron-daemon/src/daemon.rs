//! Daemon lifecycle management

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use ziron_core::error::{Error, Result};

/// Daemon PID file path
pub fn get_pid_file() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| Error::Config("HOME not set".to_string()))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("ziron")
        .join("ziron-daemon.pid"))
}

/// Check if daemon is running
pub fn is_running() -> bool {
    if let Ok(pid_file) = get_pid_file() {
        if pid_file.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    // Check if process is still running using /proc (Linux) or kill -0
                    #[cfg(target_os = "linux")]
                    {
                        let proc_path = format!("/proc/{}", pid);
                        if std::path::Path::new(&proc_path).exists() {
                            return true;
                        } else {
                            // PID file exists but process is dead, clean it up
                            let _ = fs::remove_file(&pid_file);
                        }
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        // For non-Linux, just check if PID file exists
                        // In production, could use libc::kill(pid, 0)
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Write PID file
pub fn write_pid_file() -> Result<()> {
    let pid_file = get_pid_file()?;
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = pid_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::Config(format!("Failed to create config directory: {}", e)))?;
    }
    
    let pid = std::process::id();
    let mut file = fs::File::create(&pid_file)
        .map_err(|e| Error::Config(format!("Failed to create PID file: {}", e)))?;
    
    file.write_all(pid.to_string().as_bytes())
        .map_err(|e| Error::Config(format!("Failed to write PID file: {}", e)))?;
    
    Ok(())
}

/// Remove PID file
pub fn remove_pid_file() -> Result<()> {
    let pid_file = get_pid_file()?;
    if pid_file.exists() {
        fs::remove_file(&pid_file)
            .map_err(|e| Error::Config(format!("Failed to remove PID file: {}", e)))?;
    }
    Ok(())
}

/// Get daemon PID if running
pub fn get_daemon_pid() -> Option<u32> {
    if let Ok(pid_file) = get_pid_file() {
        if pid_file.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    return Some(pid);
                }
            }
        }
    }
    None
}
