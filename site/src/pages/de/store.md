---
layout: ../../layouts/Doc.astro
title: "Store, Ringe, Einstellungen"
description: "Wie Cyberbrain Light Notizen ablegt: das Verzeichnis .cyberbrain, die fünf Vertrauensringe, Zitate und die drei Einstellungen in light.toml."
lang: de
slug: "store/"
---

# Store, Ringe, Einstellungen

## Der Store

`cbl init` legt im Projekt `.cyberbrain/` an:

```text
.cyberbrain/
├── notes/          # deine Notizen, einfaches Markdown, ein Ordner je Ring
│   ├── r0/
│   ├── r1/
│   └── r2/ …
├── cyberbrain.db   # der Suchindex (SQLite, FTS5); jederzeit neu mit `cbl scan --full`
└── light.toml      # drei Einstellungen
```

Maßgeblich sind die Notizen, der Index wird aus ihnen gebaut. Soll das Team das Gedächtnis teilen, kommt `notes/` mit ins Repository, `cyberbrain.db` nicht.

Jeder Befehl sucht den Store vom Arbeitsverzeichnis aufwärts und funktioniert deshalb überall im Projekt. Mit `--store` oder `CYBERBRAIN_STORE` geht es auch woanders hin.

## Ringe

Jede Notiz liegt in einem nummerierten Ring, und die Nummer sagt, wie sehr man ihr traut.

| Ring | was hineingehört | in jeder Sitzung |
|---|---|---|
| r0 | feste Regeln des Betreibers | ja |
| r1 | Arbeitsablauf, Übergabestand | ja |
| r2 | Projektwissen | bei Suche |
| r3 | Sitzungsprotokolle | bei Suche |
| r4 | Importiertes oder Ungeprüftes | bei Suche |

Kleinere Nummer heißt mehr Vertrauen. Ring 0 und 1 kommen unter einem [Token-Deckel](#lighttoml) in jede Sitzung mit, damit der Agent die Regeln von Anfang an kennt, statt sie neu zu entdecken. Sie gehören dir: Das MCP-Werkzeug `write` lehnt sie ab.

Eine Notiz bleibt in ihrem Ring. Ein Umzug ist eine Entscheidung und keine Nebenwirkung, deshalb wird ein `write` mit anderem Ring abgelehnt; erst `cbl forget`, dann neu schreiben.

## Zitate

Jeder Treffer trägt ein Zitat wie `r2-867ef2a8cd01`: Ring und Kennung eines Abschnitts. Es bezeichnet einen Abschnitt einer Notiz, nicht die Datei, und überlebt deshalb, wenn die Notiz wächst. `cbl recall --id <zitat>` gibt die ganze Notiz zurück.

## light.toml

Drei Einstellungen. Wer die Datei löscht, bekommt die Vorgaben zurück.

```toml
# Rings 0 and 1 are injected into every session; this caps what that costs.
resident_cap_tokens = 8192

# Hits returned by `cbl recall`.
n = 8

# Candidates BM25 weighs before the list is cut to `n`.
k_lex = 50
```

Unabhängig vom Token-Deckel bleibt der Hook beim Sitzungsstart unter den 10.000 Zeichen, die Claude Code vollständig zeigt; siehe [Hooks](../claude-code/#der-hook-beim-sitzungsstart).

## Gemeinsam mit dem vollen Cyberbrain

Der Aufbau des Stores ist der von Cyberbrain: gleiches Verzeichnis, gleiches Markdown, gleiche Ringe, gleiche Zitate. Wer mit Light anfängt und später [Cyberbrain](../light-or-full/) installiert, holt mit einem `cyberbrain scan` alle bisherigen Notizen herein. Light hält seine Einstellungen in `light.toml` und fasst die `cyberbrain.toml` des vollen Produkts nicht an.
