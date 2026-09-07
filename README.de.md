<div align="center">

# Cyberbrain Light

**Zitiertes, vertrauensgestuftes Gedächtnis für KI-Coding-Agenten. Eine kleine Binärdatei, Suche nach Wörtern, nichts einzurichten.**

[![Lizenz: FSL-1.1-ALv2](https://img.shields.io/badge/Lizenz-FSL--1.1--ALv2-blue)](LICENSE.md)
[![Rust 1.98+](https://img.shields.io/badge/rust-1.98%2B-b7410e)](rust-toolchain.toml)
[![Linux und Windows](https://img.shields.io/badge/l%C3%A4uft%20auf-Linux%20%C2%B7%20Windows-333)](#installieren)
[![Keine Telemetrie](https://img.shields.io/badge/Telemetrie-gibt%20es%20nicht-2ea44f)](#was-es-nicht-tut)

[English](README.md) · **Deutsch**

[Installieren](#installieren) · [Fünf Minuten](#fünf-minuten) · [Was es kostet](#was-es-kostet) ·
[Light oder groß](#cyberbrain-oder-cyberbrain-light) · [Lizenz](#lizenz)

</div>

Dein Agent vergisst zwischen zwei Sitzungen alles. Light gibt ihm Notizen, die das überleben:
reines Markdown im Repository, eine Suche über die Wörter darin, und jeder Treffer trägt ein
Zitat, das auf genau den Block zurückführt, aus dem er stammt. Fünf Megabyte, kein Modell
abzulegen, kein Dienst zu betreiben, keine Konfiguration, die man erst richtig hinbekommen
muss.

```console
$ cbl recall 'postgres datenverzeichnis'
1. r2-867ef2a8cd01  r2  pg18-moves-pgdata  (100% of top)
     # PostgreSQL 18 verlegt PGDATA
     Das offizielle `postgres:18`-Image legt das Datenverzeichnis nach
     `/var/lib/postgresql/18/docker` statt `/var/lib/postgresql/data` ...
caveat: search is lexical: it matches the words in the text, not the meaning.
```

Der Caveat ist keine Entschuldigung, er ist die Zusage. Light sucht nach Wörtern. Es sagt das
bei jedem Ergebnis, damit niemand auf eine Suche baut, die gar nicht stattgefunden hat.

## Installieren

```console
$ cargo install cyberbrain-light
$ cbl init
$ cbl install          # schreibt die zwei Hooks in .claude/settings.json
```

Das Paket heißt `cyberbrain-light`, das Programm, das dabei in den Pfad kommt, heißt `cbl`.

Oder ein Binary aus dem [letzten Release](https://github.com/bassprofressor-lab/cyberbrain-light/releases/latest)
nehmen und gegen die `SHA256SUMS` daneben prüfen:

```console
$ sha256sum -c SHA256SUMS
$ ./cbl-linux-x86_64 init
```

Linux und Windows. Zur Laufzeit braucht es nichts weiter: kein SQLite aus dem System, kein
OpenSSL, keinen Modellserver, kein node. `cbl install --undo` nimmt die Hooks wieder heraus,
und es fasst keinen Eintrag an, den es nicht selbst geschrieben hat.

## Fünf Minuten

```console
$ cbl init
$ cbl write --ring 2 --kind bug --name pg18-moves-pgdata --body 'was du gelernt hast'
$ cbl recall 'was du gelernt hast'
$ cbl recall --id r2-867ef2a8cd01     # die ganze Notiz hinter einem Zitat
```

Vier Wege hinein, alle aus derselben Binärdatei:

| | |
|---|---|
| `cbl <befehl>` | `init`, `write`, `scan`, `recall`, `forget`, `status`, `doctor` — überall mit `--json` |
| `cbl hook <ereignis>` | `session-start` speist Ring 0 und 1 ein, `pre-compact` sagt, was gleich vergessen wird |
| `cbl mcp` | Model Context Protocol über stdio: `recall`, `recall_id`, `write`, `status` |
| `cbl install` | setzt die Hooks in die `.claude/settings.json` eines Projekts und nimmt sie wieder heraus |

### Ringe

Notizen liegen in nummerierten Ringen, und die Nummer ist eine Aussage über Vertrauen.

| Ring | was dort hingehört | in jeder Sitzung eingespeist |
|---|---|---|
| r0 | Invarianten des Betreibers, harte Regeln | ja |
| r1 | Arbeitsprotokoll, Übergabestand | ja |
| r2 | Projektwissen | bei `recall` |
| r3 | Sitzungsverläufe | bei `recall` |
| r4 | Importiertes oder Ungeprüftes | bei `recall` |

Ring 0 und 1 fahren in jeder Sitzung mit, gedeckelt über ein Token-Budget, damit der Agent die
Regeln kennt, statt sie neu zu entdecken. Sie gehören dir: das MCP-Werkzeug `write` weist sie
ab und sagt, man solle den Text vorschlagen.

## Was es kostet

| | |
|---|---|
| die Binärdatei | 5,0 MB |
| eine Suche | 3 ms, 8 MB Arbeitsspeicher |
| der session-start-Hook | 2 ms, 6 MB |
| den ganzen Index neu bauen | 0,7 s |
| auf der Platte | deine Notizen, dazu 18 MB Index |

<sub>Gemessen am 06.09.2026 gegen einen echten Store mit 1.007 Notizen und 4.034 Blöcken, auf
einem Server ohne GPU (AMD EPYC-Milan, 12 vCPU). Die Zeiten enthalten den Prozessstart, weil
das der Preis ist, den ein Hook tatsächlich zahlt.</sub>

## Was es nicht tut

Light ist genauso durch das bestimmt, was nicht drin ist. Nichts davon gibt es hier, und
nichts davon kommt noch: **semantische Suche** (es gibt kein Modell und keine Einbettung
abzulegen), eine **Widerspruchsprüfung** (kein Inferenz-Endpunkt, also keine zweite Meinung zu
einem Treffer), **Ausgangsregister, Audit-Kette, PII-Gate, Löschprotokoll oder
Pflichtenkatalog** (das Compliance-Teilsystem ist die Daseinsberechtigung des anderen
Produkts), und eine **Weboberfläche**.

Ebenfalls nicht: Telemetrie, Hintergrund-Abgleich, Netzzugriff jeder Art. Light baut überhaupt
keine Verbindung nach draußen auf, nicht als Einstellung, sondern weil kein Code darin das
könnte.

## Cyberbrain oder Cyberbrain Light

|  | Light | [Cyberbrain](https://github.com/bassprofressor-lab/cyberbrain) |
|---|---|---|
| Suche | Wörter (BM25) | Wörter **und** Bedeutung, verschmolzen |
| Modell | keines | ein statisches Einbettungsmodell, das du selbst ablegst |
| Speicher während der Suche | 8 MB | 1,55 GB |
| Widerspruchsprüfung | — | gegen einen lokalen Inferenz-Endpunkt |
| Belege für DSGVO und EU AI Act | — | Löschung, Auskunft, Audit-Kette, PII-Gate, Pflichten |
| Weboberfläche | — | in der Binärdatei, deutsch und englisch |
| Binärdatei | 5,0 MB | 13,6 MB |

**Der Store ist derselbe.** Gleiches Verzeichnis, gleiches Markdown, gleiche Ringe, gleiche
Zitate. Wer mit Light anfängt und später Cyberbrain installiert, holt mit einem
`cyberbrain scan` jede Notiz ab, die schon da ist. Light schreibt seine drei Einstellungen
nach `light.toml` und fasst die `cyberbrain.toml` des großen Produkts nie an, ein von beiden
benutzter Store behält also zwei unabhängige Konfigurationen.

## Herkunft

Light baut auf den Paketen `cyberbrain-core` und `cyberbrain-index` desselben Autors auf und
teilt keinen Quelltext mit irgendeinem anderen Gedächtnis-Werkzeug. §0 von Cyberbrains
[`docs/SPEC.md`](https://github.com/bassprofressor-lab/cyberbrain/blob/main/docs/SPEC.md) hält
die Clean-Room-Grenze fest, unter der beide Produkte gebaut sind.

## Lizenz

[FSL-1.1-ALv2](LICENSE.md). Nutzbar für alles, außer um ein konkurrierendes Produkt zu bauen,
und jede Version wird zwei Jahre nach ihrem Erscheinen Apache-2.0. Copyright 2026 Krynex Labs.
