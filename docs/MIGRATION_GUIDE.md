# Migration Guide: v0.1.0 to v0.2.0

This guide helps you migrate from Ziron v0.1.0 to v0.2.0, covering new features, configuration changes, and best practices.

---

## Overview

Ziron v0.2.0 introduces significant enhancements:
- **Advanced parsing**: Quoted strings, variable expansion, command substitution, arithmetic expansion, globbing, tilde expansion, brace expansion
- **Tab completion**: File, command, variable, and custom completion
- **Extended built-ins**: Additional commands for job control, directory stack, and system management
- **Advanced theming**: Multi-line prompts, right-side prompts, color presets
- **Performance improvements**: Optimized parsing and completion lookup

---

## Breaking Changes

**None!** v0.2.0 maintains full backward compatibility with v0.1.0. All existing configurations, themes, and scripts will continue to work.

---

## New Features

### 1. Advanced Command Parsing

#### Quoted Strings
You can now use quotes to handle spaces and special characters:

```bash
# v0.1.0: Limited quote support
echo hello world

# v0.2.0: Full quote support
echo "hello world"
echo 'single quoted string'
echo "nested 'quotes' work"
```

#### Variable Expansion
Enhanced variable expansion with modifiers:

```bash
# Basic expansion
echo $HOME

# Default value
echo ${VAR:-default}

# Alternative value
echo ${VAR:+value}

# Error on unset
echo ${VAR?error message}

# Variable length
echo ${#VAR}

# Substring
echo ${VAR:0:5}
```

#### Command Substitution
Execute commands and use their output:

```bash
# Backtick syntax
echo `date`

# Dollar-paren syntax (preferred)
echo $(date)

# Nested substitution
echo $(echo $(date))
```

#### Arithmetic Expansion
Perform calculations:

```bash
# Basic arithmetic
echo $((2 + 2))

# With variables
x=10
echo $((x * 2))

# Bitwise operations
echo $((5 & 3))

# Comparison
echo $((x > 5 ? 1 : 0))
```

#### Globbing
Pattern matching for files:

```bash
# Wildcard
ls *.txt

# Single character
ls file?.txt

# Character class
ls file[0-9].txt

# Negated class
ls file[!0-9].txt
```

#### Tilde Expansion
Home directory shortcuts:

```bash
# Home directory
cd ~

# User home
cd ~user

# Current directory
cd ~+

# Previous directory
cd ~-
```

#### Brace Expansion
Generate multiple strings:

```bash
# Simple expansion
echo {a,b,c}

# Range expansion
echo {1..10}

# Nested expansion
echo {a,{b,c}}

# Prefix/suffix
echo prefix{a,b}suffix
```

### 2. Tab Completion

Tab completion is now available for:
- **Commands**: Built-ins, PATH executables, aliases, functions
- **Files**: Files and directories with path completion
- **Variables**: Environment variables (with `$` prefix)
- **Custom**: Register custom completion functions

**Usage:**
- Press `Tab` to complete
- Press `Tab` twice to see all matches
- Use arrow keys to navigate completion menu

**Configuration:**
```toml
[completion]
partial_completion = true  # Auto-complete common prefix
```

### 3. Extended Built-in Commands

#### Job Control
```bash
# List jobs
jobs

# Bring job to foreground
fg %1

# Send job to background
bg %1

# Kill job
kill %1

# Wait for job
wait %1
```

#### Directory Stack
```bash
# Push directory
pushd /path/to/dir

# Pop directory
popd

# List directory stack
dirs
```

#### System Commands
```bash
# Resource limits
ulimit -a

# File mode mask
umask

# Process times
times
```

### 4. Advanced Theming

#### Multi-line Prompts
```toml
[theme]
multiline = true
```

#### Right-side Prompt
```toml
[[theme.right_segments]]
module = "time"
color = "#888888"
```

#### Color Presets
```toml
[theme]
preset = "dark"  # Options: dark, light, solarized, nord
```

Available presets:
- `dark`: Dark theme with blue accents
- `light`: Light theme with blue accents
- `solarized`: Solarized color scheme
- `nord`: Nord color scheme

#### Custom Color Palette
```toml
[theme.color_palette]
primary = "#58a6ff"
secondary = "#79c0ff"
success = "#3fb950"
warning = "#d29922"
error = "#f85149"
```

---

## Configuration Migration

### v0.1.0 Configuration
```toml
[shell]
default = "zsh"

[performance]
cache_ttl_ms = 50

modules = ["git", "sysinfo"]
theme = "default"
```

