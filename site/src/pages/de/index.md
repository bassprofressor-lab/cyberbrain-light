---
layout: ../../layouts/Doc.astro
title: "Cyberbrain Light – Dokumentation"
description: "Zitiertes Gedächtnis mit Vertrauensstufen für KI-Coding-Agenten. Ein Programm mit 5 MB, Stichwortsuche, nichts einzurichten. Installation, Befehle, Hooks, MCP und Store."
lang: de
slug: ""
---

# Cyberbrain Light

<p class="lead">Zitiertes Gedächtnis mit Vertrauensstufen für KI-Coding-Agenten. Ein kleines Programm, Stichwortsuche, nichts einzurichten.</p>

Ein Agent vergisst zwischen zwei Sitzungen alles. Light gibt ihm Notizen, die bleiben: einfaches Markdown im eigenen Repository, eine Stichwortsuche darüber, und jeder Treffer trägt ein Zitat, das auf genau den Abschnitt zurückführt, aus dem er stammt.

<div class="btns">
  <a class="btn primary" href="install/">Installation</a>
  <a class="btn ghost" href="quickstart/">Fünf Minuten</a>
  <a class="btn ghost" href="https://github.com/bassprofressor-lab/cyberbrain-light">Quelltext auf GitHub</a>
</div>

```console
$ cbl recall 'postgres data directory'
1. r2-867ef2a8cd01  r2  pg18-moves-pgdata  (100% of top)
     # PostgreSQL 18 moves PGDATA
     The official `postgres:18` image puts the data directory at
     `/var/lib/postgresql/18/docker` instead of `/var/lib/postgresql/data` ...
caveat: search is lexical: it matches the words in the text, not the meaning.
```

Der Hinweis am Ende ist keine Entschuldigung, sondern die Abmachung. Light sucht nach Wörtern und sagt das bei jedem Ergebnis, damit niemand auf einer Suche aufbaut, die nicht stattgefunden hat.

<div class="cards">
  <div class="card"><div class="k">Programm</div><div class="v">5,0 MB</div><div class="s">Linux und Windows</div></div>
  <div class="card"><div class="k">eine Suche</div><div class="v">3 ms</div><div class="s">8 MB Speicher</div></div>
  <div class="card"><div class="k">Hook beim Sitzungsstart</div><div class="v">2 ms</div><div class="s">6 MB Speicher</div></div>
  <div class="card"><div class="k">Index neu bauen</div><div class="v">0,7 s</div><div class="s">1.007 Notizen, 4.034 Abschnitte</div></div>
</div>

<sub>Gemessen am 06.09.2026 an einem echten Store mit 1.007 Notizen auf einem Server ohne Grafikkarte (AMD EPYC-Milan, 12 vCPU). Die Zeiten enthalten den Prozessstart, denn den zahlt ein Hook tatsächlich.</sub>

## Vier Zugänge

| | |
|---|---|
| [`cbl <befehl>`](commands/) | `init`, `write`, `scan`, `recall`, `forget`, `status`, `doctor`, alle mit `--json` |
| [`cbl hook <ereignis>`](claude-code/#der-hook-beim-sitzungsstart) | `session-start` gibt Ring 0 und 1 in jede neue Claude-Code-Sitzung |
| [`cbl mcp`](claude-code/#der-mcp-server) | Model Context Protocol über stdio: `recall`, `recall_id`, `write`, `status` |
| [`cbl install`](claude-code/#cbl-install) | trägt den Hook in die `.claude/settings.json` eines Projekts ein und wieder aus |

## Was es nicht kann

Light ist ebenso durch das bestimmt, was fehlt. Es gibt **keine Bedeutungssuche** (kein Modell, keine Einbettung), **keine Widerspruchsprüfung** und **kein Compliance-Teilsystem** (Egress-Register, Audit-Kette, PII-Prüfung, Löschnachweis, Pflichtenkatalog). Dafür gibt es das [volle Cyberbrain](light-or-full/).

Light hat außerdem keine Telemetrie, keinen Hintergrundabgleich und überhaupt keinen Netzwerkzugriff. Es baut keine Verbindung nach außen auf, nicht als Einstellung, sondern weil kein Code darin steht, der es könnte.
