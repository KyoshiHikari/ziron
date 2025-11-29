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
# Automatisches Setup
./setup.sh

# Oder manuell
cargo build --release
```

### Erste Schritte

```bash
# 1. Projekt bauen
cargo build --release

# 2. Ziron Shell starten
target/release/ziron-shell

# 3. Konfiguration initialisieren (innerhalb der Shell oder extern)
target/release/ziron-cli init

# 4. Plugins hinzufügen
target/release/ziron-cli plugin add git
target/release/ziron-cli plugin add sysinfo

# 5. Theme setzen
target/release/ziron-cli theme set default
```

**Ziron ist eine vollständige Shell** – Sie können sie direkt verwenden wie `zsh` oder `bash`!

**Ausführliche Anleitung:** Siehe [docs/USAGE.md](docs/USAGE.md)

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

