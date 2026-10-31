---
layout: ../layouts/Doc.astro
title: "Cyberbrain Light documentation"
description: "Cited, trust-tiered memory for AI coding agents. One 5 MB binary, keyword search, nothing to set up. Install, commands, hooks, MCP and store layout."
slug: ""
---

# Cyberbrain Light

<p class="lead">Cited, trust-tiered memory for AI coding agents. One small binary, keyword search, nothing to set up.</p>

Your agent forgets everything between sessions. Light gives it notes that survive: plain Markdown in your repository, keyword search over them, and every hit carrying a citation that resolves back to the exact block it came from.

<div class="btns">
  <a class="btn primary" href="install/">Install</a>
  <a class="btn ghost" href="quickstart/">Five minutes</a>
  <a class="btn ghost" href="https://github.com/bassprofressor-lab/cyberbrain-light">Source on GitHub</a>
</div>

```console
$ cbl recall 'postgres data directory'
1. r2-867ef2a8cd01  r2  pg18-moves-pgdata  (100% of top)
     # PostgreSQL 18 moves PGDATA
     The official `postgres:18` image puts the data directory at
     `/var/lib/postgresql/18/docker` instead of `/var/lib/postgresql/data` ...
caveat: search is lexical: it matches the words in the text, not the meaning.
```

The caveat is not an apology, it is the contract. Light searches for words, and it says so on every result, so nobody builds on a search that did not happen.

<div class="cards">
  <div class="card"><div class="k">binary</div><div class="v">5.0 MB</div><div class="s">Linux and Windows</div></div>
  <div class="card"><div class="k">a search</div><div class="v">3 ms</div><div class="s">8 MB of memory</div></div>
  <div class="card"><div class="k">session-start hook</div><div class="v">2 ms</div><div class="s">6 MB of memory</div></div>
  <div class="card"><div class="k">full reindex</div><div class="v">0.7 s</div><div class="s">1,007 notes, 4,034 blocks</div></div>
</div>

<sub>Measured on 2026-09-06 against a real store of 1,007 notes, on a server without a GPU (AMD EPYC-Milan, 12 vCPU). Times include process start, because that is what a hook actually pays.</sub>

## Four ways in

| | |
|---|---|
| [`cbl <command>`](commands/) | `init`, `write`, `scan`, `recall`, `forget`, `status`, `doctor`, all with `--json` |
| [`cbl hook <event>`](claude-code/#the-session-start-hook) | `session-start` puts rings 0 and 1 into every new Claude Code session |
| [`cbl mcp`](claude-code/#the-mcp-server) | Model Context Protocol over stdio: `recall`, `recall_id`, `write`, `status` |
| [`cbl install`](claude-code/#cbl-install) | writes the hook into a project's `.claude/settings.json`, and takes it out again |

## What it does not do

Light is defined as much by what is not in it. There is **no semantic search** (no model, no embedding), **no contradiction check**, and **no compliance subsystem** (egress register, audit chain, PII gate, erasure record, obligations). Those are the reason the [full Cyberbrain](light-or-full/) exists.

Light also has no telemetry, no background sync and no network access of any kind. It makes no outbound connections, not as a setting, but because there is no code in it that could.
