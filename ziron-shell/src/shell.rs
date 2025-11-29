//! Main shell implementation

use crate::completion::ZironCompleter;
use crate::executor::Executor;
use crate::jobs::JobManager;
use crate::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::config::{CompletionType, Configurer};
use rustyline::Editor;
use ziron_core::config::Config;
use ziron_core::error::{Error, Result};
use ziron_core::module::ModuleContext;
use ziron_core::prompt::PromptRenderer;

/// Main Ziron shell
pub struct ZironShell {
    #[allow(dead_code)]
    config: Config,
    renderer: PromptRenderer,
    editor: Editor<ZironCompleter, DefaultHistory>,
    executor: Executor,
    completer: ZironCompleter,
    aliases: std::collections::HashMap<String, String>,
    functions: std::collections::HashMap<String, String>, // Function name -> body
    directory_stack: Vec<std::path::PathBuf>,
    job_manager: JobManager,
    script_args: Vec<String>, // Script arguments ($1, $2, etc.)
    last_exit_code: i32, // Last command exit code ($?)
}

impl ZironShell {
    /// Create a new shell instance
    pub fn new(config: Config, renderer: PromptRenderer) -> Result<Self> {
        let mut completer = ZironCompleter::new();
        completer.set_partial_completion(config.completion.partial_completion);
        let mut editor = Editor::new()
            .map_err(|e| Error::Config(format!("Failed to initialize line editor: {}", e)))?;
        
        // Configure multi-column completion display with menu navigation
        // Circular type enables menu navigation with arrow keys
        editor.set_completion_type(CompletionType::Circular);
        let _ = editor.set_max_history_size(10000); // Ignore errors for history size
        editor.set_completion_prompt_limit(100); // Limit completion items shown
        
        editor.set_helper(Some(completer.clone()));

        let executor = Executor::new();

        Ok(Self {
            config,
            renderer,
            editor,
            executor,
            completer,
            aliases: std::collections::HashMap::new(),
            functions: std::collections::HashMap::new(),
            directory_stack: Vec::new(),
            job_manager: JobManager::new(),
            script_args: Vec::new(),
            last_exit_code: 0,
        })
    }


