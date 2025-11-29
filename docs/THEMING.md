# Ziron Themes – Anleitung zur Theme-Erstellung

## Überblick

Themes in Ziron definieren, wie der Prompt dargestellt wird. Sie bestimmen:
- Welche Module angezeigt werden
- In welcher Reihenfolge sie erscheinen
- Welche Farben verwendet werden
- Welche Separatoren zwischen Segmenten stehen
- Unter welchen Bedingungen Module angezeigt werden

Themes werden in **TOML-Format** geschrieben und sind einfach zu erstellen und anzupassen.

## Theme-Struktur

Ein Theme besteht aus:
- **Name**: Identifikation des Themes
- **Segments**: Liste von Modulen, die im Prompt angezeigt werden

### Grundstruktur

```toml
[theme]
name = "mein-theme"

[[segments]]
module = "modulname"
color = "farbe"
separator = "trenner"
```

## Theme-Verzeichnis erstellen

1. **Verzeichnis anlegen**
   ```bash
   mkdir -p themes/mein-theme
   ```

2. **Theme-Datei erstellen**
   ```bash
   touch themes/mein-theme/theme.toml
   ```

3. **Grundstruktur hinzufügen**
   ```toml
   [theme]
   name = "mein-theme"
   
   [[segments]]
   module = "sysinfo"
   color = "green"
   ```

## Verfügbare Module

Ziron unterstützt verschiedene Module, die im Prompt angezeigt werden können:

### Standard-Module

- **`sysinfo`**: Zeigt User@Hostname an
- **`git`**: Zeigt Git-Branch und Status an
- **`cwd`**: Zeigt aktuelles Arbeitsverzeichnis an

### Module hinzufügen

Module werden über Plugins hinzugefügt. Verfügbare Module finden Sie mit:
```bash
ziron-cli plugin list
```

## Farben

Themes unterstützen folgende Farben:

### Standard-Farben

- `black` - Schwarz
- `red` - Rot
- `green` - Grün
- `yellow` - Gelb
- `blue` - Blau
- `magenta` - Magenta
- `cyan` - Cyan
- `white` - Weiß

### Verwendung

```toml
[[segments]]
module = "git"
color = "blue"  # Blauer Text
```

**Hinweis**: Wenn keine Farbe angegeben wird, wird der Standard-Terminal-Farbwert verwendet.

## Separatoren

Separatoren werden zwischen Segmenten angezeigt. Sie können beliebigen Text enthalten:

### Beispiele

```toml
# Einfacher Leerzeichen-Separator
separator = " "

# Pfeil-Separator
separator = " → "

# Pipe-Separator
separator = " | "

# Kein Separator (nur für letztes Segment)
# separator weglassen
```

### Praktisches Beispiel

```toml
[[segments]]
module = "sysinfo"
color = "green"
separator = " "  # Leerzeichen nach sysinfo

[[segments]]
module = "git"
color = "blue"
separator = " "  # Leerzeichen nach git

[[segments]]
module = "cwd"
color = "cyan"
# Kein Separator - ist das letzte Segment
```

## Rules (Bedingungen)

Rules ermöglichen es, Module nur unter bestimmten Bedingungen anzuzeigen:

```toml
[[segments]]
module = "git"
color = "blue"
rules = [
    { condition = "is_git_repo", value = true }
]
```

**Hinweis**: Rules werden aktuell noch nicht vollständig implementiert, sind aber für zukünftige Erweiterungen vorgesehen.

## Vollständiges Beispiel

### Beispiel 1: Standard-Theme

```toml
[theme]
name = "ziron-default"

[[segments]]
module = "sysinfo"
color = "green"
separator = " "

[[segments]]
module = "git"
color = "blue"
separator = " "

[[segments]]
module = "cwd"
color = "cyan"
```

**Ausgabe**: `user@hostname [git-branch] /current/directory`

### Beispiel 2: Minimal-Theme

```toml
[theme]
name = "ziron-minimal"

[[segments]]
module = "cwd"
color = "cyan"
```

**Ausgabe**: `/current/directory`

### Beispiel 3: Farbenfrohes Theme

```toml
[theme]
name = "colorful"

[[segments]]
module = "sysinfo"
color = "magenta"
separator = " → "

[[segments]]
module = "git"
color = "yellow"
separator = " → "

[[segments]]
module = "cwd"
color = "green"
separator = " ❯ "
```

**Ausgabe**: `user@hostname → [git-branch] → /current/directory ❯`

### Beispiel 4: Kompakt mit Pipe-Separatoren

```toml
[theme]
name = "compact"

[[segments]]
module = "sysinfo"
color = "blue"
separator = " | "

[[segments]]
module = "git"
color = "cyan"
separator = " | "

[[segments]]
module = "cwd"
color = "yellow"
```

**Ausgabe**: `user@hostname | [git-branch] | /current/directory`

## Theme aktivieren

### 1. Theme erstellen

Erstellen Sie Ihr Theme in `themes/mein-theme/theme.toml`

