---
layout: ../layouts/Doc.astro
title: "Commands"
description: "Every cbl command and flag: init, write, scan, recall, forget, status, doctor, hook, install, mcp."
slug: "commands/"
---

# Commands

```text
cbl [OPTIONS] <COMMAND>
```

## Options for every command

| option | what it does |
|---|---|
| `--store <STORE>` | Store to operate on. Defaults to `.cyberbrain` found from the working directory upwards, so a command works anywhere inside a project. Also read from `CYBERBRAIN_STORE`. |
| `--json` | Machine-readable output. Every command that prints anything supports it. |
| `-q`, `--quiet` | Print less. Errors still go to stderr. |
| `-h`, `--help` | Help for the command. |
| `-V`, `--version` | Print the version. |

## init

Create a store in the current directory.

| option | default | |
|---|---|---|
| `--cap <CAP>` | `8192` | Token cap for the resident rings 0 and 1 |

## write

Write a note. Writing an existing name updates it and keeps its id.

```text
cbl write --ring <RING> --kind <KIND> --name <NAME> --body <BODY> [--tags <TAGS>]
```

| option | |
|---|---|
| `--ring <RING>` | 0 to 4, see [rings](../store/#rings) |
| `--kind <KIND>` | `knowledge`, `bug`, `lesson`, `decision`, `reference`, `session` |
| `--name <NAME>` | kebab-case slug, unique in the store |
| `--body <BODY>` | the note body in Markdown; `-` reads it from standard input |
| `--tags <TAGS>` | optional, comma-separated |

An existing note keeps its ring. A write that names a different ring is refused, because moving a note changes what it is allowed to override; `cbl forget` it first if the move is what you mean. Fields that Light does not edit, including those the full Cyberbrain writes, are kept on an edit. Tags are not: an edit replaces them with the `--tags` it is given, so repeat them when you update a note.

## scan

Rebuild the index from the notes tree. Use it after editing notes by hand or after pulling notes from someone else.

| option | |
|---|---|
| `--full` | Discard the index and rebuild everything |

## recall

Search the store. The search covers every ring, 0 to 4; `--ring` restricts it to one.

```text
cbl recall [QUERY] [--id <CITATION>] [-n <N>] [--ring <RING>]
```

| option | default | |
|---|---|---|
| `[QUERY]` | | The query. Omit when using `--id` |
| `--id <CITATION>` | | Expand a citation to its full note instead of searching |
| `-n`, `--n <N>` | `8` | How many hits to return |
| `--ring <RING>` | | Restrict to exactly this ring |

## forget

Delete a note, its blocks and its index rows.

```text
cbl forget <NAME>
```

## status

What the store holds and what the index knows: notes per ring, index state, the resident token budget.

## doctor

Report the store's footprint and tidy it up.

| option | |
|---|---|
| `--fix` | Also compact the database and rebuild what is stale |

## hook

Answer an agent lifecycle hook. Reads the event payload on stdin. Claude Code calls this, not you; see [hooks and MCP](../claude-code/).

```text
cbl hook <session-start | pre-compact>
```

`pre-compact` is still accepted so that hooks written by version 0.1 keep working, but `cbl install` no longer writes it: Claude Code discards a PreCompact hook's message.

## install

Write the hook entries into a project's `.claude/settings.json`.

| option | |
|---|---|
| `--project <PROJECT>` | Project directory. Defaults to the current one |
| `--undo` | Remove the entries instead of writing them |

## mcp

Speak the Model Context Protocol over stdio. See [the MCP server](../claude-code/#the-mcp-server).
