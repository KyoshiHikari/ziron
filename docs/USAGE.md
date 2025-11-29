# Ziron – Benutzeranleitung

## Installation

### 1. Projekt bauen

```bash
cd /root/projects/ziron
cargo build --release
```

Die Binaries werden in `target/release/` erstellt:
- `ziron-cli` – Verwaltungstool
- `ziron-daemon` – Hintergrundprozess
- `ziron-prompt` – Prompt-Binary für die Shell

### 2. Binaries in den PATH aufnehmen (optional)

```bash
# Temporär für diese Session
export PATH="$PATH:/root/projects/ziron/target/release"

# Oder dauerhaft in ~/.zshrc oder ~/.bashrc hinzufügen:
echo 'export PATH="$PATH:/root/projects/ziron/target/release"' >> ~/.zshrc
source ~/.zshrc
```

Alternativ können Sie Symlinks erstellen:

```bash
sudo ln -s /root/projects/ziron/target/release/ziron-cli /usr/local/bin/ziron
sudo ln -s /root/projects/ziron/target/release/ziron-daemon /usr/local/bin/ziron-daemon
sudo ln -s /root/projects/ziron/target/release/ziron-prompt /usr/local/bin/ziron-prompt
```

## Erste Schritte

### 1. Konfiguration initialisieren

```bash
ziron init
```

Dies erstellt die Standard-Konfiguration in `~/.config/ziron/config.toml`.

### 2. Plugins hinzufügen

```bash
# Git-Plugin hinzufügen
ziron plugin add git

# Systeminfo-Plugin hinzufügen
ziron plugin add sysinfo

# Alle installierten Plugins anzeigen
ziron plugin list
```

### 3. Theme auswählen

```bash
# Verfügbare Themes anzeigen
ziron theme list

# Theme setzen
ziron theme set default
# oder
ziron theme set minimal
```

### 4. Konfiguration validieren

```bash
ziron config validate
```

## Shell-Integration

### Für Zsh

Fügen Sie folgende Zeilen zu Ihrer `~/.zshrc` hinzu:

```bash
# Ziron Prompt
eval "$(ziron-daemon &)"
export PROMPT='$(ziron-prompt)'
```

Oder für eine bessere Integration:

```bash
# Ziron Prompt Setup
if command -v ziron-daemon >/dev/null 2>&1; then
    # Starte Daemon im Hintergrund (falls nicht bereits läuft)
    pgrep -x ziron-daemon >/dev/null || ziron-daemon &
    
    # Setze Prompt-Funktion
    function ziron_prompt() {
        echo -n "$(ziron-prompt)"
    }
    
    # Verwende Ziron als Prompt
    setopt PROMPT_SUBST
    PROMPT='$(ziron_prompt)'
fi
```

### Für Bash

Fügen Sie folgende Zeilen zu Ihrer `~/.bashrc` hinzu:

```bash
# Ziron Prompt Setup
if command -v ziron-daemon >/dev/null 2>&1; then
    # Starte Daemon im Hintergrund (falls nicht bereits läuft)
    pgrep -x ziron-daemon >/dev/null || ziron-daemon &
    
    # Setze Prompt-Funktion
    function ziron_prompt() {
        echo -n "$(ziron-prompt)"
    }
    
    # Verwende Ziron als Prompt
    PS1='$(ziron_prompt)'
fi
```

## Daemon-Verwaltung

### Daemon starten

```bash
# Im Vordergrund (für Debugging)
ziron-daemon

# Im Hintergrund
ziron-daemon &
```

### Daemon stoppen

```bash
pkill ziron-daemon
```

### Daemon-Status prüfen

```bash
pgrep -x ziron-daemon && echo "Daemon läuft" || echo "Daemon läuft nicht"
```

## Konfiguration anpassen

Die Konfigurationsdatei liegt unter `~/.config/ziron/config.toml`.

Beispiel-Konfiguration:

```toml
[shell]
default = "zsh"

[performance]
cache_ttl_ms = 50

modules = ["git", "sysinfo"]

theme = "default"
```

### Verfügbare Einstellungen

- `shell.default`: Standard-Shell (zsh, bash, etc.)
- `performance.cache_ttl_ms`: Cache-Zeit in Millisekunden
- `modules`: Liste der aktivierten Module
- `theme`: Name des aktiven Themes

## Module

### Verfügbare Module

- **git**: Zeigt Git-Branch und Status an
- **sysinfo**: Zeigt User@Hostname an

### Module manuell hinzufügen/entfernen

```bash
# Modul hinzufügen
ziron plugin add <modulname>

# Modul entfernen
ziron plugin remove <modulname>

# Alle Module auflisten
ziron plugin list
```

## Themes

### Verfügbare Themes

- **default**: Standard-Theme mit sysinfo, git und cwd
- **minimal**: Minimales Theme nur mit cwd

### Theme wechseln

```bash
ziron theme set <themename>
```

## Troubleshooting

### Daemon startet nicht

1. Prüfen Sie, ob der Socket-Pfad existiert:
   ```bash
   ls -la ~/.config/ziron/ziron.sock
   ```

2. Prüfen Sie die Logs:
   ```bash
   RUST_LOG=debug ziron-daemon
   ```

### Prompt wird nicht angezeigt

1. Prüfen Sie, ob der Daemon läuft:
   ```bash
   pgrep -x ziron-daemon
   ```

2. Testen Sie das Prompt-Binary direkt:
   ```bash
   ziron-prompt
   ```

3. Prüfen Sie die Shell-Konfiguration:
   ```bash
   echo $PROMPT  # für Zsh
   echo $PS1     # für Bash
   ```

### Konfiguration wird nicht geladen

1. Validieren Sie die Konfiguration:
   ```bash
   ziron config validate
   ```

2. Prüfen Sie die Konfigurationsdatei:
   ```bash
   cat ~/.config/ziron/config.toml
   ```

## Entwicklung

### Projekt neu bauen

```bash
cargo build --release
```

### Tests ausführen

```bash
cargo test
```

### Code formatieren

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

