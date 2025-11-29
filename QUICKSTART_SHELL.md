# Ziron Shell – Schnellstart

## 🚀 Ziron ist eine Shell!

Ziron ist eine **vollständige Shell** wie `zsh`, `bash` oder `fish`. Sie können sie direkt verwenden:

```bash
# Shell starten
target/release/ziron-shell
```

## Installation

```bash
# Projekt bauen
cargo build --release

# Shell starten
target/release/ziron-shell
```

## Erste Schritte

### 1. Shell starten

```bash
target/release/ziron-shell
```

Sie sehen dann einen Prompt wie:
```
ziron:/root/projects/ziron> 
```

### 2. Befehle ausführen

```bash
# Externe Programme
ls -la
cd /tmp
pwd

# Built-in Commands
cd ~
pwd
echo "Hello Ziron!"
exit
```

### 3. Konfiguration

```bash
# In einer anderen Shell oder Terminal:
target/release/ziron-cli init
target/release/ziron-cli plugin add git
target/release/ziron-cli theme set default
```

### 4. Als Standard-Shell setzen (optional)

```bash
# Ziron zur Liste der erlaubten Shells hinzufügen
sudo sh -c 'echo "/root/projects/ziron/target/release/ziron-shell" >> /etc/shells'

# Als Standard-Shell setzen
chsh -s /root/projects/ziron/target/release/ziron-shell
```

## Features

✅ **Vollständige Shell-Funktionalität**
- Command-Execution
- Built-in Commands (cd, pwd, echo, exit, etc.)
- Piping (`ls | grep test`)

✅ **Modernes Theming**
- Einfache TOML-Konfiguration
- Plugin-System für Module
- Anpassbare Prompts

✅ **Performance**
- Rust-basiert
- Schnell und sicher

## Unterschied zu anderen Shells

| Feature | Ziron | Zsh/Bash | Fish |
|--------|-------|----------|------|
| Shell-Interpreter | ✅ | ✅ | ✅ |
| Modernes Theming | ✅ | ❌ | ⚠️ |
| Einfache Konfiguration | ✅ | ❌ | ⚠️ |
| Plugin-System | ✅ | ⚠️ | ⚠️ |

## Weitere Informationen

- **Vollständige Anleitung**: [docs/SHELL_USAGE.md](docs/SHELL_USAGE.md)
- **Projekt-Dokumentation**: [docs/INSTRUCTION.md](docs/INSTRUCTION.md)

