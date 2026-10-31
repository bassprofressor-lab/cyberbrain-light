---
layout: ../layouts/Doc.astro
title: "Claude Code: hooks and MCP"
description: "Connect Cyberbrain Light to Claude Code: the session-start hook, cbl install, and the MCP server with recall, recall_id, write and status."
slug: "claude-code/"
---

# Claude Code: hooks and MCP

Light reaches the agent two ways. The **hook** puts the rules into every session before the first prompt. The **MCP server** lets the agent search and write notes while it works.

## cbl install

```console
$ cbl install
1 hook(s) in /path/to/project/.claude/settings.json pointing at /home/you/.cargo/bin/cbl
```

It adds one entry and leaves everything else in the file alone:

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
            "command": "/home/you/.cargo/bin/cbl",
            "args": ["hook", "session-start"],
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

The `_managedBy` marker is how `cbl install --undo` finds its own entries; it never touches one it did not write. The command is the absolute path of the binary that ran `install`, so move or reinstall `cbl` and run `cbl install` again. Since 0.2.0 the hook has a 30-second timeout (the harness default is 600 s), and an old PreCompact entry from 0.1 is taken out.

## The session-start hook

On every new session Claude Code runs `cbl hook session-start`, and Light answers with rings 0 and 1: the operator's invariants and the current protocol. Two limits apply:

- the **token cap** from `light.toml` (`resident_cap_tokens`, default 8,192), and
- the **10,000 characters** Claude Code shows of a hook's context in full. Past that it shows only a 2,000-character preview, so Light leaves notes out rather than let the whole block be cut, and names the notes that did not fit where the model reads it.

Keep rings 0 and 1 short. They are paid for in every session.

## The MCP server

Register it with Claude Code for the current project:

```console
$ claude mcp add cyberbrain-light -- cbl mcp
```

or as a project file `.mcp.json` that the whole team shares:

```json
{
  "mcpServers": {
    "cyberbrain-light": { "command": "cbl", "args": ["mcp"] }
  }
}
```

The server speaks MCP over stdio, answers `initialize` and, since 0.2.0, the 2026-07-28 protocol's `server/discover` and stateless requests. It offers four tools:

| tool | arguments | what it does |
|---|---|---|
| `recall` | `query`, `n` (default 8), `ring` | keyword search over rings 2 to 4 (0 and 1 only when `ring` names them), hits with citations |
| `recall_id` | a citation such as `r2-a91f2c33e1bd` | the whole note behind it |
| `write` | `ring` (2, 3 or 4), `kind`, `name`, `body` | write or update a note; an update keeps its tags |
| `status` | | note counts per ring, the index, the resident budget |

`write` refuses rings 0 and 1. They belong to the person running the project; the tool says to propose the text instead, or to run `cbl write --ring 0 …` yourself.

> Tool arguments come from the model, and the model reads whatever is in its context, including text from web pages and files. Keeping rings 0 and 1 out of the agent's reach is what stops a prompt injection from rewriting the rules every later session starts with.
