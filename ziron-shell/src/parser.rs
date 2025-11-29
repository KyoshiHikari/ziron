//! Command parser with advanced features

use crate::command::{Command, Redirection};
use ziron_core::error::{Error, Result};

/// Context for variable expansion (script arguments, etc.)
pub struct ExpansionContext {
    pub script_args: Vec<String>,
}

impl Default for ExpansionContext {
    fn default() -> Self {
        Self {
            script_args: Vec::new(),
        }
    }
}

/// Command parser
pub struct Parser;

impl Parser {
    /// Parse a command line into commands
    #[allow(dead_code)] // Used in tests
    pub fn parse(line: &str) -> Result<Vec<Command>> {
        Self::parse_with_context(line, &ExpansionContext::default())
    }
    
    /// Parse a command line into commands with expansion context
    pub fn parse_with_context(line: &str, ctx: &ExpansionContext) -> Result<Vec<Command>> {
        let mut commands = Vec::new();
        
        if line.trim().is_empty() {
            return Ok(commands);
        }

        // Split by pipes, handling quoted strings
        let pipe_parts = Self::split_by_pipes(line)?;

        for (i, pipe_part) in pipe_parts.iter().enumerate() {
            let trimmed = pipe_part.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Parse tokens with quote handling and redirection
            let (tokens, stdout_redir, stderr_redir, stdin_redir) = Self::parse_with_redirection(trimmed, ctx)?;
            if tokens.is_empty() {
                continue;
            }

            let name = tokens[0].clone();
            let args = tokens[1..].to_vec();

            // Determine stdin: pipe from previous command OR input redirection
            let stdin_cmd = if i > 0 {
                Some(Box::new(commands[i - 1].clone()))
            } else {
                None
            };

            let command = Command {
                name,
                args,
                stdin: stdin_cmd,
                stdout: stdout_redir,
                stderr: stderr_redir,
                stdin_file: stdin_redir,
            };

            commands.push(command);
        }

        Ok(commands)
    }

    /// Split line by pipes, respecting quoted strings
    fn split_by_pipes(line: &str) -> Result<Vec<String>> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;

