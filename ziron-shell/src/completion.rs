//! Tab completion system

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Context;
use rustyline::Helper;
use rustyline::Result as RustylineResult;
use std::env;

/// Built-in commands for completion
const BUILTIN_COMMANDS: &[&str] = &[
    "cd", "exit", "pwd", "echo", "export", "unset", "history",
    "alias", "unalias", "type", "which", "source", "jobs", "fg", "bg",
    "kill", "wait", "ulimit", "umask", "times", "pushd", "popd", "dirs",
    "read", "printf", "test", "true", "false",
];

/// Completion function trait for custom completions
pub trait CompletionFunction: Send + Sync {
    fn complete(&self, word: &str, line: &str, pos: usize) -> Vec<String>;
}

/// Completion helper for Ziron shell
pub struct ZironCompleter {
    filename_completer: FilenameCompleter,
    aliases: Vec<String>,
    functions: Vec<String>,
    completion_functions: std::collections::HashMap<String, Box<dyn CompletionFunction>>,
    case_insensitive: bool,
    partial_completion: bool,
}

impl Clone for ZironCompleter {
    fn clone(&self) -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            aliases: self.aliases.clone(),
            functions: self.functions.clone(),
            completion_functions: std::collections::HashMap::new(), // Can't clone trait objects
            case_insensitive: self.case_insensitive,
            partial_completion: self.partial_completion,
        }
    }
}

impl ZironCompleter {
    pub fn new() -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            aliases: Vec::new(),
            functions: Vec::new(),
            completion_functions: std::collections::HashMap::new(),
            case_insensitive: false,
            partial_completion: true, // Enable by default
        }
    }

    /// Set partial completion enabled/disabled
    pub fn set_partial_completion(&mut self, enabled: bool) {
        self.partial_completion = enabled;
    }

    pub fn add_alias(&mut self, alias: String) {
        if !self.aliases.contains(&alias) {
            self.aliases.push(alias);
        }
    }

    pub fn remove_alias(&mut self, alias: &str) {
        self.aliases.retain(|a| a != alias);
    }

    pub fn add_function(&mut self, function: String) {
        if !self.functions.contains(&function) {
            self.functions.push(function);
        }
    }

    #[allow(dead_code)]
    pub fn remove_function(&mut self, function: &str) {
        self.functions.retain(|f| f != function);
    }

    #[allow(dead_code)]
    pub fn register_completion_function(&mut self, command: String, func: Box<dyn CompletionFunction>) {
        self.completion_functions.insert(command, func);
    }

    #[allow(dead_code)]
    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.case_insensitive = case_insensitive;
    }

    /// Get all available commands (built-ins + PATH executables + aliases + functions)
    fn get_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();

        // Add built-in commands
        commands.extend(BUILTIN_COMMANDS.iter().map(|s| s.to_string()));

        // Add aliases
        commands.extend(self.aliases.iter().cloned());

        // Add functions
        commands.extend(self.functions.iter().cloned());

        // Add PATH executables (optimized: use HashSet for deduplication)
        let mut path_commands_set = std::collections::HashSet::new();
        if let Ok(path) = env::var("PATH") {
            for dir in path.split(':') {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        if let Ok(file_name) = entry.file_name().into_string() {
                            path_commands_set.insert(file_name);
                        }
                    }
                }
            }
        }
        commands.extend(path_commands_set);

        commands.sort();
        commands
    }

    /// Check if two strings match (case-sensitive or case-insensitive)
    fn matches(&self, text: &str, pattern: &str) -> bool {
        if self.case_insensitive {
            text.to_lowercase().starts_with(&pattern.to_lowercase())
        } else {
            text.starts_with(pattern)
        }
    }

    /// Get environment variables for completion
    fn get_env_vars(&self) -> Vec<String> {
        env::vars()
            .map(|(key, _)| key)
            .collect()
    }

    /// Find common prefix of multiple strings
    fn find_common_prefix(strings: &[String]) -> String {
        if strings.is_empty() {
            return String::new();
        }
        if strings.len() == 1 {
            return strings[0].clone();
        }

        let first = &strings[0];
        let mut prefix_len = first.len();

        for s in strings.iter().skip(1) {
            prefix_len = prefix_len.min(s.len());
            for (i, ch) in first.chars().take(prefix_len).enumerate() {
                if s.chars().nth(i) != Some(ch) {
                    prefix_len = i;
                    break;
                }
            }
        }

        first.chars().take(prefix_len).collect()
    }

    /// Apply partial completion if enabled
    fn apply_partial_completion(&self, matches: &mut Vec<Pair>, prefix: &str) {
        if !self.partial_completion || matches.len() <= 1 {
            return;
        }

        // Get all replacement strings
        let replacements: Vec<String> = matches.iter().map(|p| p.replacement.clone()).collect();
        let common_prefix = Self::find_common_prefix(&replacements);

        // If common prefix is longer than current prefix, use it
        if common_prefix.len() > prefix.len() && common_prefix.starts_with(prefix) {
            // Replace all matches with a single match that has the common prefix
            matches.clear();
            matches.push(Pair {
                display: format!("{} (common prefix)", common_prefix),
                replacement: common_prefix,
            });
        }
    }
}

