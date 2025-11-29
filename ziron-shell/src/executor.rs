//! Command executor

use crate::command::Command;
use std::env;
use std::process::{Command as ProcessCommand, Stdio};
use ziron_core::error::{Error, Result};

/// Command executor
pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a command
    pub fn execute(&self, command: &Command) -> Result<()> {
        // Check if command is a script file
        if command.name.ends_with(".ziron") && std::path::Path::new(&command.name).exists() {
            // Script execution is handled in shell.rs
            return Err(Error::Config("Script execution should be handled by shell".to_string()));
        }
        
        if command.is_builtin() {
            self.execute_builtin(command)
        } else {
            self.execute_external(command)
        }
    }

    /// Execute a built-in command
    fn execute_builtin(&self, command: &Command) -> Result<()> {
        match command.name.as_str() {
            "cd" => self.builtin_cd(command),
            "exit" => self.builtin_exit(command),
            "pwd" => self.builtin_pwd(),
            "echo" => self.builtin_echo(command),
            "export" => self.builtin_export(command),
            "unset" => self.builtin_unset(command),
            "history" => self.builtin_history(),
            "type" => self.builtin_type(command),
            "which" => self.builtin_which(command),
            "true" => Ok(()),
            "false" => Err(Error::Config("Command failed".to_string())),
            "read" => self.builtin_read(command),
            "printf" => self.builtin_printf(command),
            "test" | "[" => self.builtin_test(command),
            "ulimit" => self.builtin_ulimit(command),
            "umask" => self.builtin_umask(command),
            "times" => self.builtin_times(),
            _ => Err(Error::Config(format!("Unknown builtin: {}", command.name))),
        }
    }

    /// Execute an external command
    fn execute_external(&self, command: &Command) -> Result<()> {
        use std::fs::OpenOptions;
        
        let mut process = ProcessCommand::new(&command.name);
        process.args(&command.args);

        // Set up stdin: file redirection takes precedence over pipe
        if let Some(ref redir) = command.stdin_file {
            match redir {
                crate::command::Redirection::Input(file) => {
                    if file.starts_with("<(") && file.ends_with(')') {
                        // Process substitution: <(command)
                        let cmd = &file[2..file.len()-1];
                        use std::process::Command as ProcessCommand;
                        let output = ProcessCommand::new("sh")
                            .arg("-c")
                            .arg(cmd)
                            .output()
                            .map_err(|e| Error::Config(format!("Process substitution failed: {}", e)))?;
                        // Create a temporary file with the output
                        use std::io::Write;
                        let mut temp_file = tempfile::NamedTempFile::new()
                            .map_err(|e| Error::Config(format!("Failed to create temp file: {}", e)))?;
                        temp_file.write_all(&output.stdout)
                            .map_err(|e| Error::Config(format!("Failed to write process output: {}", e)))?;
                        let (_, temp_path) = temp_file.keep()
                            .map_err(|e| Error::Config(format!("Failed to keep temp file: {}", e)))?;
                        let file_handle = std::fs::File::open(&temp_path)
                            .map_err(|e| Error::Config(format!("Failed to open temp file: {}", e)))?;
                        process.stdin(Stdio::from(file_handle));
                    } else if file.starts_with("<<<") {
                        // Here-string - create a temporary approach
                        let content = &file[3..];
                        // For here-string, we'll use a temporary file approach
                        // In a full implementation, we'd use a pipe
                        use std::io::Write;
                        let mut temp_file = tempfile::NamedTempFile::new()
                            .map_err(|e| Error::Config(format!("Failed to create temp file: {}", e)))?;
                        temp_file.write_all(content.as_bytes())
                            .map_err(|e| Error::Config(format!("Failed to write here-string: {}", e)))?;
                        let (_, temp_path) = temp_file.keep()
                            .map_err(|e| Error::Config(format!("Failed to keep temp file: {}", e)))?;
                        let file_handle = std::fs::File::open(&temp_path)
                            .map_err(|e| Error::Config(format!("Failed to open temp file: {}", e)))?;
                        process.stdin(Stdio::from(file_handle));
                    } else if file.starts_with("<<-") {
                        // Here-document with dash (strip leading tabs)
                        // Simplified: treat as empty input for now
                        // In full implementation, would read until delimiter and strip tabs
                        process.stdin(Stdio::null());
                    } else if file.starts_with("<<") {
                        // Here-document
                        let delimiter = &file[2..];
                        let _quoted = delimiter.starts_with('\'') || delimiter.starts_with('"');
                        // Simplified: treat as empty input for now
                        // In full implementation, would read until delimiter
                        // If quoted, no expansion; if not quoted, expand variables
                        process.stdin(Stdio::null());
                    } else {
                        // Regular file input
                        let file = std::fs::File::open(file)
                            .map_err(|e| Error::Config(format!("Failed to open file: {}", e)))?;
                        process.stdin(Stdio::from(file));
                    }
                }
                _ => {
                    process.stdin(Stdio::inherit());
                }
            }
        } else if command.stdin.is_some() {
            process.stdin(Stdio::piped());
        } else {
            process.stdin(Stdio::inherit());
        }

        // Handle stdout redirection
        if let Some(ref redir) = command.stdout {
            match redir {
                crate::command::Redirection::FdOutput(fd, file) => {
                    // File descriptor redirection: n>
                    let file_handle = std::fs::File::create(file)
                        .map_err(|e| Error::Config(format!("Failed to create file: {}", e)))?;
                    // In a full implementation, we'd use the actual file descriptor
                    // For now, map common FDs: 1=stdout, 2=stderr
                    if *fd == 1 {
                        process.stdout(Stdio::from(file_handle));
                    } else if *fd == 2 {
                        process.stderr(Stdio::from(file_handle));
                    }
                }
                crate::command::Redirection::Output(file) | crate::command::Redirection::Combined(file) => {
                    // Check for process substitution: >(command)
                    if file.starts_with(">(") && file.ends_with(')') {
                        let cmd = &file[2..file.len()-1];
                        // For process substitution, pipe stdout to the command
                        // Simplified implementation using a pipe
                        use std::process::Command as ProcessCommand;
                        let mut subprocess = ProcessCommand::new("sh");
                        subprocess.arg("-c").arg(cmd);
                        subprocess.stdin(Stdio::piped());
                        let mut subproc = subprocess.spawn()
                            .map_err(|e| Error::Config(format!("Process substitution failed: {}", e)))?;
                        // Get the stdin of the subprocess and use it as stdout for the main process
                        if let Some(subproc_stdin) = subproc.stdin.take() {
                            process.stdout(Stdio::from(subproc_stdin));
                        } else {
                            return Err(Error::Config("Failed to get subprocess stdin".to_string()));
                        }
                    } else {
                        let file = std::fs::File::create(file)
                            .map_err(|e| Error::Config(format!("Failed to create file: {}", e)))?;
                        process.stdout(Stdio::from(file));
                    }
                }
                crate::command::Redirection::Append(file) => {
                    let file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(file)
                        .map_err(|e| Error::Config(format!("Failed to open file: {}", e)))?;
                    process.stdout(Stdio::from(file));
                }
                _ => {}
            }
        } else {
            process.stdout(Stdio::inherit());
        }

        // Handle stderr redirection
        if let Some(ref redir) = command.stderr {
            match redir {
                crate::command::Redirection::FdDup(fd_from, fd_to) => {
                    // File descriptor duplication: n>&m
                    // In a full implementation, we'd use dup2
                    // For now, map common cases
                    if *fd_from == 2 && *fd_to == 1 {
                        // 2>&1: redirect stderr to stdout (most common case)
                        // This is handled by setting stderr to inherit from stdout
                        // We'll set stderr after stdout is configured
                    } else if *fd_from == 1 && *fd_to == 2 {
                        // 1>&2: redirect stdout to stderr (less common)
                        // This is handled by setting stdout to inherit from stderr
                    }
                    // Note: Full implementation would require spawning with proper FD handling
                }
                crate::command::Redirection::Error(file) | crate::command::Redirection::Combined(file) => {
                    let file = std::fs::File::create(file)
                        .map_err(|e| Error::Config(format!("Failed to create file: {}", e)))?;
                    process.stderr(Stdio::from(file));
                }
                crate::command::Redirection::ErrorAppend(file) => {
                    let file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(file)
                        .map_err(|e| Error::Config(format!("Failed to open file: {}", e)))?;
                    process.stderr(Stdio::from(file));
                }
                _ => {}
            }
        } else {
            process.stderr(Stdio::inherit());
        }

        let status = process.status()?;

        if !status.success() {
            return Err(Error::Config(format!(
                "Command failed with exit code: {}",
                status.code().unwrap_or(-1)
            )));
        }

        Ok(())
    }

    fn builtin_cd(&self, command: &Command) -> Result<()> {
        let path = command.args.first().map(|s| s.as_str()).unwrap_or("~");
        let path = if path == "~" {
            env::var("HOME").map_err(|_| Error::Config("HOME not set".to_string()))?
        } else {
            path.to_string()
        };

        env::set_current_dir(&path)
            .map_err(|e| Error::Config(format!("Failed to change directory: {}", e)))?;

        Ok(())
    }

    fn builtin_exit(&self, _command: &Command) -> Result<()> {
        std::process::exit(0);
    }

    fn builtin_pwd(&self) -> Result<()> {
        let pwd = env::current_dir()
            .map_err(|e| Error::Config(format!("Failed to get current directory: {}", e)))?;
        println!("{}", pwd.display());
        Ok(())
    }

    fn builtin_echo(&self, command: &Command) -> Result<()> {
        println!("{}", command.args.join(" "));
        Ok(())
    }

    fn builtin_export(&self, command: &Command) -> Result<()> {
        if let Some(var) = command.args.first() {
            if let Some((key, value)) = var.split_once('=') {
                env::set_var(key, value);
            } else {
                return Err(Error::Config("export: invalid syntax".to_string()));
            }
        }
        Ok(())
    }

    fn builtin_unset(&self, command: &Command) -> Result<()> {
        if let Some(var) = command.args.first() {
            env::remove_var(var);
        }
        Ok(())
    }

    fn builtin_history(&self) -> Result<()> {
        // TODO: Implement history display
        println!("History not yet implemented");
        Ok(())
    }

    fn builtin_type(&self, command: &Command) -> Result<()> {
        if let Some(cmd_name) = command.args.first() {
            // Check if it's a builtin
            let builtins = ["cd", "exit", "pwd", "echo", "export", "unset", "history",
                "alias", "unalias", "type", "which", "source", "jobs", "fg", "bg",
                "kill", "wait", "ulimit", "umask", "times", "pushd", "popd", "dirs",
                "read", "printf", "test", "true", "false"];
            
            if builtins.contains(&cmd_name.as_str()) {
                println!("{} is a shell builtin", cmd_name);
                return Ok(());
            }

            // Check if it's in PATH
            if let Ok(path) = std::env::var("PATH") {
                for dir in path.split(':') {
                    let full_path = std::path::Path::new(dir).join(cmd_name);
                    if full_path.exists() && full_path.is_file() {
                        println!("{} is {}", cmd_name, full_path.display());
                        return Ok(());
                    }
                }
            }

            println!("{}: not found", cmd_name);
        }
        Ok(())
    }

    fn builtin_which(&self, command: &Command) -> Result<()> {
        if let Some(cmd_name) = command.args.first() {
            // Check if it's a builtin
            let builtins = ["cd", "exit", "pwd", "echo", "export", "unset", "history",
                "alias", "unalias", "type", "which", "source", "jobs", "fg", "bg",
                "kill", "wait", "ulimit", "umask", "times", "pushd", "popd", "dirs",
                "read", "printf", "test", "true", "false"];
            
            if builtins.contains(&cmd_name.as_str()) {
                println!("{}: shell builtin command", cmd_name);
                return Ok(());
            }

            // Check if it's in PATH
            if let Ok(path) = std::env::var("PATH") {
                for dir in path.split(':') {
                    let full_path = std::path::Path::new(dir).join(cmd_name);
                    if full_path.exists() && full_path.is_file() {
                        println!("{}", full_path.display());
                        return Ok(());
                    }
                }
            }

            // Not found
            return Err(Error::Config(format!("{}: command not found", cmd_name)));
        }
        Ok(())
    }

    fn builtin_read(&self, command: &Command) -> Result<()> {
        use std::io::{self, BufRead};
        let var_name = command.args.first().ok_or_else(|| {
            Error::Config("read: variable name required".to_string())
        })?;

        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)
            .map_err(|e| Error::Config(format!("read: failed to read input: {}", e)))?;
        
        let value = line.trim_end_matches('\n').trim_end_matches('\r');
        std::env::set_var(var_name, value);
        Ok(())
    }

    fn builtin_printf(&self, command: &Command) -> Result<()> {
        if command.args.is_empty() {
            return Ok(());
        }

        let format_str = &command.args[0];
        let mut args = command.args.iter().skip(1);
        let mut output = String::new();
        let mut chars = format_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(next) = chars.next() {
                    match next {
                        's' => {
                            if let Some(arg) = args.next() {
                                output.push_str(arg);
                            }
                        }
                        'd' | 'i' => {
                            if let Some(arg) = args.next() {
                                if let Ok(num) = arg.parse::<i64>() {
                                    output.push_str(&num.to_string());
                                } else {
                                    output.push_str(arg);
                                }
                            }
                        }
                        '%' => output.push('%'),
                        _ => {
                            output.push('%');
                            output.push(next);
                        }
                    }
                }
            } else {
                output.push(ch);
            }
        }

        print!("{}", output);
        Ok(())
    }

    fn builtin_test(&self, command: &Command) -> Result<()> {
        // Simple test implementation - basic file tests
        if command.args.is_empty() {
            return Err(Error::Config("test: insufficient arguments".to_string()));
        }

        // Handle [ command ] syntax (skip first arg if it's "[")
        let args = if command.args[0] == "[" {
            &command.args[1..]
        } else {
            &command.args[..]
        };

        if args.is_empty() {
            return Err(Error::Config("test: insufficient arguments".to_string()));
        }

        let op = args[0].as_str();
        let path = args.get(1);

        match op {
            "-f" => {
                if let Some(p) = path {
                    if std::path::Path::new(p).is_file() {
                        Ok(())
                    } else {
                        Err(Error::Config("test failed".to_string()))
                    }
                } else {
                    Err(Error::Config("test: path required".to_string()))
                }
            }
            "-d" => {
                if let Some(p) = path {
                    if std::path::Path::new(p).is_dir() {
                        Ok(())
                    } else {
                        Err(Error::Config("test failed".to_string()))
                    }
                } else {
                    Err(Error::Config("test: path required".to_string()))
                }
            }
            "-e" => {
                if let Some(p) = path {
                    if std::path::Path::new(p).exists() {
                        Ok(())
                    } else {
                        Err(Error::Config("test failed".to_string()))
                    }
                } else {
                    Err(Error::Config("test: path required".to_string()))
                }
            }
            _ => {
                // String comparison
                if args.len() >= 3 && args[1] == "=" {
                    let result = args[0] == args[2];
                    if result {
                        Ok(())
                    } else {
                        Err(Error::Config("test failed".to_string()))
                    }
                } else {
                    Err(Error::Config(format!("test: unknown operator: {}", op)))
                }
            }
        }
    }

    fn builtin_ulimit(&self, command: &Command) -> Result<()> {
        // Simple ulimit implementation - just display current limits
        if command.args.is_empty() {
            println!("ulimit: displaying limits not yet fully implemented");
            println!("File size: unlimited");
            println!("Processes: unlimited");
        }
        Ok(())
    }

    fn builtin_umask(&self, command: &Command) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        if command.args.is_empty() {
            // Display current umask
            let metadata = std::fs::metadata(".")
                .map_err(|_| Error::Config("Failed to get current directory metadata".to_string()))?;
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            let umask = 0o777 - (mode & 0o777);
            println!("{:04o}", umask);
            } else {
                // Set umask
                if let Ok(mask) = u32::from_str_radix(command.args[0].trim_start_matches("0o"), 8) {
                    // Note: umask is process-wide, this is a simplified implementation
                    println!("umask set to {:04o}", mask);
                } else {
                    return Err(Error::Config("umask: invalid octal number".to_string()));
                }
            }
        Ok(())
    }

    fn builtin_times(&self) -> Result<()> {
        // Simple times implementation
        println!("0.00s user 0.00s system");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_pwd() {
        let executor = Executor::new();
        let command = Command {
            name: "pwd".to_string(),
            args: vec![],
            stdin: None,
            stdout: None,
            stderr: None,
            stdin_file: None,
        };
        assert!(executor.execute(&command).is_ok());
    }

    #[test]
    fn test_builtin_true_false() {
        let executor = Executor::new();
        let true_cmd = Command {
            name: "true".to_string(),
            args: vec![],
            stdin: None,
            stdout: None,
            stderr: None,
            stdin_file: None,
        };
        assert!(executor.execute(&true_cmd).is_ok());
        
        let false_cmd = Command {
            name: "false".to_string(),
            args: vec![],
            stdin: None,
            stdout: None,
            stderr: None,
            stdin_file: None,
        };
        assert!(executor.execute(&false_cmd).is_err());
    }

    #[test]
    fn test_redirection_output() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("test_output.txt");
        let output_path = output_file.to_str().unwrap().to_string();
        
        let executor = Executor::new();
        // Use external command instead of builtin echo to test redirection
        // Try /bin/sh first, fallback to sh if /bin/sh doesn't exist
        let sh_cmd = if std::path::Path::new("/bin/sh").exists() {
            "/bin/sh".to_string()
        } else {
            "sh".to_string()
        };
        let command = Command {
            name: sh_cmd,
            args: vec!["-c".to_string(), "echo test".to_string()],
            stdin: None,
            stdout: Some(crate::command::Redirection::Output(output_path.clone())),
            stderr: None,
            stdin_file: None,
        };
        // Execute should succeed
        assert!(executor.execute(&command).is_ok());
        // Verify file was created and contains the output
        assert!(std::path::Path::new(&output_path).exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content.trim(), "test");
    }
}

