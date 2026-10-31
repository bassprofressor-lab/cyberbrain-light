---
layout: ../layouts/Doc.astro
title: "Install"
description: "Install Cyberbrain Light from crates.io or as a checked release binary for Linux and Windows."
slug: "install/"
---

# Install

The crate is called `cyberbrain-light`; the binary it puts on your path is `cbl`.

## From crates.io

```console
$ cargo install cyberbrain-light --locked
$ cbl --version
cbl 0.2.0
```

This needs a Rust toolchain (1.98 or newer). `--locked` builds with exactly the dependency versions the release was tested with.

## As a release binary

Every release on GitHub carries a Linux and a Windows binary and a `SHA256SUMS` file. Check the binary before you run it:

```console
$ sha256sum -c SHA256SUMS --ignore-missing
cbl-linux-x86_64: OK
$ chmod +x cbl-linux-x86_64
$ ./cbl-linux-x86_64 --version
```

[Latest release on GitHub](https://github.com/bassprofressor-lab/cyberbrain-light/releases/latest)

## What it needs at runtime

Nothing. No system SQLite, no OpenSSL, no model server, no Node. The search index is SQLite with FTS5, compiled into the binary.

## Then

```console
$ cd your-project
$ cbl init             # creates .cyberbrain/ here
$ cbl install          # writes the session-start hook into .claude/settings.json
```

Continue with [five minutes](../quickstart/).

## Uninstall

```console
$ cbl install --undo   # removes only the entries cbl wrote
$ cargo uninstall cyberbrain-light
```

The notes stay where they are: `.cyberbrain/notes/` is plain Markdown and belongs to you.
