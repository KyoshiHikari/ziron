# Was ist Ziron?

## Ziron ist eine Shell!

**Ziron** ist eine **eigene Shell** – ein vollständiger Shell-Interpreter wie `zsh`, `bash` oder `fish`.

Ziron bietet:
- ✅ Vollständige Shell-Funktionalität (Command-Execution, Piping, etc.)
- ✅ Modernes, erweiterbares Theming-System
- ✅ Einfache Konfiguration und Erweiterbarkeit
- ✅ Plugin-System für Module
- ✅ Bessere Performance durch Rust-Implementierung

## Wie funktioniert es?

```
┌─────────────────────────────────────────┐
│  Ihre Shell (zsh/bash/fish)             │
│  ↓                                      │
│  Shell führt Befehle aus                │
│  ↓                                      │
│  Shell zeigt Prompt an                 │
│  ↓                                      │
│  Ziron rendert den Prompt              │
└─────────────────────────────────────────┘
```

### Beispiel-Workflow:

1. Sie öffnen ein Terminal
2. Ihre Shell (z.B. `zsh`) startet
3. Die Shell ruft `ziron-prompt` auf, um den Prompt zu rendern
4. Ziron zeigt einen schönen, konfigurierbaren Prompt an
5. Sie geben Befehle ein → die **Shell** führt sie aus
6. Nach jedem Befehl zeigt Ziron wieder den Prompt an

## Vergleich

| Tool | Typ | Funktion |
|------|-----|----------|
| `zsh` | Shell | Interpretiert Befehle, führt sie aus |
| `bash` | Shell | Interpretiert Befehle, führt sie aus |
| `fish` | Shell | Interpretiert Befehle, führt sie aus |
| **Ziron** | **Prompt-Framework** | **Gestaltet den Prompt** |
| Starship | Prompt-Framework | Gestaltet den Prompt |
| Oh-My-Zsh | Prompt-Framework | Gestaltet den Prompt |

## Wie verwende ich Ziron?

### 1. Sie brauchen eine Shell
Ziron läuft **auf** einer Shell, z.B.:
- Zsh (`/bin/zsh`)
- Bash (`/bin/bash`)
- Fish (`/usr/bin/fish`)

### 2. Ziron installieren und konfigurieren
```bash
# Projekt bauen
cargo build --release

# Konfiguration initialisieren
target/release/ziron-cli init

# Plugins aktivieren
target/release/ziron-cli plugin add git
```

### 3. In Ihre Shell integrieren
Fügen Sie in `~/.zshrc` (oder `~/.bashrc`) ein:

```bash
# Ziron Prompt Setup
if [ -f /root/projects/ziron/target/release/ziron-daemon ]; then
    pgrep -x ziron-daemon >/dev/null || /root/projects/ziron/target/release/ziron-daemon &
    setopt PROMPT_SUBST  # nur für Zsh
    PROMPT='$(/root/projects/ziron/target/release/ziron-prompt)'
fi
```

### 4. Shell neu starten
```bash
source ~/.zshrc
# oder einfach ein neues Terminal öffnen
```

## Was macht Ziron dann?

- ✅ Zeigt einen schönen, konfigurierbaren Prompt
- ✅ Zeigt Git-Status, Verzeichnis, User-Info, etc.
- ✅ Unterstützt Themes und Plugins
- ✅ Läuft als Hintergrundprozess für Performance

## Was macht Ziron NICHT?

- ❌ Interpretiert keine Befehle
- ❌ Führt keine Programme aus
- ❌ Ist keine interaktive Shell
- ❌ Kann nicht wie `zsh` oder `fish` aufgerufen werden

## Zusammenfassung

**Ziron = Prompt-Framework** (wie Starship, Oh-My-Zsh)
- Läuft **auf** einer Shell
- Gestaltet den **Prompt**
- Führt **keine** Befehle aus

**Shell = Interpreter** (wie zsh, bash, fish)
- Interpretiert Befehle
- Führt Programme aus
- Bietet interaktive Umgebung

Sie können Ziron **nicht** wie `zsh` aufrufen:
```bash
# ❌ Funktioniert NICHT
ziron

# ✅ So funktioniert es:
# 1. Shell starten (zsh, bash, etc.)
zsh

# 2. Ziron rendert dann automatisch den Prompt
```