        for ch in line.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if !in_single_quote => {
                    escape_next = true;
                    current.push(ch);
                }
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                    current.push(ch);
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                    current.push(ch);
                }
                '|' if !in_single_quote && !in_double_quote => {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if in_single_quote || in_double_quote {
            return Err(Error::Config("Unclosed quote".to_string()));
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        Ok(parts)
    }

    /// Tokenize a command string, handling quotes and expansion
    fn tokenize(line: &str, ctx: &ExpansionContext) -> Result<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;

        for ch in line.chars() {
            if escape_next {
                if in_single_quote {
                    // In single quotes, only \' is special
                    if ch == '\'' {
                        current.push('\'');
                    } else {
                        current.push('\\');
                        current.push(ch);
                    }
                } else {
                    // Outside or in double quotes, handle escape sequences
                    match ch {
                        'n' => current.push('\n'),
                        't' => current.push('\t'),
                        'r' => current.push('\r'),
                        '\\' => current.push('\\'),
                        '"' => current.push('"'),
                        '$' => current.push('$'),
                        '`' => current.push('`'),
                        _ => {
                            current.push('\\');
                            current.push(ch);
                        }
                    }
                }
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if !in_single_quote => {
                    escape_next = true;
                }
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                }
                ' ' | '\t' if !in_single_quote && !in_double_quote => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if in_single_quote || in_double_quote {
            return Err(Error::Config("Unclosed quote".to_string()));
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        // Apply expansions
        let expanded_tokens: Result<Vec<String>> = tokens.iter()
            .map(|token| Self::expand_token(token, ctx))
            .collect();

        expanded_tokens
    }

    /// Parse tokens and extract redirections
    fn parse_with_redirection(line: &str, ctx: &ExpansionContext) -> Result<(Vec<String>, Option<Redirection>, Option<Redirection>, Option<Redirection>)> {
        let tokens = Self::tokenize(line, ctx)?;
        let mut result_tokens = Vec::new();
        let mut stdout_redir: Option<Redirection> = None;
        let mut stderr_redir: Option<Redirection> = None;
        let mut stdin_redir: Option<Redirection> = None;
        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];
            
            match token.as_str() {
                ">" => {
                    if i + 1 < tokens.len() {
                        stdout_redir = Some(Redirection::Output(tokens[i + 1].clone()));
                        i += 2;
                        continue;
                    }
                }
                ">>" => {
                    if i + 1 < tokens.len() {
                        stdout_redir = Some(Redirection::Append(tokens[i + 1].clone()));
                        i += 2;
                        continue;
                    }
                }
                "<" => {
                    if i + 1 < tokens.len() {
                        let next_token = &tokens[i + 1];
                        // Check for process substitution <(command)
                        if next_token.starts_with("<(") && next_token.ends_with(')') {
                            // Process substitution: <(command)
                            let cmd = &next_token[2..next_token.len()-1];
                            stdin_redir = Some(Redirection::Input(format!("<({})", cmd)));
                            i += 2;
                            continue;
                        } else {
                            // Regular input redirection
                            stdin_redir = Some(Redirection::Input(next_token.clone()));
                            i += 2;
                            continue;
                        }
                    }
                }
                "<<" => {
                    // Here-document delimiter
                    if i + 1 < tokens.len() {
                        let delimiter = tokens[i + 1].clone();
                        stdin_redir = Some(Redirection::Input(format!("<<{}", delimiter)));
                        i += 2;
                        continue;
                    }
                }
                "<<-" => {
                    // Here-document with dash (strip leading tabs)
                    if i + 1 < tokens.len() {
                        let delimiter = tokens[i + 1].clone();
                        stdin_redir = Some(Redirection::Input(format!("<<-{}", delimiter)));
                        i += 2;
                        continue;
                    }
                }
                "<<<" => {
                    // Here-string
                    if i + 1 < tokens.len() {
                        let content = tokens[i + 1].clone();
                        stdin_redir = Some(Redirection::Input(format!("<<<{}", content)));
                        i += 2;
                        continue;
                    }
                }
                "2>" => {
                    if i + 1 < tokens.len() {
                        stderr_redir = Some(Redirection::Error(tokens[i + 1].clone()));
                        i += 2;
                        continue;
                    }
                }
                "2>>" => {
                    if i + 1 < tokens.len() {
                        stderr_redir = Some(Redirection::ErrorAppend(tokens[i + 1].clone()));
                        i += 2;
                        continue;
                    }
                }
                "&>" => {
                    if i + 1 < tokens.len() {
                        let file = tokens[i + 1].clone();
                        stdout_redir = Some(Redirection::Combined(file.clone()));
                        stderr_redir = Some(Redirection::Combined(file));
                        i += 2;
                        continue;
                    }
                }
                _ => {
                    // Check for file descriptor redirection: n>, n<, n>&m
                    if let Some(fd_redir) = Self::parse_fd_redirection(token, &tokens, &mut i)? {
                        match fd_redir {
                            (Some(stdout), None, None) => stdout_redir = stdout,
                            (None, Some(stderr), None) => stderr_redir = stderr,
                            (None, None, Some(stdin)) => stdin_redir = stdin,
                            _ => {}
                        }
                        continue;
                    }
                    // Check for process substitution in token: <(command) or >(command)
                    if token.starts_with("<(") && token.ends_with(')') {
                        // Input process substitution
                        let cmd = &token[2..token.len()-1];
                        stdin_redir = Some(Redirection::Input(format!("<({})", cmd)));
                    } else if token.starts_with(">(") && token.ends_with(')') {
                        // Output process substitution
                        let cmd = &token[2..token.len()-1];
                        stdout_redir = Some(Redirection::Output(format!(">({})", cmd)));
                    } else {
                        result_tokens.push(token.clone());
                    }
                }
            }
            i += 1;
        }

        Ok((result_tokens, stdout_redir, stderr_redir, stdin_redir))
    }

    /// Parse file descriptor redirection: n>, n<, n>&m
    fn parse_fd_redirection(
        token: &str,
        tokens: &[String],
        i: &mut usize,
    ) -> Result<Option<(Option<Option<Redirection>>, Option<Option<Redirection>>, Option<Option<Redirection>>)>> {
        // Check for n> or n>> (file descriptor output)
        if let Some(pos) = token.find('>') {
            if pos > 0 && pos < token.len() {
                if let Ok(fd) = token[..pos].parse::<u32>() {
                    if *i + 1 < tokens.len() {
                        let file = tokens[*i + 1].clone();
                        let redir = if token.ends_with(">>") {
                            Redirection::Append(file)
                        } else {
                            Redirection::FdOutput(fd, file)
                        };
                        *i += 2;
                        return Ok(Some((Some(Some(redir)), None, None)));
                    }
                }
            }
        }
        
        // Check for n< (file descriptor input)
        if let Some(pos) = token.find('<') {
            if pos > 0 && pos < token.len() {
                if let Ok(fd) = token[..pos].parse::<u32>() {
                    if *i + 1 < tokens.len() {
                        let file = tokens[*i + 1].clone();
                        let redir = Redirection::FdInput(fd, file);
                        *i += 2;
                        return Ok(Some((None, None, Some(Some(redir)))));
                    }
                }
            }
        }
        
        // Check for n>&m (file descriptor duplication)
        if let Some(pos) = token.find(">&") {
            if pos > 0 && pos + 2 < token.len() {
                if let Ok(fd_from) = token[..pos].parse::<u32>() {
                    let rest = &token[pos + 2..];
                    if let Ok(fd_to) = rest.parse::<u32>() {
                        let _redir = Redirection::FdDup(fd_from, fd_to);
                        *i += 1;
                        // Map to appropriate redirection based on file descriptors
                        if fd_from == 1 {
                            return Ok(Some((Some(Some(Redirection::Output(format!("&{}", fd_to)))), None, None)));
                        } else if fd_from == 2 {
                            return Ok(Some((None, Some(Some(Redirection::Error(format!("&{}", fd_to)))), None)));
                        }
                        return Ok(Some((None, None, None)));
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Expand a single token (variable expansion, tilde expansion, command substitution, globbing, arithmetic)
    fn expand_token(token: &str, ctx: &ExpansionContext) -> Result<String> {
        // First handle command substitution
        let token = Self::expand_command_substitution(token)?;
        
        // Handle arithmetic expansion
        let token = Self::expand_arithmetic(&token)?;
        
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = token.chars().collect();

        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() {
                // Variable expansion
                if chars[i + 1] == '{' {
                    // ${VAR} syntax with possible modifiers
                    let mut j = i + 2;
                    let mut var_name = String::new();
                    let mut modifier = None;
                    
                    while j < chars.len() && chars[j] != '}' {
                        // Check for modifiers: ${VAR:-default}, ${VAR:+value}, ${VAR?error}, ${#VAR}
                        if j + 1 < chars.len() && chars[j] == ':' {
                            match chars[j + 1] {
                                '-' => {
                                    // ${VAR:-default}
                                    modifier = Some(("default", j + 2));
                                    break;
                                }
                                '+' => {
                                    // ${VAR:+value}
                                    modifier = Some(("alternative", j + 2));
                                    break;
                                }
                                '?' => {
                                    // ${VAR?error}
                                    modifier = Some(("error", j + 2));
                                    break;
                                }
                                _ => {
                                    var_name.push(chars[j]);
                                }
                            }
                        } else if chars[j] == '#' && var_name.is_empty() {
                            // ${#VAR} - variable length
                            modifier = Some(("length", j + 1));
                            break;
                        } else if chars[j] == ':' && j + 1 < chars.len() && 
                                  (chars[j + 1].is_ascii_digit() || chars[j + 1] == '-') {
                            // ${VAR:offset:length} - substring
                            modifier = Some(("substring", j + 1));
                            break;
                        } else {
                            var_name.push(chars[j]);
                        }
                        j += 1;
                    }
                    
                    if j < chars.len() {
                        if let Some((mod_type, mod_start)) = modifier {
                            let mod_end = j;
                            let mod_content = &token[mod_start..mod_end];
                            
                            match mod_type {
                                "default" => {
                                    let value = std::env::var(&var_name).unwrap_or_else(|_| mod_content.to_string());
                                    result.push_str(&value);
                                }
                                "alternative" => {
                                    if std::env::var(&var_name).is_ok() {
                                        result.push_str(mod_content);
                                    }
                                }
                                "error" => {
                                    if std::env::var(&var_name).is_err() {
                                        return Err(Error::Config(format!("Variable {} not set: {}", var_name, mod_content)));
                                    }
                                    let value = std::env::var(&var_name).unwrap();
                                    result.push_str(&value);
                                }
                                "length" => {
                                    let var_name_after_hash = &token[i + 3..j];
                                    let value = std::env::var(var_name_after_hash).unwrap_or_default();
                                    result.push_str(&value.len().to_string());
                                }
                                "substring" => {
                                    // ${VAR:offset:length} - simplified implementation
                                    let parts: Vec<&str> = mod_content.split(':').collect();
                                    let value = std::env::var(&var_name).unwrap_or_default();
                                    if parts.len() >= 1 {
                                        if let Ok(offset) = parts[0].parse::<usize>() {
                                            if offset < value.len() {
                                                if parts.len() >= 2 {
                                                    if let Ok(length) = parts[1].parse::<usize>() {
                                                        let end = (offset + length).min(value.len());
                                                        result.push_str(&value[offset..end]);
                                                    } else {
                                                        result.push_str(&value[offset..]);
                                                    }
                                                } else {
                                                    result.push_str(&value[offset..]);
                                                }
                                            }
                                        }
                                    } else {
                                        result.push_str(&value);
                                    }
                                }
                                _ => {
                                    let value = std::env::var(&var_name).unwrap_or_default();
                                    result.push_str(&value);
                                }
                            }
                        } else {
                            let value = std::env::var(&var_name).unwrap_or_default();
                            result.push_str(&value);
                        }
                        i = j + 1;
                        continue;
                    }
                } else if chars[i + 1] == '(' && i + 2 < chars.len() {
                    // $(command) syntax - already handled by expand_command_substitution
                    // Skip it here
                    i += 1;
                } else if chars[i + 1].is_ascii_digit() {
                    // Script argument: $1, $2, etc.
                    let mut j = i + 1;
                    while j < chars.len() && chars[j].is_ascii_digit() {
                        j += 1;
                    }
                    let arg_num_str = &token[i + 1..j];
                    if let Ok(arg_num) = arg_num_str.parse::<usize>() {
                        if arg_num > 0 && arg_num <= ctx.script_args.len() {
                            result.push_str(&ctx.script_args[arg_num - 1]);
                        }
                        i = j;
                        continue;
                    }
                } else if chars[i + 1] == '#' {
                    // $# - number of script arguments
                    result.push_str(&ctx.script_args.len().to_string());
                    i += 2;
                    continue;
                } else if chars[i + 1] == '@' {
                    // $@ - all script arguments as separate words
                    result.push_str(&ctx.script_args.join(" "));
                    i += 2;
                    continue;
                } else if chars[i + 1] == '*' {
                    // $* - all script arguments as single string
                    result.push_str(&ctx.script_args.join(" "));
                    i += 2;
                    continue;
                } else if chars[i + 1].is_alphanumeric() || chars[i + 1] == '_' {
                    // $VAR syntax
                    let mut var_name = String::new();
                    let mut j = i + 1;
                    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                        var_name.push(chars[j]);
                        j += 1;
                    }
                    let value = std::env::var(&var_name).unwrap_or_default();
                    result.push_str(&value);
                    i = j;
                    continue;
                }
            } else if chars[i] == '~' && (i == 0 || result.is_empty() || result.ends_with(' ') || result.ends_with('/')) {
                // Tilde expansion
                if i + 1 >= chars.len() || chars[i + 1] == '/' || chars[i + 1] == ' ' {
                    // ~ expansion
                    if let Ok(home) = std::env::var("HOME") {
                        result.push_str(&home);
                        i += 1;
                        continue;
                    }
                } else if i + 1 < chars.len() && chars[i + 1] == '+' {
                    // ~+ expansion (current directory / PWD)
                    if let Ok(pwd) = std::env::var("PWD") {
                        result.push_str(&pwd);
                        i += 2;
                        continue;
                    } else if let Ok(pwd) = std::env::current_dir() {
                        result.push_str(&pwd.to_string_lossy());
                        i += 2;
                        continue;
                    }
                } else if i + 1 < chars.len() && chars[i + 1] == '-' {
                    // ~- expansion (previous directory / OLDPWD)
                    if let Ok(oldpwd) = std::env::var("OLDPWD") {
                        result.push_str(&oldpwd);
                        i += 2;
                        continue;
                    }
                } else if i + 1 < chars.len() && chars[i + 1] != '/' {
                    // ~user expansion (simplified - just use HOME for now)
                    let mut j = i + 1;
                    while j < chars.len() && chars[j] != '/' && chars[j] != ' ' {
                        j += 1;
                    }
                    if let Ok(home) = std::env::var("HOME") {
                        result.push_str(&home);
                        i = j;
                        continue;
                    }
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        // Apply brace expansion
        let result = Self::expand_brace(&result)?;
        
        // Apply globbing
        Self::expand_glob(&result)
    }

    /// Expand brace expressions: {a,b,c} or {1..10} or {a,{b,c}} or prefix{a,b}suffix
    fn expand_brace(token: &str) -> Result<String> {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = token.chars().collect();
        let mut brace_start = None;
        let mut brace_depth = 0;

        while i < chars.len() {
            if chars[i] == '{' && (i == 0 || chars[i - 1] != '$') {
                if brace_depth == 0 {
                    brace_start = Some(i);
                }
                brace_depth += 1;
                i += 1;
                continue;
            }

            if chars[i] == '}' && brace_depth > 0 {
                brace_depth -= 1;
                if brace_depth == 0 {
                    if let Some(start) = brace_start {
                        // Extract prefix (text before the brace)
                        let prefix = if start > 0 {
                            token[..start].to_string()
                        } else {
                            String::new()
                        };
                        
                        // Extract suffix (text after the brace)
                        let suffix = if i + 1 < token.len() {
                            token[i + 1..].to_string()
                        } else {
                            String::new()
                        };
                        
                        let brace_content = &token[start + 1..i];
                        let expanded = Self::expand_brace_content(brace_content, &prefix, &suffix)?;
                        result.push_str(&expanded);
                        // Skip the suffix since we already included it in the expansion
                        if !suffix.is_empty() {
                            i += suffix.chars().count();
                        }
                        brace_start = None;
                        i += 1;
                        continue;
                    }
                }
            }

            if brace_depth == 0 {
                result.push(chars[i]);
            }
            i += 1;
        }

        if brace_depth > 0 {
            // Unclosed brace - return as-is
            return Ok(token.to_string());
        }

        Ok(result)
    }

    /// Expand brace content: a,b,c or 1..10 or {a,{b,c}} with prefix/suffix
    fn expand_brace_content(content: &str, prefix: &str, suffix: &str) -> Result<String> {
        // Check for range expansion: {1..10}
        if let Some(range_pos) = content.find("..") {
            let start_str = content[..range_pos].trim();
            let end_str = content[range_pos + 2..].trim();
            
            if let (Ok(start), Ok(end)) = (start_str.parse::<i32>(), end_str.parse::<i32>()) {
                let mut result = Vec::new();
                let step = if start <= end { 1 } else { -1 };
                let mut current = start;
                while (step > 0 && current <= end) || (step < 0 && current >= end) {
                    result.push(format!("{}{}{}", prefix, current, suffix));
                    current += step;
                }
                return Ok(result.join(" "));
            }
        }

        // Handle nested braces and comma-separated expansion
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut depth = 0;
        let mut i = 0;
        let chars: Vec<char> = content.chars().collect();
        
        while i < chars.len() {
            match chars[i] {
                '{' => {
                    depth += 1;
                    current_part.push(chars[i]);
                }
                '}' => {
                    depth -= 1;
                    current_part.push(chars[i]);
                }
                ',' if depth == 0 => {
                    // At top level, split on comma
                    parts.push(current_part.trim().to_string());
                    current_part.clear();
                }
                _ => {
                    current_part.push(chars[i]);
                }
            }
            i += 1;
        }
        
        if !current_part.is_empty() {
            parts.push(current_part.trim().to_string());
        }
        
        // Expand each part (recursively handle nested braces)
        let mut expanded_parts = Vec::new();
        for part in parts {
            // Check if this part contains nested braces
            if part.contains('{') && part.contains('}') {
                // Recursively expand nested braces
                let nested_expanded = Self::expand_brace(&part)?;
                // Split the nested expansion and apply prefix/suffix to each
                for nested_part in nested_expanded.split_whitespace() {
                    expanded_parts.push(format!("{}{}{}", prefix, nested_part, suffix));
                }
            } else {
                // Simple part, just apply prefix/suffix
                expanded_parts.push(format!("{}{}{}", prefix, part, suffix));
            }
        }
        
        Ok(expanded_parts.join(" "))
    }

    /// Expand arithmetic expressions: $((expression))
    fn expand_arithmetic(token: &str) -> Result<String> {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = token.chars().collect();

        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '(' && 
               i + 2 < chars.len() && chars[i + 2] == '(' {
                // $((expression)) syntax
                let mut depth = 2;
                let mut j = i + 3;
                let mut expr = String::new();
                
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth > 0 {
                                expr.push(chars[j]);
                            }
                        }
                        _ => expr.push(chars[j]),
                    }
                    j += 1;
                }

                if depth == 0 {
                    // Evaluate arithmetic expression
                    let value = Self::evaluate_arithmetic(&expr)?;
                    result.push_str(&value.to_string());
                    i = j;
                    continue;
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        Ok(result)
    }

    /// Evaluate an arithmetic expression
    fn evaluate_arithmetic(expr: &str) -> Result<i64> {
        let expr = expr.trim();
        
        // Simple arithmetic evaluator - handles basic operations
        // This is a simplified version; a full implementation would need proper parsing
        
        // Try to parse as integer first
        if let Ok(val) = expr.parse::<i64>() {
            return Ok(val);
        }

        // Handle operators in order of precedence (lowest to highest)
        // Logical OR
        if let Some(pos) = expr.rfind("||") {
            let left = expr[..pos].trim();
            let right = expr[pos + 2..].trim();
            let left_val = Self::evaluate_arithmetic(left)?;
            let right_val = Self::evaluate_arithmetic(right)?;
            return Ok(if left_val != 0 || right_val != 0 { 1 } else { 0 });
        }
        
        // Logical AND
        if let Some(pos) = expr.rfind("&&") {
            let left = expr[..pos].trim();
            let right = expr[pos + 2..].trim();
            let left_val = Self::evaluate_arithmetic(left)?;
            let right_val = Self::evaluate_arithmetic(right)?;
            return Ok(if left_val != 0 && right_val != 0 { 1 } else { 0 });
        }
        
        // Bitwise OR
        if let Some(pos) = expr.rfind('|') {
            let left = expr[..pos].trim();
            let right = expr[pos + 1..].trim();
            let left_val = Self::evaluate_arithmetic(left)?;
            let right_val = Self::evaluate_arithmetic(right)?;
            return Ok(left_val | right_val);
        }
        
        // Bitwise XOR
        if let Some(pos) = expr.rfind('^') {
            let left = expr[..pos].trim();
            let right = expr[pos + 1..].trim();
            let left_val = Self::evaluate_arithmetic(left)?;
            let right_val = Self::evaluate_arithmetic(right)?;
            return Ok(left_val ^ right_val);
        }
        
        // Bitwise AND
        if let Some(pos) = expr.rfind('&') {
            let left = expr[..pos].trim();
            let right = expr[pos + 1..].trim();
            let left_val = Self::evaluate_arithmetic(left)?;
            let right_val = Self::evaluate_arithmetic(right)?;
            return Ok(left_val & right_val);
        }
        
        // Comparison operators
        for op in ["==", "!=", "<=", ">=", "<", ">"] {
            if let Some(pos) = expr.rfind(op) {
                let left = expr[..pos].trim();
                let right = expr[pos + op.len()..].trim();
                let left_val = Self::evaluate_arithmetic(left)?;
                let right_val = Self::evaluate_arithmetic(right)?;
                
                return Ok(match op {
                    "==" => if left_val == right_val { 1 } else { 0 },
                    "!=" => if left_val != right_val { 1 } else { 0 },
                    "<=" => if left_val <= right_val { 1 } else { 0 },
                    ">=" => if left_val >= right_val { 1 } else { 0 },
                    "<" => if left_val < right_val { 1 } else { 0 },
                    ">" => if left_val > right_val { 1 } else { 0 },
                    _ => return Err(Error::Config(format!("Unknown comparison operator: {}", op))),
                });
            }
        }
        
        // Bitwise shift operators
        if let Some(pos) = expr.rfind("<<") {
            let left = expr[..pos].trim();
            let right = expr[pos + 2..].trim();
            let left_val = Self::evaluate_arithmetic(left)?;
            let right_val = Self::evaluate_arithmetic(right)?;
            return Ok(left_val << right_val);
        }
        
        if let Some(pos) = expr.rfind(">>") {
            let left = expr[..pos].trim();
            let right = expr[pos + 2..].trim();
            let left_val = Self::evaluate_arithmetic(left)?;
            let right_val = Self::evaluate_arithmetic(right)?;
            return Ok(left_val >> right_val);
        }
        
        // Bitwise NOT
        if expr.starts_with('~') {
            let val = Self::evaluate_arithmetic(&expr[1..])?;
            return Ok(!val);
        }
        
        // Logical NOT
        if expr.starts_with('!') {
            let val = Self::evaluate_arithmetic(&expr[1..])?;
            return Ok(if val == 0 { 1 } else { 0 });
        }

        // Handle basic binary operations
        for op in ["+", "-", "*", "/", "%"] {
            if let Some(pos) = expr.rfind(op) {
                let left = expr[..pos].trim();
                let right = expr[pos + op.len()..].trim();
                
                let left_val = Self::evaluate_arithmetic(left)?;
                let right_val = Self::evaluate_arithmetic(right)?;
                
                return match op {
                    "+" => Ok(left_val + right_val),
                    "-" => Ok(left_val - right_val),
                    "*" => Ok(left_val * right_val),
                    "/" => {
                        if right_val == 0 {
                            return Err(Error::Config("Division by zero".to_string()));
                        }
                        Ok(left_val / right_val)
                    }
                    "%" => {
                        if right_val == 0 {
                            return Err(Error::Config("Modulo by zero".to_string()));
                        }
                        Ok(left_val % right_val)
                    }
                    _ => Err(Error::Config(format!("Unknown operator: {}", op))),
                };
            }
        }

        // Try variable expansion
        if expr.starts_with('$') {
            let var_name = &expr[1..];
            if let Ok(val_str) = std::env::var(var_name) {
                return val_str.parse::<i64>()
                    .map_err(|_| Error::Config(format!("Variable {} is not a number", var_name)));
            }
        }

        Err(Error::Config(format!("Invalid arithmetic expression: {}", expr)))
    }

    /// Expand command substitution: $(command) or `command`
    fn expand_command_substitution(token: &str) -> Result<String> {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = token.chars().collect();

        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
                // $(command) syntax
                let mut depth = 1;
                let mut j = i + 2;
                let mut command = String::new();
                
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        command.push(chars[j]);
                    }
                    j += 1;
                }

                if depth == 0 {
                    // Execute command
                    let output = Self::execute_substitution(&command)?;
                    result.push_str(&output);
                    i = j;
                    continue;
                }
            } else if chars[i] == '`' {
                // `command` syntax
                let mut j = i + 1;
                let mut command = String::new();
                
                while j < chars.len() && chars[j] != '`' {
                    if chars[j] == '\\' && j + 1 < chars.len() {
                        j += 1;
                        command.push(chars[j]);
                    } else {
                        command.push(chars[j]);
                    }
                    j += 1;
                }

                if j < chars.len() {
                    // Execute command
                    let output = Self::execute_substitution(&command)?;
                    result.push_str(&output);
                    i = j + 1;
                    continue;
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        Ok(result)
    }

    /// Execute a command substitution
    fn execute_substitution(command: &str) -> Result<String> {
        use std::process::Command;
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| Error::Config(format!("Command substitution failed: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim_end().to_string())
    }

    /// Expand glob patterns
    fn expand_glob(pattern: &str) -> Result<String> {
        // Check if pattern contains glob characters
        if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            match glob::glob(pattern) {
                Ok(paths) => {
                    let matches: Vec<String> = paths
                        .filter_map(|p| p.ok())
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    
                    if matches.is_empty() {
                        // No matches, return pattern as-is
                        Ok(pattern.to_string())
                    } else {
                        // Return first match (simplified - in real shell, all matches would be separate args)
                        Ok(matches[0].clone())
                    }
                }
                Err(_) => Ok(pattern.to_string()),
            }
        } else {
            Ok(pattern.to_string())
        }
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let commands = Parser::parse("ls -la").unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "ls");
        assert_eq!(commands[0].args, vec!["-la"]);
    }

    #[test]
    fn test_parse_pipe() {
        let commands = Parser::parse("ls | grep test").unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].name, "ls");
        assert_eq!(commands[1].name, "grep");
    }

    #[test]
    fn test_parse_quotes() {
        let commands = Parser::parse(r#"echo "hello world""#).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "echo");
        assert_eq!(commands[0].args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_single_quotes() {
        let commands = Parser::parse("echo 'hello world'").unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "echo");
        assert_eq!(commands[0].args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_variable_expansion() {
        std::env::set_var("TEST_VAR", "test_value");
        let commands = Parser::parse("echo $TEST_VAR").unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].args[0], "test_value");
    }

    #[test]
    fn test_script_argument_expansion() {
        let ctx = ExpansionContext {
            script_args: vec!["arg1".to_string(), "arg2".to_string()],
        };
        let commands = Parser::parse_with_context("echo $1", &ctx).unwrap();
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn test_arithmetic_expansion() {
        let commands = Parser::parse("echo $((2 + 3))").unwrap();
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn test_brace_expansion() {
        let commands = Parser::parse("echo {a,b,c}").unwrap();
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn test_redirection_parsing() {
        let commands = Parser::parse("echo test > output.txt").unwrap();
        assert_eq!(commands.len(), 1);
        assert!(commands[0].stdout.is_some());
    }

    #[test]
    fn test_input_redirection() {
        let commands = Parser::parse("cat < input.txt").unwrap();
        assert_eq!(commands.len(), 1);
        assert!(commands[0].stdin_file.is_some());
    }

    #[test]
    fn test_here_string() {
        let commands = Parser::parse("cat <<< hello").unwrap();
        assert_eq!(commands.len(), 1);
        assert!(commands[0].stdin_file.is_some());
    }
}