    /// Run the shell main loop
    pub fn run(&mut self) -> Result<()> {
        // Load history (try to load, ignore errors)
        let history_path = std::path::PathBuf::from(
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        ).join(".ziron_history");
        let _ = self.editor.load_history(&history_path);

        loop {
            // Render prompt
            let prompt_str = match self.render_prompt() {
                Ok(prompt) => prompt,
                Err(e) => {
                    eprintln!("Error rendering prompt: {}", e);
                    "ziron> ".to_string()
                }
            };

            // Use readline with the prompt - rustyline will handle display
            match self.editor.readline(&prompt_str) {
                Ok(line) => {
                    // Add to history
                    let _ = self.editor.add_history_entry(line.as_str());

                    // Parse and execute
                    if let Err(e) = self.execute_line(&line) {
                        eprintln!("Error: {}", e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D
                    println!("\nExit");
                    break;
                }
                Err(e) => {
                    eprintln!("Error reading line: {}", e);
                    break;
                }
            }
        }

        // Save history
        let history_path = std::path::PathBuf::from(
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        ).join(".ziron_history");
        let _ = self.editor.save_history(&history_path);

        Ok(())
    }

    /// Render the prompt
    fn render_prompt(&self) -> Result<String> {
        use ziron_core::module::ModuleData;
        
        let context = ModuleContext::from_env()?;
        let mut module_data = Vec::new();

        // Get modules from theme segments (what should be displayed)
        let modules_to_fetch: Vec<String> = if !self.renderer.theme().segments.is_empty() {
            self.renderer.theme().segments.iter()
                .map(|s| s.module.clone())
                .collect()
        } else {
            self.config.modules.clone()
        };

        // Fetch data from all modules
        for module_name in &modules_to_fetch {
            if let Some(data) = Self::fetch_module_data(module_name, &context)? {
                module_data.push(data);
            }
        }

        // Render using theme
        let prompt = self.renderer.render(&context, &module_data)?;
        Ok(prompt)
    }

    /// Fetch module data (same logic as daemon)
    fn fetch_module_data(module_name: &str, context: &ModuleContext) -> Result<Option<ziron_core::module::ModuleData>> {
        use ziron_core::module::ModuleData;
        
        let result = match module_name {
            "symbol" => {
                Ok(ModuleData {
                    module: "symbol".to_string(),
                    data: serde_json::json!({"text": " ⚡"}),
                    cached: false,
                })
            }
            "sysinfo" => {
                Ok(ModuleData {
                    module: "sysinfo".to_string(),
                    data: serde_json::json!({
                        "text": format!("{}@{}", context.user, context.hostname)
                    }),
                    cached: false,
                })
            }
            "cwd" => {
                let cwd = context.current_dir.display().to_string();
                let home = std::env::var("HOME").unwrap_or_else(|_| format!("/home/{}", context.user));
                let cwd_short = if cwd.starts_with(&home) {
                    cwd.replace(&home, "~")
                } else {
                    cwd
                };
                Ok(ModuleData {
                    module: "cwd".to_string(),
                    data: serde_json::json!({"text": cwd_short}),
                    cached: false,
                })
            }
            "git" => ziron_module_git::GitModule::fetch_data(context),
            "exitcode" => exitcode::ExitCodeModule::fetch_data(context),
            "timer" => timer::TimerModule::fetch_data(context),
            "time" => time::TimeModule::fetch_data(context),
            "venv" => venv::VenvModule::fetch_data(context),
            "node" => node::NodeModule::fetch_data(context),
            "rust" => rust::RustModule::fetch_data(context),
            "conda" => conda::CondaModule::fetch_data(context),
            "svn" => ziron_module_svn::SvnModule::fetch_data(context),
            "mercurial" => ziron_module_mercurial::MercurialModule::fetch_data(context),
            "docker" => ziron_module_docker::DockerModule::fetch_data(context),
            "kubernetes" => ziron_module_kubernetes::KubernetesModule::fetch_data(context),
            "aws" => ziron_module_aws::AwsModule::fetch_data(context),
            "gcp" => ziron_module_gcp::GcpModule::fetch_data(context),
            "azure" => ziron_module_azure::AzureModule::fetch_data(context),
            "terraform" => ziron_module_terraform::TerraformModule::fetch_data(context),
            "go" => ziron_module_go::GoModule::fetch_data(context),
            _ => {
                // Unknown module, return None
                return Ok(None);
            }
        };
        
        match result {
            Ok(data) => Ok(Some(data)),
            Err(e) => {
                tracing::warn!("Error fetching data for module {}: {}", module_name, e);
                Ok(None)
            }
        }
    }

    /// Execute a command line
    fn execute_line(&mut self, line: &str) -> Result<()> {
        let line = line.trim();
        if line.is_empty() {
            self.last_exit_code = 0;
            return Ok(());
        }

        // Check for alias expansion
        let expanded_line = self.expand_aliases(line);

        // Check if it's a script file execution
        if expanded_line.ends_with(".ziron") || (expanded_line.contains(' ') && expanded_line.split_whitespace().next().map(|s| s.ends_with(".ziron")).unwrap_or(false)) {
            let script_path = expanded_line.split_whitespace().next().unwrap_or(&expanded_line);
            if std::path::Path::new(script_path).exists() {
                return self.execute_script(script_path);
            }
        }

        // Create expansion context with script arguments and last exit code
        let expansion_ctx = crate::parser::ExpansionContext {
            script_args: self.script_args.clone(),
            last_exit_code: Some(self.last_exit_code),
        };
        
        // Parse command
        let commands = Parser::parse_with_context(&expanded_line, &expansion_ctx)?;

        // Execute commands
        for command in commands {
            // Check if command is a builtin that needs shell state
            match command.name.as_str() {
                "alias" | "unalias" | "pushd" | "popd" | "dirs" | "source" | "jobs" | "fg" | "bg" | "kill" | "wait" => {
                    self.execute_builtin_with_state(&command)?;
                }
                "cd" => {
                    // Track directory changes for pushd/popd
                    let current_dir = std::env::current_dir().ok();
                    self.executor.execute(&command)?;
                    if let Some(dir) = current_dir {
                        // Don't add if it's the same directory
                        if let Ok(new_dir) = std::env::current_dir() {
                            if dir != new_dir {
                                self.directory_stack.push(dir);
                            }
                        }
                    }
                }
                _ => {
                    // Check if command should run in background
                    let is_background = command.args.last()
                        .map(|arg| arg == "&")
                        .unwrap_or(false);
                    
                    if is_background {
                        // Remove & from args
                        let mut bg_command = command.clone();
                        bg_command.args.pop();
                        self.execute_background(&bg_command)?;
                    } else {
                        self.executor.execute(&command)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Expand aliases in command line
    fn expand_aliases(&self, line: &str) -> String {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            return line.to_string();
        }

        if let Some(alias_value) = self.aliases.get(words[0]) {
            let mut result = alias_value.clone();
            if words.len() > 1 {
                result.push(' ');
                result.push_str(&words[1..].join(" "));
            }
            result
        } else {
            line.to_string()
        }
    }

    /// Execute built-in commands that need shell state
    fn execute_builtin_with_state(&mut self, command: &crate::command::Command) -> Result<()> {
        match command.name.as_str() {
            "alias" => {
                if command.args.is_empty() {
                    // List all aliases
                    for (name, value) in &self.aliases {
                        println!("alias {}='{}'", name, value);
                    }
                } else {
                    // Set alias
                    for arg in &command.args {
                        if let Some((name, value)) = arg.split_once('=') {
                            self.aliases.insert(name.to_string(), value.to_string());
                            self.completer.add_alias(name.to_string());
                            // Update editor's helper
                            if let Some(helper) = self.editor.helper_mut() {
                                helper.add_alias(name.to_string());
                            }
                        }
                    }
                }
                Ok(())
            }
            "unalias" => {
                for arg in &command.args {
                    self.aliases.remove(arg);
                    self.completer.remove_alias(arg);
                    // Update editor's helper
                    if let Some(helper) = self.editor.helper_mut() {
                        helper.remove_alias(arg);
                    }
                }
                Ok(())
            }
            "function" => {
                // Function definition: function name() { body }
                // Simplified: function name body
                if command.args.len() >= 2 {
                    let func_name = command.args[0].clone();
                    let func_body = command.args[1..].join(" ");
                    self.functions.insert(func_name.clone(), func_body);
                    self.completer.add_function(func_name.clone());
                    // Update editor's helper
                    if let Some(helper) = self.editor.helper_mut() {
                        helper.add_function(func_name);
                    }
                }
                Ok(())
            }
            "pushd" => {
                let target_dir = if let Some(dir) = command.args.first() {
                    std::path::PathBuf::from(dir)
                } else {
                    // Swap current directory with top of stack
                    if let Some(top) = self.directory_stack.pop() {
                        let current = std::env::current_dir()
                            .map_err(|e| Error::Config(format!("Failed to get current directory: {}", e)))?;
                        self.directory_stack.push(current.clone());
                        top
                    } else {
                        return Err(Error::Config("pushd: directory stack empty".to_string()));
                    }
                };

                let current_dir = std::env::current_dir()
                    .map_err(|e| Error::Config(format!("Failed to get current directory: {}", e)))?;
                self.directory_stack.push(current_dir);
                std::env::set_current_dir(&target_dir)
                    .map_err(|e| Error::Config(format!("Failed to change directory: {}", e)))?;
                self.builtin_dirs();
                Ok(())
            }
            "popd" => {
                if let Some(dir) = self.directory_stack.pop() {
                    std::env::set_current_dir(&dir)
                        .map_err(|e| Error::Config(format!("Failed to change directory: {}", e)))?;
                    self.builtin_dirs();
                } else {
                    return Err(Error::Config("popd: directory stack empty".to_string()));
                }
                Ok(())
            }
            "dirs" => {
                self.builtin_dirs();
                Ok(())
            }
            "source" => {
                if let Some(script_path) = command.args.first() {
                    let content = std::fs::read_to_string(script_path)
                        .map_err(|e| Error::Config(format!("Failed to read script: {}", e)))?;
                    // Execute each line
                    for line in content.lines() {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with('#') {
                            if let Err(e) = self.execute_line(line) {
                                eprintln!("Error executing line: {}", e);
                            }
                        }
                    }
                }
                Ok(())
            }
            "jobs" => {
                let jobs = self.job_manager.list_jobs();
                for job in jobs {
                    use crate::jobs::JobStatus;
                    let status_str: String = match job.status {
                        JobStatus::Running => "Running".to_string(),
                        JobStatus::Stopped => "Stopped".to_string(),
                        JobStatus::Done(code) => {
                            if let Some(c) = code {
                                format!("Done({})", c)
                            } else {
                                "Done".to_string()
                            }
                        }
                    };
                    println!("[{}] {} {} {}", job.id, status_str, job.pid, job.command);
                }
                Ok(())
            }
            "fg" => {
                use crate::jobs::JobStatus;
                let spec = command.args.first().map(|s| s.as_str()).unwrap_or("+");
                if let Some(job) = self.job_manager.get_job(spec) {
                    match job.status {
                        JobStatus::Stopped => {
                            // In a real implementation, we would resume the process
                            println!("Bringing job {} to foreground", job.id);
                        }
                        JobStatus::Done(_) => {
                            return Err(Error::Config(format!("fg: job {} already completed", job.id)));
                        }
                        JobStatus::Running => {
                            println!("Job {} is already running", job.id);
                        }
                    }
                } else {
                    return Err(Error::Config(format!("fg: job not found: {}", spec)));
                }
                Ok(())
            }
            "bg" => {
                use crate::jobs::JobStatus;
                let spec = command.args.first().map(|s| s.as_str()).unwrap_or("+");
                if let Some(job) = self.job_manager.get_job(spec) {
                    match job.status {
                        JobStatus::Stopped => {
                            // In a real implementation, we would resume the process in background
                            println!("Resuming job {} in background", job.id);
                            self.job_manager.update_job_status(job.pid, JobStatus::Running);
                        }
                        JobStatus::Done(_) => {
                            return Err(Error::Config(format!("bg: job {} already completed", job.id)));
                        }
                        JobStatus::Running => {
                            println!("Job {} is already running", job.id);
                        }
                    }
                } else {
                    return Err(Error::Config(format!("bg: job not found: {}", spec)));
                }
                Ok(())
            }
            "kill" => {
                use crate::jobs::JobStatus;
                if let Some(spec) = command.args.first() {
                    if spec.starts_with('%') {
                        // Job kill
                        if let Some(job) = self.job_manager.get_job(spec) {
                            // In a real implementation, we would send SIGTERM to the process
                            println!("Killing job {}", job.id);
                            self.job_manager.update_job_status(job.pid, JobStatus::Done(Some(130))); // SIGTERM
                            self.job_manager.remove_job(job.id);
                        } else {
                            return Err(Error::Config(format!("kill: job not found: {}", spec)));
                        }
                    } else {
                        // Process kill - handled by executor
                        return self.executor.execute(command);
                    }
                }
                Ok(())
            }
            "wait" => {
                use crate::jobs::JobStatus;
                if let Some(spec) = command.args.first() {
                    if spec.starts_with('%') {
                        if let Some(job) = self.job_manager.get_job(spec) {
                            // In a real implementation, we would wait for the process
                            match job.status {
                                JobStatus::Done(code) => {
                                    println!("Job {} already completed with code {:?}", job.id, code);
                                }
                                JobStatus::Running | JobStatus::Stopped => {
                                    println!("Waiting for job {}...", job.id);
                                    // In a real implementation, we would block here
                                }
                            }
                        } else {
                            return Err(Error::Config(format!("wait: job not found: {}", spec)));
                        }
                    }
                }
                Ok(())
            }
            _ => Err(Error::Config(format!("Unknown builtin: {}", command.name))),
        }
    }

    fn builtin_dirs(&self) {
        let current = std::env::current_dir().unwrap_or_default();
        print!("{}", current.display());
        for dir in self.directory_stack.iter().rev() {
            print!(" {}", dir.display());
        }
        println!();
    }

    /// Execute a script file
    fn execute_script(&mut self, script_path: &str) -> Result<()> {
        let content = std::fs::read_to_string(script_path)
            .map_err(|e| Error::Config(format!("Failed to read script: {}", e)))?;
        
        let mut script_exit_code = 0;
        
        // Execute each line
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Err(e) = self.execute_line(line) {
                eprintln!("Error executing script line: {}", e);
                script_exit_code = 1;
                // Continue execution (don't stop on error unless explicitly needed)
            } else {
                // Update script exit code with last command's exit code
                script_exit_code = self.last_exit_code;
            }
        }
        
        // Set script exit code as last exit code
        self.last_exit_code = script_exit_code;
        
        Ok(())
    }

    /// Execute a command in the background
    fn execute_background(&mut self, command: &crate::command::Command) -> Result<()> {
        use std::process::{Command as ProcessCommand, Stdio};
        use crate::jobs::JobStatus;
        
        let mut process = ProcessCommand::new(&command.name);
        process.args(&command.args);
        process.stdin(Stdio::null());
        process.stdout(Stdio::null());
        process.stderr(Stdio::null());
        
        // Spawn process
        let mut child = process.spawn()
            .map_err(|e| Error::Config(format!("Failed to spawn process: {}", e)))?;
        
        let pid = child.id();
        let command_str = format!("{} {}", command.name, command.args.join(" "));
        
        // Add job to job manager
        let job_id = self.job_manager.add_job(command_str.clone(), pid);
        println!("[{}] {}", job_id, pid);
        
        // Check if process completed immediately
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process already finished
                let exit_code = status.code();
                self.job_manager.update_job_status(pid, JobStatus::Done(exit_code));
            }
            Ok(None) => {
                // Process is still running - in a full implementation, we'd spawn a task
                // to monitor it and update status when it completes
            }
            Err(_) => {
                // Error checking status - mark as stopped
                self.job_manager.update_job_status(pid, JobStatus::Stopped);
            }
        }
        
        Ok(())
    }
}

