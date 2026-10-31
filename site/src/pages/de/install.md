---
layout: ../../layouts/Doc.astro
title: "Installation"
description: "Cyberbrain Light von crates.io installieren oder als geprüftes Release-Programm für Linux und Windows."
lang: de
slug: "install/"
---

# Installation

Das Paket heißt `cyberbrain-light`, das Programm, das es installiert, heißt `cbl`.

## Über crates.io

```console
$ cargo install cyberbrain-light --locked
$ cbl --version
cbl 0.2.0
```

Dafür braucht es Rust 1.98 oder neuer. `--locked` baut mit genau den Abhängigkeiten, mit denen das Release getestet wurde.

## Als Release-Programm

Jedes Release auf GitHub enthält ein Programm für Linux und eins für Windows sowie die Datei `SHA256SUMS`. Vor dem Start prüfen:

```console
$ sha256sum -c SHA256SUMS --ignore-missing
cbl-linux-x86_64: OK
$ chmod +x cbl-linux-x86_64
$ ./cbl-linux-x86_64 --version
```

[Neuestes Release auf GitHub](https://github.com/bassprofressor-lab/cyberbrain-light/releases/latest)

## Was es zur Laufzeit braucht

Nichts. Kein SQLite im System, kein OpenSSL, keinen Modellserver, kein Node. Der Suchindex ist SQLite mit FTS5 und steckt im Programm.

## Danach

```console
$ cd dein-projekt
$ cbl init             # legt hier .cyberbrain/ an
$ cbl install          # trägt den Hook in .claude/settings.json ein
```

Weiter mit [fünf Minuten](../quickstart/).

## Deinstallieren

```console
$ cbl install --undo   # entfernt nur die Einträge, die cbl geschrieben hat
$ cargo uninstall cyberbrain-light
```

Die Notizen bleiben, wo sie sind: `.cyberbrain/notes/` ist einfaches Markdown und gehört dir.
