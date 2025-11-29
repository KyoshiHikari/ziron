# Ziron – Schnellstart

## 🚀 Installation

```bash
# Projekt bauen
cargo build --release

# Oder automatisches Setup verwenden
./setup.sh
```

## 📋 Erste Schritte

### 1. Konfiguration initialisieren

```bash
target/release/ziron-cli init
```

### 2. Plugins aktivieren

```bash
target/release/ziron-cli plugin add git
target/release/ziron-cli plugin add sysinfo
target/release/ziron-cli plugin list
```

### 3. Theme auswählen

```bash
target/release/ziron-cli theme set default
```

### 4. Daemon starten

```bash
# Im Hintergrund starten
target/release/ziron-daemon &
```

### 5. Shell-Integration

**Für Zsh** (`~/.zshrc`):
```bash
# Ziron Prompt Setup
if [ -f /root/projects/ziron/target/release/ziron-daemon ]; then
    pgrep -x ziron-daemon >/dev/null || /root/projects/ziron/target/release/ziron-daemon &
    setopt PROMPT_SUBST
    PROMPT='$(/root/projects/ziron/target/release/ziron-prompt)'
fi
```

**Für Bash** (`~/.bashrc`):
```bash
# Ziron Prompt Setup
if [ -f /root/projects/ziron/target/release/ziron-daemon ]; then
    pgrep -x ziron-daemon >/dev/null || /root/projects/ziron/target/release/ziron-daemon &
    PS1='$(/root/projects/ziron/target/release/ziron-prompt)'
fi
```

Dann Shell neu laden:
```bash
source ~/.zshrc  # oder source ~/.bashrc
```

## 🎯 Wichtige Befehle

```bash
# Konfiguration validieren
target/release/ziron-cli config validate

# Plugins verwalten
target/release/ziron-cli plugin add <name>
target/release/ziron-cli plugin remove <name>
target/release/ziron-cli plugin list

# Themes verwalten
target/release/ziron-cli theme set <name>
target/release/ziron-cli theme list

# Daemon-Status prüfen
pgrep -x ziron-daemon && echo "Läuft" || echo "Läuft nicht"
```

## 📍 Binaries im PATH (optional)

Für einfachere Nutzung können Sie die Binaries in den PATH aufnehmen:

```bash
# Temporär
export PATH="$PATH:/root/projects/ziron/target/release"

# Dauerhaft (in ~/.zshrc oder ~/.bashrc)
echo 'export PATH="$PATH:/root/projects/ziron/target/release"' >> ~/.zshrc
```

Dann können Sie einfach `ziron-cli`, `ziron-daemon` und `ziron-prompt` verwenden.

## 🔧 Konfiguration

Die Konfiguration liegt unter: `~/.config/ziron/config.toml`

Beispiel:
```toml
[shell]
default = "zsh"

[performance]
cache_ttl_ms = 50

modules = ["git", "sysinfo"]

theme = "default"
```

## 📚 Weitere Informationen

- **Vollständige Anleitung**: [docs/USAGE.md](docs/USAGE.md)
- **Projekt-Dokumentation**: Siehe [docs/](docs/) für alle Dokumentationen

