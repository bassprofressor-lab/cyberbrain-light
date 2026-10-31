---
layout: ../layouts/Doc.astro
title: "Five minutes"
description: "Create a store, write a note, find it again and expand a citation, in five minutes."
slug: "quickstart/"
---

# Five minutes

```console
$ cbl init
$ cbl write --ring 2 --kind bug --name pg18-moves-pgdata \
    --body 'The postgres:18 image keeps its data in /var/lib/postgresql/18/docker.'
$ cbl recall 'postgres data'
$ cbl recall --id r2-867ef2a8cd01     # the whole note behind a citation
```

## What just happened

**`cbl init`** created a store in `.cyberbrain/` in the current directory: a `notes/` tree, the index `cyberbrain.db` and `light.toml` with three settings.

**`cbl write`** wrote one Markdown file into `notes/r2/` and indexed it. A note has a **ring** (how much it is trusted), a **kind** (`knowledge`, `bug`, `lesson`, `decision`, `reference`, `session`) and a **name**, a kebab-case slug that is unique in the store. Writing the same name again updates the note and keeps its id. `--body -` reads the body from standard input, which is easier for anything longer than a sentence.

**`cbl recall`** searched rings 2 to 4 for the words you gave it and printed hits with a **citation** such as `r2-867ef2a8cd01`. The citation names a block, not a file, so it stays valid when the note around it grows.

**`cbl recall --id`** expanded a citation into the full note.

## Write notes that can be found

Search is by keyword. A note is found by the words in it, so write them the way someone will ask, not only the way the answer reads. A first line with the symptoms and half-sentences of the person who has the problem works well:

```markdown
*For: postgres container starts empty · data gone after upgrade · PGDATA moved*

The official postgres:18 image puts the data directory at /var/lib/postgresql/18/docker ...
```

## Next

- Put the hook into Claude Code: [hooks and MCP](../claude-code/)
- What rings 0 and 1 are for: [store, rings, config](../store/)
- Every flag: [commands](../commands/)
