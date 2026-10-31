---
layout: ../../layouts/Doc.astro
title: "Fünf Minuten"
description: "Store anlegen, Notiz schreiben, wiederfinden und ein Zitat aufklappen, in fünf Minuten."
lang: de
slug: "quickstart/"
---

# Fünf Minuten

```console
$ cbl init
$ cbl write --ring 2 --kind bug --name pg18-moves-pgdata \
    --body 'Das Image postgres:18 legt seine Daten unter /var/lib/postgresql/18/docker ab.'
$ cbl recall 'postgres daten'
$ cbl recall --id r2-867ef2a8cd01     # die ganze Notiz hinter einem Zitat
```

## Was dabei passiert

**`cbl init`** legt im aktuellen Verzeichnis einen Store unter `.cyberbrain/` an: einen Ordner `notes/`, den Index `cyberbrain.db` und `light.toml` mit drei Einstellungen.

**`cbl write`** schreibt eine Markdown-Datei nach `notes/r2/` und nimmt sie in den Index auf. Eine Notiz hat einen **Ring** (wie sehr man ihr traut), eine **Art** (`knowledge`, `bug`, `lesson`, `decision`, `reference`, `session`) und einen **Namen**, einen eindeutigen Kurznamen mit Bindestrichen. Wer denselben Namen noch einmal schreibt, aktualisiert die Notiz, die Kennung bleibt. Mit `--body -` kommt der Text von der Standardeingabe, das ist bei allem Längeren bequemer.

**`cbl recall`** sucht nach den angegebenen Wörtern und gibt Treffer mit einem **Zitat** wie `r2-867ef2a8cd01` aus. Das Zitat bezeichnet einen Abschnitt, nicht die Datei, und bleibt deshalb gültig, wenn die Notiz wächst.

**`cbl recall --id`** klappt ein Zitat zur ganzen Notiz auf.

## Notizen so schreiben, dass man sie findet

Gesucht wird nach Wörtern. Eine Notiz wird über die Wörter gefunden, die in ihr stehen. Schreib sie also so, wie jemand fragen wird, nicht nur so, wie die Antwort lautet. Gut funktioniert eine erste Zeile mit den Symptomen und Halbsätzen dessen, der das Problem gerade hat:

```markdown
*Für: postgres container startet leer · daten weg nach upgrade · PGDATA verschoben*

Das offizielle Image postgres:18 legt das Datenverzeichnis unter /var/lib/postgresql/18/docker ab ...
```

## Weiter

- Hook in Claude Code eintragen: [Hooks und MCP](../claude-code/)
- Wofür Ring 0 und 1 da sind: [Store, Ringe, Einstellungen](../store/)
- Alle Optionen: [Befehle](../commands/)