### v0.2.0 Configuration (Backward Compatible)
```toml
[shell]
default = "zsh"

[performance]
cache_ttl_ms = 50

[completion]
partial_completion = true

modules = ["git", "sysinfo"]
theme = "default"
```

**No changes required!** Your existing configuration will work as-is. The new `[completion]` section is optional and uses sensible defaults.

---

## Theme Migration

### v0.1.0 Theme
```toml
[theme]
name = "my-theme"
background = "#15161e"

[[segments]]
module = "symbol"
color = "#58a6ff"
separator = " "

[[segments]]
module = "sysinfo"
color = "#c9d1d9"
separator = " > "
```

### v0.2.0 Theme (Backward Compatible)
```toml
[theme]
name = "my-theme"
preset = "dark"  # New: Use color preset
background = "#15161e"

[theme.color_palette]  # New: Custom colors
primary = "#58a6ff"
secondary = "#79c0ff"

[[segments]]
module = "symbol"
color = "#58a6ff"
separator = " "

[[segments]]
module = "sysinfo"
color = "#c9d1d9"
separator = " > "
```

**No changes required!** Existing themes work without modification. You can optionally add:
- `preset` field for color presets
- `color_palette` for custom colors
- `multiline` for multi-line prompts
- `right_segments` for right-side prompts

---

## Script Migration

### v0.1.0 Scripts
```bash
# Basic script
echo "Hello"
cd /path/to/dir
ls
```

### v0.2.0 Scripts (Backward Compatible)
```bash
# Same script works!
echo "Hello"
cd /path/to/dir
ls

# But now you can use advanced features:
echo "Hello $(whoami)"
x=$((2 + 2))
echo "Result: $x"
cd ~/projects/{project1,project2}
```

**No changes required!** All existing scripts continue to work. You can optionally use new features.

---

## Performance Improvements

v0.2.0 includes several performance optimizations:
- **Parsing**: Optimized token allocation using `mem::take`
- **Completion**: HashSet-based PATH deduplication
- **Memory**: Reduced allocations in common paths

These improvements are automatic and require no configuration changes.

---

## Troubleshooting

### Issue: Completion not working
**Solution**: Ensure your terminal supports ANSI escape codes. Try:
```bash
export TERM=xterm-256color
```

### Issue: Scripts with quotes fail
**Solution**: Check for unclosed quotes. Use:
```bash
echo "string with 'nested' quotes"
```

### Issue: Variable expansion not working
**Solution**: Use `${VAR}` syntax for complex cases:
```bash
echo ${VAR:-default}
```

### Issue: Theme colors not applying
**Solution**: Check theme file syntax. Ensure color codes are valid hex:
```toml
color = "#58a6ff"  # Correct
color = "58a6ff"   # Missing #
```

---

## Best Practices

### 1. Use Dollar-Paren for Command Substitution
```bash
# Preferred
echo $(date)

# Also works but less readable
echo `date`
```

### 2. Quote Variables with Spaces
```bash
# Safe
echo "$VAR"

# Risky if VAR contains spaces
echo $VAR
```

### 3. Use Brace Expansion for Repetitive Commands
```bash
# Instead of
mkdir dir1 dir2 dir3

# Use
mkdir {dir1,dir2,dir3}
```

### 4. Leverage Color Presets
```toml
# Instead of defining all colors manually
[theme]
preset = "dark"

# Then override only what you need
[theme.color_palette]
primary = "#custom-color"
```

### 5. Enable Partial Completion
```toml
[completion]
partial_completion = true  # Auto-complete common prefix
```

---

## Upgrading

1. **Backup your configuration** (optional but recommended):
   ```bash
   cp ~/.config/ziron/config.toml ~/.config/ziron/config.toml.backup
   ```

2. **Update Ziron**:
   ```bash
   cargo install --path . --force
   ```

3. **Test your configuration**:
   ```bash
   ziron-shell
   ```

4. **Try new features**:
   - Test tab completion
   - Try advanced parsing features
   - Experiment with new built-ins

---

## Getting Help

- **Documentation**: See `docs/` directory
- **Built-in Commands**: `docs/BUILTIN_COMMANDS.md`
- **Parsing Syntax**: `docs/PARSING_SYNTAX.md`
- **Completion System**: `docs/COMPLETION_SYSTEM.md`
- **Theming**: `docs/THEMING.md`

---

## Summary

- ✅ **No breaking changes**: All v0.1.0 features work in v0.2.0
- ✅ **Backward compatible**: Existing configs and themes work as-is
- ✅ **New features**: Advanced parsing, completion, and theming
- ✅ **Performance**: Automatic optimizations
- ✅ **Optional**: Use new features when ready

Enjoy the enhanced Ziron experience!

