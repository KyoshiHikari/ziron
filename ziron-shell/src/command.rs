//! Command representation

/// Redirection type
#[derive(Debug, Clone)]
pub enum Redirection {
    Output(String),      // >
    Append(String),      // >>
    Input(String),       // <
    Error(String),       // 2>
    ErrorAppend(String), // 2>>
    Combined(String),    // &>
    FdOutput(u32, String),  // n> (file descriptor output)
    #[allow(dead_code)]
    FdInput(u32, String),   // n< (file descriptor input)
    FdDup(u32, u32),        // n>&m (file descriptor duplication)
}

/// A command to execute
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub stdin: Option<Box<Command>>,
    pub stdout: Option<Redirection>,
    pub stderr: Option<Redirection>,
    pub stdin_file: Option<Redirection>,
}

impl Command {
    /// Check if this is a built-in command
    pub fn is_builtin(&self) -> bool {
        matches!(
            self.name.as_str(),
            "cd" | "exit" | "pwd" | "echo" | "export" | "unset" | "history"
                | "alias" | "unalias" | "type" | "which" | "source" | "jobs" | "fg" | "bg"
                | "kill" | "wait" | "ulimit" | "umask" | "times" | "pushd" | "popd" | "dirs"
                | "read" | "printf" | "test" | "true" | "false" | "function"
        )
    }
}

