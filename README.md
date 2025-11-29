# Ziron

A modern, modular, and extensible shell interpreter written in Rust.

## Overview

Ziron is a **full-featured shell** like `zsh`, `bash`, or `fish`, but with:
- Modern, extensible theming system
- Easy configuration (TOML-based)
- Plugin architecture for modules
- High performance through Rust
- Better security and safety

## Components

- **ziron-shell**: The main shell interpreter (like `zsh` or `bash`)
- **ziron-core**: Core library with configuration loading, module registry, event system, prompt pipeline
- **ziron-daemon**: Background process that aggregates status information
- **ziron-cli**: Command-line tool for managing Ziron configuration
- **modules**: Official modules (git, sysinfo, etc.)
- **themes**: Collection of pre-built themes

## Quick Start

### Installation

```bash
# Automatic setup
./setup.sh

# Or manually
cargo build --release
```

### Getting Started

```bash
# 1. Build the project
cargo build --release

# 2. Start Ziron Shell
target/release/ziron-shell

# 3. Initialize configuration (inside the shell or externally)
target/release/ziron-cli init

# 4. Add plugins
target/release/ziron-cli plugin add git
target/release/ziron-cli plugin add sysinfo

# 5. Set theme
target/release/ziron-cli theme set default
```

**Ziron is a full-featured shell** – You can use it directly like `zsh` or `bash`!

**Detailed Guide:** See [docs/USAGE.md](docs/USAGE.md)

## Project Structure

```
ziron/
 ├─ ziron-core/      # Core library
 ├─ ziron-cli/       # CLI tool
 ├─ ziron-daemon/    # Background daemon
 ├─ modules/         # Official modules
 │   ├─ git/
 │   └─ sysinfo/
 ├─ themes/          # Themes
 │   ├─ default/
 │   └─ minimal/
 └─ docs/            # Documentation
```

## Development

### Requirements

- Rust stable
- Edition 2021

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Lint

```bash
cargo clippy
cargo fmt
```

## License

MIT

