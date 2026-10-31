---
layout: ../layouts/Doc.astro
title: "Light or full"
description: "When Cyberbrain Light is enough and when the full Cyberbrain with semantic search, contradiction check and GDPR/AI Act artefacts is the better choice."
slug: "light-or-full/"
---

# Light or full

|  | Light | [Cyberbrain](https://github.com/bassprofressor-lab/cyberbrain) |
|---|---|---|
| search | keyword (BM25) | keyword **and** meaning, fused |
| model | none | an embedding model you place yourself |
| memory while searching | 8 MB | 1.55 GB |
| contradiction check | no | against a local inference endpoint |
| GDPR and EU AI Act artefacts | no | erasure, subject access, audit chain, PII gate, obligations |
| web page | no | built into the binary, English and German |
| binary | 5.0 MB | 13.6 MB |

**Take Light** when one person or a small team wants the agent to remember project knowledge, and the notes contain no personal data: build steps, decisions, bugs and their fixes. It runs anywhere, costs nothing to operate and cannot leak anything over the network.

**Take the full Cyberbrain** when search has to find meaning and not only words, or when the store may hold personal data and you have to show what was stored, who wrote it and that a deletion really happened. That is what its audit chain, PII gate and erasure records are for.

> A store can be used by both. Light does not run the full product's checks, though: a note written through Light skips the PII gate and the audit record, and a note deleted with `cbl forget` leaves no erasure record. In a store that relies on those, write through `cyberbrain` only.

[Cyberbrain on krynexlabs.de](https://www.krynexlabs.de/cyberbrain/)
