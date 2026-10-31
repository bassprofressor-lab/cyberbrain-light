---
layout: ../../layouts/Doc.astro
title: "Claude Code: Hooks und MCP"
description: "Cyberbrain Light mit Claude Code verbinden: Hook beim Sitzungsstart, cbl install und der MCP-Server mit recall, recall_id, write und status."
lang: de
slug: "claude-code/"
---

# Claude Code: Hooks und MCP

Light erreicht den Agenten auf zwei Wegen. Der **Hook** gibt die Regeln vor dem ersten Prompt in jede Sitzung. Der **MCP-Server** lässt den Agenten während der Arbeit suchen und Notizen schreiben.

## cbl install

```console
$ cbl install
1 hook(s) in /pfad/zum/projekt/.claude/settings.json pointing at /home/du/.cargo/bin/cbl
```

Es fügt genau einen Eintrag hinzu und lässt den Rest der Datei in Ruhe:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "_managedBy": "cyberbrain-light",
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "/home/du/.cargo/bin/cbl",
            "args": ["hook", "session-start"],
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

An der Markierung `_managedBy` erkennt `cbl install --undo` seine eigenen Einträge; fremde fasst es nicht an. Eingetragen wird der absolute Pfad des Programms, das `install` ausgeführt hat. Wer `cbl` verschiebt oder neu installiert, ruft `cbl install` noch einmal auf. Seit 0.2.0 hat der Hook eine Zeitgrenze von 30 Sekunden (Vorgabe von Claude Code: 600 s), und ein alter PreCompact-Eintrag aus 0.1 wird entfernt.

## Der Hook beim Sitzungsstart

Bei jeder neuen Sitzung ruft Claude Code `cbl hook session-start` auf, und Light antwortet mit den Ringen 0 und 1: den festen Regeln und dem aktuellen Ablauf. Dabei gelten zwei Grenzen:

- der **Token-Deckel** aus `light.toml` (`resident_cap_tokens`, Vorgabe 8.192) und
- die **10.000 Zeichen**, die Claude Code vom Kontext eines Hooks vollständig zeigt. Darüber hinaus zeigt es nur eine Vorschau von 2.000 Zeichen. Light lässt deshalb lieber Notizen weg, als den ganzen Block abschneiden zu lassen, und nennt die fehlenden dort, wo das Modell es liest.

Ring 0 und 1 kurz halten. Sie kosten in jeder Sitzung.

## Der MCP-Server

Für das aktuelle Projekt bei Claude Code anmelden:

```console
$ claude mcp add cyberbrain-light -- cbl mcp
```

oder als Projektdatei `.mcp.json`, die das ganze Team teilt:

```json
{
  "mcpServers": {
    "cyberbrain-light": { "command": "cbl", "args": ["mcp"] }
  }
}
```

Der Server spricht MCP über stdio, beantwortet `initialize` und seit 0.2.0 auch `server/discover` und zustandslose Anfragen aus dem Protokoll 2026-07-28. Er bietet vier Werkzeuge:

| Werkzeug | Argumente | Wirkung |
|---|---|---|
| `recall` | `query`, `n` (Vorgabe 8), `ring` | Stichwortsuche in Ring 2 bis 4 (0 und 1 nur, wenn `ring` sie nennt), Treffer mit Zitat |
| `recall_id` | ein Zitat wie `r2-a91f2c33e1bd` | die ganze Notiz dahinter |
| `write` | `ring` (2, 3 oder 4), `kind`, `name`, `body` | Notiz schreiben oder aktualisieren; die Tags bleiben dabei erhalten |
| `status` | | Notizen je Ring, Index, Budget der Ringe 0 und 1 |

`write` lehnt Ring 0 und 1 ab. Die gehören der Person, die das Projekt führt; das Werkzeug sagt, man solle den Text vorschlagen oder `cbl write --ring 0 …` selbst ausführen.

> Die Argumente der Werkzeuge kommen vom Modell, und das Modell liest alles, was in seinem Kontext steht, auch Text aus Webseiten und Dateien. Dass Ring 0 und 1 für den Agenten gesperrt sind, verhindert, dass eine Prompt Injection die Regeln umschreibt, mit denen jede spätere Sitzung beginnt.
