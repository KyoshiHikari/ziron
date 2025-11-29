# Ziron Completion System

This document describes the tab completion system in Ziron shell.

## Overview

Ziron provides intelligent tab completion for commands, files, variables, and more. The completion system is extensible and supports custom completion functions.

## Basic Completion

### Command Completion
Press `Tab` to complete command names:
- Built-in commands (`cd`, `echo`, `pwd`, etc.)
- External commands (from PATH)
- Aliases
- Functions

**Examples:**
```bash
cd <Tab>        # Completes to: cd
ech<Tab>        # Completes to: echo
```

### File and Directory Completion
Press `Tab` to complete file and directory names:
- Current directory files
- Subdirectories
- Hidden files (with `.` prefix)
- Path completion

**Examples:**
```bash
cat file<Tab>           # Completes file names
cd /usr/b<Tab>          # Completes to /usr/bin
ls .zir<Tab>            # Completes hidden files
```

### Variable Completion
Press `Tab` after `$` to complete environment variable names:

**Examples:**
```bash
echo $HO<Tab>           # Completes to $HOME
echo $PAT<Tab>          # Completes to $PATH
```

---

## Advanced Completion

### Function Completion
Functions defined with `function` are available for completion:

**Examples:**
```bash
function myfunc() { echo "Hello"; }
my<Tab>                 # Completes to: myfunc
```

### Case-Insensitive Completion
Completion can be configured for case-insensitive matching (if enabled).

---

## Custom Completion Functions

### Registration
Custom completion functions can be registered for specific commands:

```rust
completer.register_completion_function(
    "command".to_string(),
    Box::new(MyCompletionFunction)
);
```

### Completion Function API
Completion functions implement the `CompletionFunction` trait:

```rust
pub trait CompletionFunction: Send + Sync {
    fn complete(&self, word: &str, line: &str, pos: usize) -> Vec<String>;
}
```

### Context-Aware Completion
Completion functions receive:
- `word`: The current word being completed
- `line`: The entire command line
- `pos`: Cursor position

This allows for context-aware completions based on:
- Command arguments
- Previous words in the line
- Current directory
- Environment state

---

## Completion UI

### Hints
When a single match is found, a hint is displayed showing the remaining characters.

**Example:**
```bash
cd /usr/b<Tab>
# Hint shows: in
# Result: cd /usr/bin
```

### Multiple Matches
When multiple matches are found:
- All matches are displayed
- User can continue typing to narrow down
- Or press Tab again to cycle through matches

---

## Configuration

### Case-Insensitive Completion
Enable case-insensitive completion:

```rust
completer.set_case_insensitive(true);
```

---

## Built-in Completion Support

The following commands have built-in completion support:

- **File operations**: `cd`, `cat`, `ls`, etc.
- **Built-in commands**: All Ziron built-ins
- **Environment variables**: All `$VAR` expansions
- **Aliases**: All defined aliases
- **Functions**: All defined functions

---

## Examples

### Command Completion
```bash
# Type partial command
ech<Tab>
# Completes to: echo

# Multiple matches
c<Tab>
# Shows: cd, cat, chmod, etc.
```

### File Completion
```bash
# Complete file name
cat file<Tab>
# Shows matching files

# Complete directory
cd /usr/<Tab>
# Shows subdirectories
```

### Variable Completion
```bash
# Complete variable
echo $HO<Tab>
# Completes to: $HOME
```

### Function Completion
```bash
# Define function
function greet() { echo "Hello, $1!"; }

# Complete function name
gre<Tab>
# Completes to: greet
```

---

## Implementation Details

### Completion Order
1. Built-in commands
2. Aliases
3. Functions
4. PATH executables
5. Files and directories
6. Environment variables

### Performance
- Command list is cached
- PATH is scanned once per session
- File completions are on-demand

---

## Troubleshooting

### Completion Not Working
- Ensure the command is in PATH
- Check if the command is a built-in
- Verify file permissions

### Too Many Matches
- Continue typing to narrow down
- Use more specific patterns
- Check if case-insensitive mode is interfering

---

## Future Enhancements

Planned features:
- Multi-column completion display
- Completion menu navigation
- Partial completion acceptance
- Command-specific completion rules
- History-based completion

