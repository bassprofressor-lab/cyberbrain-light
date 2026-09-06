<div align="center">

# Cyberbrain Light

**Cited, trust-tiered memory for AI coding agents. One small binary, keyword search, nothing to set up.**

[![Licence: FSL-1.1-ALv2](https://img.shields.io/badge/licence-FSL--1.1--ALv2-blue)](LICENSE.md)
[![Rust 1.98+](https://img.shields.io/badge/rust-1.98%2B-b7410e)](rust-toolchain.toml)
[![Linux and Windows](https://img.shields.io/badge/runs%20on-Linux%20%C2%B7%20Windows-333)](#install)
[![No telemetry](https://img.shields.io/badge/telemetry-does%20not%20exist-2ea44f)](#what-it-does-not-do)

**English** · [Deutsch](README.de.md)

[Install](#install) · [Five minutes](#five-minutes) · [What it costs](#what-it-costs) ·
[Light or full](#cyberbrain-or-cyberbrain-light) · [Licence](#licence)

</div>

Your agent forgets everything between sessions. Light gives it notes that survive: plain
Markdown in your repository, keyword search over them, and every hit carrying a citation that
resolves back to the exact block it came from. Five megabytes, no model to place, no service
to run, no configuration to get right first.

```console
$ cbl recall 'postgres data directory'
1. r2-867ef2a8cd01  r2  pg18-moves-pgdata  (100% of top)
     # PostgreSQL 18 moves PGDATA
     The official `postgres:18` image puts the data directory at
     `/var/lib/postgresql/18/docker` instead of `/var/lib/postgresql/data` ...
caveat: search is lexical: it matches the words in the text, not the meaning.
```

The caveat is not an apology, it is the contract. Light searches for words. It says so on
every result, so nobody builds on a search that did not happen.

## Install

```console
$ cargo install --git https://github.com/bassprofressor-lab/cyberbrain-light
$ cbl init
$ cbl install          # writes the two hooks into .claude/settings.json
```

Not on crates.io yet, so the install goes through git for now.

Linux and Windows. Nothing is needed at runtime: no system SQLite, no OpenSSL, no model
server, no node. `cbl install --undo` takes the hooks out again, and it never touches an
entry it did not write.

## Five minutes

```console
$ cbl init
$ cbl write --ring 2 --kind bug --name pg18-moves-pgdata --body 'what you learned'
$ cbl recall 'what you learned'
$ cbl recall --id r2-867ef2a8cd01     # the whole note behind a citation
```

Four ways in, all from the same binary:

| | |
|---|---|
| `cbl <command>` | `init`, `write`, `scan`, `recall`, `forget`, `status`, `doctor` — `--json` on all of them |
| `cbl hook <event>` | `session-start` injects rings 0 and 1; `pre-compact` says what is about to be forgotten |
| `cbl mcp` | Model Context Protocol over stdio: `recall`, `recall_id`, `write`, `status` |
| `cbl install` | puts the hooks into a project's `.claude/settings.json`, and takes them out |

### Rings

Notes live in numbered rings, and the number is a claim about trust.

| ring | what belongs there | injected every session |
|---|---|---|
| r0 | operator invariants, hard rules | yes |
| r1 | operating protocol, handoff state | yes |
| r2 | project knowledge | on recall |
| r3 | session records | on recall |
| r4 | imported or unverified material | on recall |

Rings 0 and 1 ride along in every session under a token cap, so the agent starts knowing the
rules instead of rediscovering them. They belong to you: the MCP `write` tool refuses them,
and says to propose the text instead.

## What it costs

| | |
|---|---|
| the binary | 5.0 MB |
| a search | 3 ms, 8 MB of memory |
| the session-start hook | 2 ms, 6 MB |
| rebuilding the whole index | 0.7 s |
| on disk | your notes, plus 18 MB of index |

<sub>Measured on 2026-09-06 against a real store of 1,007 notes and 4,034 blocks, on a server
with no GPU (AMD EPYC-Milan, 12 vCPU). Times include process start, because that is what a
hook actually pays.</sub>

## What it does not do

Light is defined as much by what is not in it. None of the following exists here, and none of
it is coming: **semantic search** (there is no model and no embedding to place), a
**contradiction check** (no inference endpoint, so no second opinion on a hit), an **egress
register, audit chain, PII gate, erasure record or obligation catalogue** (the compliance
subsystem is the other product's reason to exist), and a **web page**.

What it also does not do: telemetry, background sync, network access of any kind. Light makes
no outbound connections at all — not as a setting, but because there is no code in it that
could.

## Cyberbrain or Cyberbrain Light

|  | Light | [Cyberbrain](https://github.com/bassprofressor-lab/cyberbrain) |
|---|---|---|
| search | keyword (BM25) | keyword **and** meaning, fused |
| model | none | a static embedding artefact you place yourself |
| memory while searching | 8 MB | 1.55 GB |
| contradiction check | — | against a local inference endpoint |
| GDPR and EU AI Act artefacts | — | erasure, subject access, audit chain, PII gate, obligations |
| web page | — | built into the binary, English and German |
| binary | 5.0 MB | 13.6 MB |

**The store is the same.** Same directory, same Markdown, same rings, same citations. Start
with Light and install Cyberbrain later, and one `cyberbrain scan` picks up every note you
already wrote. Light writes its three settings to `light.toml` and never touches the full
product's `cyberbrain.toml`, so a store used by both keeps two independent configurations.

## Provenance

Light is built on the `cyberbrain-core` and `cyberbrain-index` crates, published by the same
author, and shares no source code with any other memory tool. §0 of Cyberbrain's
[`docs/SPEC.md`](https://github.com/bassprofressor-lab/cyberbrain/blob/main/docs/SPEC.md)
records the clean-room boundary both products are built under.

## Licence

[FSL-1.1-ALv2](LICENSE.md). Use it for anything except building a competing product, and each
release becomes Apache-2.0 two years after it ships. Copyright 2026 Krynex Labs.