impl Completer for ZironCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        let line_before_cursor = &line[..pos];
        let words: Vec<&str> = line_before_cursor.split_whitespace().collect();

        // If we're completing the first word, it's a command
        if words.is_empty() || (words.len() == 1 && !line_before_cursor.ends_with(' ')) {
            // Complete command names
            let prefix = words.first().copied().unwrap_or("");
            let commands = self.get_commands();
            let mut matches: Vec<Pair> = commands
                .iter()
                .filter(|cmd| self.matches(cmd, prefix))
                .map(|cmd| {
                    let display = format!("{} (command)", cmd);
                    Pair {
                        display,
                        replacement: cmd.clone(),
                    }
                })
                .collect();

            if !matches.is_empty() {
                // Apply partial completion if enabled
                self.apply_partial_completion(&mut matches, prefix);
                let start_pos = if prefix.is_empty() { pos } else { pos - prefix.len() };
                return Ok((start_pos, matches));
            }
        }

        // Check for custom completion function for the command
        if words.len() >= 1 {
            let command = words[0];
            if let Some(completion_func) = self.completion_functions.get(command) {
                let current_word = words.last().copied().unwrap_or("");
                let completions = completion_func.complete(current_word, line, pos);
                if !completions.is_empty() {
                    let matches: Vec<Pair> = completions
                        .iter()
                        .filter(|comp| self.matches(comp, current_word))
                        .map(|comp| {
                            Pair {
                                display: comp.clone(),
                                replacement: comp.clone(),
                            }
                        })
                        .collect();
                    
                    if !matches.is_empty() {
                        let start_pos = if current_word.is_empty() { 
                            pos 
                        } else { 
                            line_before_cursor.rfind(current_word).unwrap_or(pos - current_word.len())
                        };
                        return Ok((start_pos, matches));
                    }
                }
            }
        }

        // Check if we're completing a variable (starts with $)
        if let Some(last_word) = words.last() {
            if last_word.starts_with('$') && !last_word.contains('/') {
                let var_prefix = &last_word[1..];
                let env_vars = self.get_env_vars();
                let matches: Vec<Pair> = env_vars
                    .iter()
                    .filter(|var| self.matches(var, var_prefix))
                    .map(|var| {
                        let replacement = format!("${}", var);
                        let display = format!("{} (env var)", replacement);
                        Pair {
                            display,
                            replacement,
                        }
                    })
                    .collect();

                if !matches.is_empty() {
                    let start_pos = line_before_cursor.rfind('$').unwrap_or(pos);
                    return Ok((start_pos, matches));
                }
            }
        }

        // Otherwise, complete as filename
        self.filename_completer.complete(line, pos, ctx)
    }
}

impl Hinter for ZironCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        // Provide hints for commands (without caching for simplicity)
        let line_before_cursor = &line[..pos];
        let words: Vec<&str> = line_before_cursor.split_whitespace().collect();
        
        if let Some(first_word) = words.first() {
            let commands = self.get_commands();
            let matches: Vec<&String> = commands
                .iter()
                .filter(|cmd| self.matches(cmd, first_word))
                .collect();
            
            if matches.len() == 1 && matches[0] != first_word {
                // Single match - provide hint
                let match_str = matches[0];
                let hint_start = if self.case_insensitive {
                    first_word.len()
                } else {
                    first_word.len()
                };
                if hint_start < match_str.len() {
                    return Some(match_str[hint_start..].to_string());
                }
            }
        }
        
        None
    }
}

impl Highlighter for ZironCompleter {}

impl Validator for ZironCompleter {}

impl Helper for ZironCompleter {}

