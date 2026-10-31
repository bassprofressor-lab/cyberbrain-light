---
layout: ../../layouts/Doc.astro
title: "Befehle"
description: "Alle Befehle und Optionen von cbl: init, write, scan, recall, forget, status, doctor, hook, install, mcp."
lang: de
slug: "commands/"
---

# Befehle

```text
cbl [OPTIONEN] <BEFEHL>
```

## Optionen für jeden Befehl

| Option | Wirkung |
|---|---|
| `--store <STORE>` | Der Store, mit dem gearbeitet wird. Ohne Angabe sucht `cbl` vom Arbeitsverzeichnis aufwärts nach `.cyberbrain`, ein Befehl funktioniert also überall im Projekt. Auch über `CYBERBRAIN_STORE`. |
| `--json` | Maschinenlesbare Ausgabe. Jeder Befehl, der etwas ausgibt, kann das. |
| `-q`, `--quiet` | Weniger ausgeben. Fehler landen weiter auf stderr. |
| `-h`, `--help` | Hilfe zum Befehl. |
| `-V`, `--version` | Version ausgeben. |

## init

Legt im aktuellen Verzeichnis einen Store an.

| Option | Vorgabe | |
|---|---|---|
| `--cap <CAP>` | `8192` | Token-Deckel für die Ringe 0 und 1 |

## write

Schreibt eine Notiz. Ein vorhandener Name wird aktualisiert und behält seine Kennung.

```text
cbl write --ring <RING> --kind <ART> --name <NAME> --body <TEXT> [--tags <TAGS>]
```

| Option | |
|---|---|
| `--ring <RING>` | 0 bis 4, siehe [Ringe](../store/#ringe) |
| `--kind <ART>` | `knowledge`, `bug`, `lesson`, `decision`, `reference`, `session` |
| `--name <NAME>` | Kurzname mit Bindestrichen, eindeutig im Store |
| `--body <TEXT>` | der Text in Markdown; `-` liest ihn von der Standardeingabe |
| `--tags <TAGS>` | optional, durch Kommas getrennt |

Eine vorhandene Notiz bleibt in ihrem Ring. Ein `write` mit anderem Ring wird abgelehnt, weil ein Umzug ändert, was die Notiz überstimmen darf; wer das wirklich will, löscht sie vorher mit `cbl forget`. Felder, die Light nicht bearbeitet, auch die des vollen Cyberbrain, bleiben bei einer Änderung erhalten. Tags nicht: Eine Änderung ersetzt sie durch die mitgegebenen `--tags`, also beim Aktualisieren wieder angeben.

## scan

Baut den Index aus dem Notizbaum neu. Nötig nach Änderungen von Hand oder nach dem Holen fremder Notizen.

| Option | |
|---|---|
| `--full` | Index verwerfen und alles neu bauen |

## recall

Durchsucht den Store, und zwar alle Ringe von 0 bis 4; `--ring` beschränkt die Suche auf einen.

```text
cbl recall [ANFRAGE] [--id <ZITAT>] [-n <N>] [--ring <RING>]
```

| Option | Vorgabe | |
|---|---|---|
| `[ANFRAGE]` | | Die Suche. Entfällt bei `--id` |
| `--id <ZITAT>` | | Ein Zitat zur ganzen Notiz aufklappen statt zu suchen |
| `-n`, `--n <N>` | `8` | Anzahl der Treffer |
| `--ring <RING>` | | Nur genau diesen Ring durchsuchen |

## forget

Löscht eine Notiz, ihre Abschnitte und ihre Einträge im Index.

```text
cbl forget <NAME>
```

## status

Was der Store enthält und was der Index weiß: Notizen je Ring, Zustand des Index, das Token-Budget der Ringe 0 und 1.

## doctor

Zeigt, wie viel Platz der Store braucht, und räumt auf.

| Option | |
|---|---|
| `--fix` | Zusätzlich die Datenbank verdichten und Veraltetes neu bauen |

## hook

Beantwortet ein Ereignis im Ablauf des Agenten und liest dazu die Daten von stdin. Das ruft Claude Code auf, nicht du; siehe [Hooks und MCP](../claude-code/).

```text
cbl hook <session-start | pre-compact>
```

`pre-compact` wird weiter angenommen, damit Hooks aus Version 0.1 nicht brechen. `cbl install` trägt ihn aber nicht mehr ein, weil Claude Code die Meldung eines PreCompact-Hooks verwirft.

## install

Trägt die Hooks in die `.claude/settings.json` eines Projekts ein.

| Option | |
|---|---|
| `--project <PROJEKT>` | Projektverzeichnis, ohne Angabe das aktuelle |
| `--undo` | Einträge entfernen statt schreiben |

## mcp

Spricht das Model Context Protocol über stdio. Siehe [der MCP-Server](../claude-code/#der-mcp-server).
