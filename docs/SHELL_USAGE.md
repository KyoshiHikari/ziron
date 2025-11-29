# Ziron Shell – Benutzeranleitung

## Was ist Ziron Shell?

Ziron ist eine **vollständige Shell** wie `zsh`, `bash` oder `fish`. Sie können sie direkt verwenden:

```bash
# Ziron Shell starten
ziron-shell

# Oder als Standard-Shell setzen
chsh -s /path/to/ziron-shell
```

## Installation und Start

### 1. Projekt bauen

```bash
cd /root/projects/ziron
cargo build --release
```

### 2. Shell starten

```bash
target/release/ziron-shell
```

### 3. Als Standard-Shell setzen (optional)

```bash
# Ziron zur Liste der erlaubten Shells hinzufügen
echo "/root/projects/ziron/target/release/ziron-shell" >> /etc/shells

# Als Standard-Shell setzen
chsh -s /root/projects/ziron/target/release/ziron-shell
```

## Verwendung

### Grundlegende Befehle

```bash
# Externe Programme ausführen
ls -la
cd /tmp
pwd

# Built-in Commands
cd ~              # Verzeichnis wechseln
pwd               # Aktuelles Verzeichnis anzeigen
echo "Hello"      # Text ausgeben
exit              # Shell beenden
export VAR=value  # Variable setzen
unset VAR         # Variable entfernen
```

### Piping

```bash
# Befehle verketten
ls | grep test
cat file.txt | head -10
```

### History

Die Shell speichert automatisch Ihre Befehls-Historie in `~/.ziron_history`.

## Konfiguration

### Konfigurationsdatei

Die Konfiguration liegt in `~/.config/ziron/config.toml`:

```toml
[shell]
default = "ziron"

[performance]
cache_ttl_ms = 50

modules = ["git", "sysinfo"]

theme = "default"
```

### Konfiguration initialisieren

```bash
target/release/ziron-cli init
```

### Plugins hinzufügen

```bash
target/release/ziron-cli plugin add git
target/release/ziron-cli plugin add sysinfo
```

### Theme ändern

```bash
target/release/ziron-cli theme set default
target/release/ziron-cli theme set minimal
```

## Unterschied zu anderen Shells

| Feature | Ziron | Zsh/Bash | Fish |
|--------|-------|----------|------|
| Shell-Interpreter | ✅ | ✅ | ✅ |
| Modernes Theming | ✅ | ❌ | ⚠️ |
| Plugin-System | ✅ | ⚠️ | ⚠️ |
| Einfache Konfiguration | ✅ | ❌ | ⚠️ |
| Rust-basiert | ✅ | ❌ | ❌ |

## Erweiterungen

### Eigene Module erstellen

Module können in Rust geschrieben werden und erweitern die Shell-Funktionalität:

1. Modul-Verzeichnis erstellen: `modules/my-module/`
2. `plugin.toml` erstellen
3. Rust-Code schreiben
4. Modul registrieren: `ziron-cli plugin add my-module`

### Themes anpassen

Themes werden in TOML definiert und können einfach angepasst werden:

```toml
[theme]
name = "my-theme"

[[segments]]
module = "git"
color = "blue"
separator = " "
```

## Vorteile von Ziron

1. **Moderne Architektur**: Rust-basiert für Performance und Sicherheit
2. **Einfache Konfiguration**: TOML-basiert, keine komplexe Script-Sprache
3. **Erweiterbar**: Plugin-System für Module
4. **Theming**: Vollständiges Theming-System für Prompts und UI
5. **Performance**: Optimiert für Geschwindigkeit

## Roadmap

### Aktuell implementiert (MVP)
- ✅ Grundlegende Command-Execution
- ✅ Built-in Commands (cd, pwd, echo, exit, etc.)
- ✅ History Management
- ✅ Prompt-Rendering
- ✅ Plugin-System (Grundstruktur)
- ✅ Theme-System

### Geplant
- ⏳ Erweiterte Piping und Redirection
- ⏳ Tab-Completion
- ⏳ Scripting-Unterstützung
- ⏳ Erweiterte Built-ins
- ⏳ Plugin-Sandboxing (WASM)
- ⏳ Async-Event-System

