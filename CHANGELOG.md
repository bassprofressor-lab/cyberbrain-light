# Changelog

What changed, in the words of someone who has to explain it to a user rather than to a
compiler.

This file is also the record of when each version shipped, which the licence needs: under
[FSL-1.1-ALv2](LICENSE.md) every release turns Apache-2.0 two years after **its own** release
date, so that date has to survive somewhere more durable than a tag that can be moved.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[semantic versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- **An edit keeps the tags.** `cbl write` on an existing note without `--tags`, and every MCP
  `write`, removed all of its tags, among them `trust:untrusted` and `src:…`, which say a note
  came from outside. One overwrite, also by the model, turned an imported e-mail into a note
  like any other. Now tags left out stay; tags given replace the others, but `trust:untrusted`
  and the `src:` tags stay either way.
- **`cbl scan` no longer drops a note it cannot read.** A note whose header Light refuses (a
  typo, a field of a newer Cyberbrain) was taken out of the index although the file was still
  there, and `doctor --fix` did the same; such a note keeps its place now, and `scan` and
  `doctor` say it cannot be read. A note `.cyberbrainignore` hides still leaves the index, and
  `scan` counts it as ignored instead of as removed.
- **The session-start hook honours `.cyberbrainignore`.** It read rings 0 and 1 straight from
  their directories, so a resident note the operator had hidden was still injected into every
  session while `cbl status` no longer counted it.
- **Session start really stays under 10,000 characters.** The closing line that names the
  resident notes left out was added after the count, with every name in it, and a large note
  plus a few long names took the context past what Claude Code shows. The line now counts, and
  past 400 characters of names it gives the number of the rest.
- **MCP refuses a ring that does not exist.** `ring: 258` was taken as ring 2 and `259` as
  ring 3, so a slip of the model wrote into a ring it had not meant. Anything but 0 to 4 is now
  an error the model is shown.

## [0.2.0] - 2026-10-23

- **Built on Cyberbrain 0.7.10** (`cyberbrain-core`, `cyberbrain-index`), Rust 1.99.1,
  `cargo update`.
- **An edit keeps what Light does not edit.** `cbl write` on an existing note set `retention` to
  nothing, and the fields the full Cyberbrain writes (`bereich`, `supersedes`, `superseded_by`,
  `valid_from`, `invalid_at`) were dropped. A store shared with Cyberbrain lost an agreed deletion
  date or a supersession that way on every edit.
- **MCP 2026-07-28.** The server answers `server/discover` and stateless requests (`resultType`,
  `ttlMs`/`cacheScope`, `serverInfo` in `_meta`, `UnsupportedProtocolVersionError`), and
  `initialize` with 2025-11-25. Claude Code 2.1.292 and later probe this first.
- **Session start stays under 10,000 characters.** Claude Code shows a hook's context in full up
  to that and only a 2,000 character preview past it, so rings 0 and 1 could be cut to a preview
  while still under the token cap. Notes that do not fit are left out and named where the model
  reads it.
- **`cbl install` no longer writes the PreCompact hook** and takes out one an older version
  wrote: Claude Code discards a PreCompact hook's `systemMessage`, so it reached nobody. The hook
  it does write has a 30 s timeout (the harness default is 600 s).

## [0.1.0] - 2026-09-07

The first version, on crates.io as `cyberbrain-light` and as a Linux and a Windows binary on
the release page.

- `init`, `write`, `scan`, `recall` (with `--id` to expand a citation), `forget`, `status`
  and `doctor`, all with `--json`.
- Two hooks: `session-start` injects rings 0 and 1 under a token cap, `pre-compact` says what
  is about to be forgotten. `cbl install` writes both into a project's `.claude/settings.json`
  without touching entries it did not write, and `--undo` takes them out again.
- An MCP server over stdio with `recall`, `recall_id`, `write` and `status`. `write` refuses
  rings 0 and 1: they are the operator's.
- The store layout, the notes, the rings and the citations are Cyberbrain's, so a store can be
  handed from one product to the other. Light keeps its own `light.toml` and never edits
  `cyberbrain.toml`.
