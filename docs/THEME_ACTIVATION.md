# Theme aktivieren – Anleitung

## Standard-Theme "ziron-default"

Das Theme **ziron-default** ist das Standard-Theme von Ziron und wird automatisch verwendet, wenn keine andere Konfiguration vorhanden ist.

## Automatische Aktivierung

Das Default-Theme wird automatisch aktiviert, wenn:

1. **Keine Konfiguration vorhanden ist** – Beim ersten Start wird automatisch `ziron-default` verwendet
2. **Kein Theme in der Config gesetzt ist** – Falls `theme` nicht gesetzt ist, wird `default` verwendet

## Manuelle Aktivierung

### Methode 1: CLI verwenden (empfohlen)

```bash
# Theme setzen
ziron-cli theme set default

# Oder explizit
ziron-cli theme set ziron-default
```

### Methode 2: Konfiguration direkt bearbeiten

Bearbeiten Sie `~/.config/ziron/config.toml`:

```toml
[shell]
default = "zsh"

[performance]
cache_ttl_ms = 50

modules = ["git", "sysinfo"]

theme = "default"
```

Dann Shell neu starten.

### Methode 3: Konfiguration initialisieren

```bash
# Erstellt Standard-Konfiguration mit default-Theme
ziron-cli init
```

## Theme-Status prüfen

```bash
# Aktuelle Konfiguration anzeigen
ziron-cli config validate
```

Ausgabe sollte enthalten:
```
Theme: default
```

## Verfügbare Themes

```bash
# Alle verfügbaren Themes auflisten
ziron-cli theme list
```

Standardmäßig verfügbar:
- **default** (ziron-default) – Vollständiges Theme mit allen wichtigen Informationen
- **minimal** – Minimales Theme nur mit Verzeichnis

## Theme wechseln

```bash
# Zu einem anderen Theme wechseln
ziron-cli theme set minimal

# Zurück zum Default-Theme
ziron-cli theme set default
```

## Troubleshooting

### Theme wird nicht angezeigt

1. **Konfiguration prüfen**:
   ```bash
   ziron-cli config validate
   ```

2. **Theme-Datei prüfen**:
   ```bash
   ls -la themes/default/theme.toml
   ```

3. **Shell neu starten**:
   ```bash
   # Shell beenden und neu starten
   exit
   ziron-shell
   ```

### Theme-Datei nicht gefunden

Stellen Sie sicher, dass die Theme-Datei existiert:
```bash
# Prüfen ob Theme existiert
test -f themes/default/theme.toml && echo "Theme gefunden" || echo "Theme nicht gefunden"
```

Falls nicht vorhanden, erstellen Sie es:
```bash
mkdir -p themes/default
# Kopieren Sie das Theme von der Dokumentation oder dem Repository
```

## Standard-Verhalten

- **Beim ersten Start**: Automatisch `default` Theme
- **Nach `ziron-cli init`**: `default` Theme wird gesetzt
- **Ohne Konfiguration**: `default` Theme wird verwendet
- **Mit leerer Config**: `default` Theme wird verwendet

## Zusammenfassung

Das Theme **ziron-default** ist bereits als Standard konfiguriert. Sie müssen nichts tun, um es zu aktivieren – es wird automatisch verwendet!

Falls Sie es explizit setzen möchten:
```bash
ziron-cli theme set default
```

