# Ziron – INSTRUCTION.md

## 1. Überblick

**Ziron** ist ein modulares, schnelles und erweiterbares Shell-Framework, entwickelt in Rust. Ziel ist es, eine moderne Alternative zu bestehenden Frameworks wie Oh‑My‑Zsh, Prezto oder Starship zu bieten – jedoch mit klarer API, hoher Performance und einer sicheren Architektur.

Ziron soll ein einheitliches Ökosystem schaffen, bestehend aus:

* einem **Core-Daemon** (konfigurations- und modulbasiert)
* einem **Plugin-System** (Rust/wasm-basierte Module)
* einem **Theming-Engine** (Prompt‑Rendering + UI‑Module)
* einer **CLI** zur Verwaltung

---

## 2. Projektziele

* Hohe Performance und geringer Overhead
* Stabile, klar definierte Plugin-API
* Theming mit minimaler Latenz
* Portabilität: Linux, macOS, BSD, optional Windows (MSYS2)
* Sicherheit: saubere Sandbox-Aufteilung
* Moderne Syntax für User-Konfiguration (TOML oder RON)

---

## 3. Architektur

### 3.1 Komponentenübersicht

1. **ziron-core**
   Rust-Bibliothek, Basis des Frameworks. Enthält:

   * Config‑Loader
   * Modul-Registry
   * Event-System
   * Prompt-Pipeline
   * IPC-Schnittstelle

2. **ziron-daemon**
   Hintergrundprozess:

   * Aggregiert Statusinformationen
   * Hält Plugin-States im Speicher
   * Bietet schnelles IPC für Prompts

3. **ziron-cli**
   Kommandozeilenwerkzeug:

   * ziron init
   * ziron plugin add/remove
   * ziron theme set
   * ziron config validate

4. **ziron-modules**
   Offizielle Module, z. B.:

   * git Status
   * Systeminfo
   * Timer
   * Exit‑Code‑Render

5. **ziron-themes**
   Sammlung vorgefertigter Themes.

---

## 4. Interne Abläufe

### 4.1 Prompt-Rendering-Pipeline

1. Shell ruft `ziron-prompt` auf.
2. ziron-prompt spricht via IPC den Daemon an.
3. Der Daemon liefert zusammengesetzten Prompt:

   * Kontextdaten
   * Plugin-Daten
   * Theme-Formatierung
4. ziron-prompt rendert ANSI/UTF‑8 Ausgaben.

### 4.2 Plugin-Lifecycle

1. Registrierung über `plugin.toml`.
2. Laden über ziron-daemon.
3. Hooks:

   * `pre_prompt` / `post_prompt`
   * `on_event` (z. B. Verzeichniswechsel)
4. Plugins laufen isoliert (Process oder WASM Sandbox).

---

## 5. Plugin-System

### 5.1 Format

Plugins bestehen aus:

* manifest: `plugin.toml`
* binary (Rust) oder WASM
* optionaler Cache

### 5.2 API (Draft)

* `init(context)`
* `fetch_data(context) -> JSON`
* `shutdown()`

### 5.3 Sicherheitsrichtlinien

* Plugins dürfen nicht direkt auf den Host zugreifen (nur via registrierten Calls).

---

## 6. Theming-System

### 6.1 Grundlagen

Themes definieren:

* Farben
* Separatoren
* Module in Reihenfolge
* Rules (z. B. wann Module aktiv sind)

### 6.2 Beispiel (TOML)

```toml
[theme]
name = "ziron-default"

[[segments]]
module = "git"
color = "blue"

[[segments]]
module = "cwd"
color = "cyan"
```

---

## 7. Konfigurationsstruktur

Benutzerkonfiguration liegt in:

```
~/.config/ziron/config.toml
```

Beispieleinstellungen:

```toml
[shell]
default = "zsh"

[performance]
cache_ttl_ms = 50
```

---

## 8. Projektstruktur (Repository)

```
ziron/
 ├─ ziron-core/
 ├─ ziron-cli/
 ├─ ziron-daemon/
 ├─ modules/
 │   ├─ git/
 │   ├─ sysinfo/
 │   └─ ...
 ├─ themes/
 │   ├─ default/
 │   └─ minimal/
 ├─ docs/
 ├─ tests/
 └─ Cargo.toml
```

---

## 9. Entwicklungsrichtlinien

* Rust stable + Edition 2024
* Strikte Linter-Regeln (clippy + rustfmt)
* Tests für alle Module
* IPC muss deterministisch und sicher bleiben

---

## 10. Roadmap (MVP → Stable)

### Phase 1 – MVP

* Config-Loader
* Einfache Plugin-Registry
* Git-Modul
* Default-Theme
* Grundlegende CLI

### Phase 2 – Erweiterung

* IPC/Daemon
* Cache-System
* 5+ Standardmodule
* Theme-Engine

### Phase 3 – Stable Release

* Plugin-Sandboxing (WASM)
* Async-Event-System
* Plugin-Store
* Dokumentation + Testsuite

---

## 11. Lizenz

MIT
