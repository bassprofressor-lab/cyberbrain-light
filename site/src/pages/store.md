---
layout: ../layouts/Doc.astro
title: "Store, rings, config"
description: "How Cyberbrain Light stores notes: the .cyberbrain directory, the five trust rings, citations and the three settings in light.toml."
slug: "store/"
---

# Store, rings, config

## The store

`cbl init` creates `.cyberbrain/` in the project:

```text
.cyberbrain/
├── notes/          # your notes, plain Markdown, one directory per ring
│   ├── r0/
│   ├── r1/
│   └── r2/ …
├── cyberbrain.db   # the search index (SQLite, FTS5); can always be rebuilt with `cbl scan --full`
└── light.toml      # three settings
```

The notes are the truth; the index is derived from them. Commit `notes/` with your code if the team should share the memory, and keep `cyberbrain.db` out of version control.

Any command finds the store from the working directory upwards, so it works anywhere inside the project. `--store` or `CYBERBRAIN_STORE` point it somewhere else.

## Rings

Every note lives in a numbered ring, and the number is a claim about trust.

| ring | what belongs there | in every session |
|---|---|---|
| r0 | operator invariants, hard rules | yes |
| r1 | operating protocol, handoff state | yes |
| r2 | project knowledge | on recall |
| r3 | session records | on recall |
| r4 | imported or unverified material | on recall |

A lower number means more trust. Rings 0 and 1 ride along in every session under a [token cap](#lighttoml), so the agent starts knowing the rules instead of rediscovering them. They belong to you: the MCP `write` tool refuses them.

A note stays in its ring. Moving it is a decision, not a side effect, so a `write` that names another ring is refused; `cbl forget` the note and write it again.

## Citations

Every search hit carries a citation such as `r2-867ef2a8cd01`: the ring and a block id. It names one block of a note, not the file, so it survives the note growing around it. `cbl recall --id <citation>` gives back the whole note.

## light.toml

Three settings, and deleting the file brings back the defaults.

```toml
# Rings 0 and 1 are injected into every session; this caps what that costs.
resident_cap_tokens = 8192

# Hits returned by `cbl recall`.
n = 8

# Candidates BM25 weighs before the list is cut to `n`.
k_lex = 50
```

Independently of the token cap, the session-start hook stays under the 10,000 characters Claude Code shows in full; see [hooks](../claude-code/#the-session-start-hook).

## Shared with the full Cyberbrain

The store layout is Cyberbrain's: same directory, same Markdown, same rings, same citations. Start with Light, install [Cyberbrain](../light-or-full/) later, and one `cyberbrain scan` picks up every note you already wrote. Light keeps its settings in `light.toml` and never touches Cyberbrain's `cyberbrain.toml`.