### 2. Theme setzen

```bash
ziron-cli theme set mein-theme
```

### 3. Shell neu starten

Starten Sie die Ziron Shell neu, um das neue Theme zu sehen:

```bash
ziron-shell
```

## Theme-Validierung

### Syntax prüfen

```bash
# TOML-Syntax prüfen (mit toml-cli oder ähnlichem Tool)
toml-cli themes/mein-theme/theme.toml
```

### In Shell testen

1. Theme setzen: `ziron-cli theme set mein-theme`
2. Shell starten: `ziron-shell`
3. Prompt überprüfen

## Best Practices

### 1. Lesbarkeit

- Verwenden Sie ausreichend Abstand zwischen Segmenten
- Wählen Sie kontrastreiche Farben für bessere Lesbarkeit
- Vermeiden Sie zu viele Segmente (max. 3-5)

### 2. Performance

- Minimieren Sie die Anzahl der Module
- Verwenden Sie einfache Separatoren
- Vermeiden Sie komplexe Rules (wenn verfügbar)

### 3. Konsistenz

- Verwenden Sie konsistente Farben für ähnliche Informationen
- Halten Sie Separator-Stil einheitlich
- Dokumentieren Sie Ihr Theme

### 4. Portabilität

- Verwenden Sie Standard-Farben
- Testen Sie auf verschiedenen Terminals
- Berücksichtigen Sie Terminal-Farbunterstützung

## Erweiterte Themen

### Eigene Module erstellen

Um eigene Module für Themes zu erstellen, siehe die [Plugin-Dokumentation](INSTRUCTION.md#5-plugin-system).

### Theme teilen

1. Theme-Verzeichnis kopieren
2. In Repository hochladen
3. Andere können es mit `ziron-cli theme add <url>` installieren

**Hinweis**: Theme-Sharing wird in zukünftigen Versionen unterstützt.

## Troubleshooting

### Theme wird nicht geladen

1. **Pfad prüfen**: Theme muss in `themes/<name>/theme.toml` liegen
2. **Syntax prüfen**: TOML-Syntax muss korrekt sein
3. **Name prüfen**: Theme-Name muss mit Verzeichnisname übereinstimmen

### Farben werden nicht angezeigt

1. **Terminal-Unterstützung**: Prüfen Sie, ob Ihr Terminal ANSI-Farben unterstützt
2. **Farbname prüfen**: Verwenden Sie nur unterstützte Farbnamen
3. **Escape-Sequenzen**: Manche Terminals benötigen spezielle Konfiguration

### Module werden nicht angezeigt

1. **Plugin installiert**: Stellen Sie sicher, dass das Modul-Plugin installiert ist
2. **Modulname prüfen**: Modulname muss exakt mit Plugin-Namen übereinstimmen
3. **Daten verfügbar**: Manche Module zeigen nur Daten an, wenn verfügbar (z.B. git nur in Git-Repositories)

## Referenz

### Vollständige Theme-Struktur

```toml
[theme]
name = "theme-name"

# Segment 1
[[segments]]
module = "modulname"        # Erforderlich: Name des Moduls
color = "farbe"            # Optional: Farbe (black, red, green, yellow, blue, magenta, cyan, white)
separator = "trenner"      # Optional: Separator nach diesem Segment

# Segment 2
[[segments]]
module = "anderes-modul"
color = "blue"
separator = " "

# Weitere Segmente...
```

### Verfügbare Module (Standard)

| Modul | Beschreibung | Beispiel-Ausgabe |
|-------|-------------|------------------|
| `sysinfo` | User@Hostname | `user@hostname` |
| `git` | Git-Branch und Status | `main ✓` oder `feature-branch ✗` |
| `cwd` | Aktuelles Verzeichnis | `/home/user/projects` |

## Beispiele zum Kopieren

### Minimalistisch

```toml
[theme]
name = "minimal"

[[segments]]
module = "cwd"
color = "cyan"
```

### Standard

```toml
[theme]
name = "standard"

[[segments]]
module = "sysinfo"
color = "green"
separator = " "

[[segments]]
module = "git"
color = "blue"
separator = " "

[[segments]]
module = "cwd"
color = "cyan"
```

### Modern

```toml
[theme]
name = "modern"

[[segments]]
module = "sysinfo"
color = "magenta"
separator = " ❯ "

[[segments]]
module = "git"
color = "yellow"
separator = " ❯ "

[[segments]]
module = "cwd"
color = "green"
```

## Weitere Ressourcen

- **Projekt-Dokumentation**: [docs/INSTRUCTION.md](INSTRUCTION.md)
- **Shell-Anleitung**: [docs/SHELL_USAGE.md](SHELL_USAGE.md)
- **Plugin-System**: Siehe Plugin-Dokumentation für eigene Module

## Feedback und Beiträge

Wenn Sie ein Theme erstellt haben, das Sie teilen möchten, oder Verbesserungsvorschläge haben, können Sie diese gerne beitragen!

