# Changelog

What changed, in the words of someone who has to explain it to a user rather than to a
compiler.

This file is also the record of when each version shipped, which the licence needs: under
[FSL-1.1-ALv2](LICENSE.md) every release turns Apache-2.0 two years after **its own** release
date, so that date has to survive somewhere more durable than a tag that can be moved.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[semantic versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2026-11-10

- **Light opens a store Cyberbrain 0.10 has touched.** Built on Cyberbrain 0.10.1
  (`cyberbrain-core`, `cyberbrain-index`), which writes index schema 6; Light 0.2.1 understood
  5 and refused every such store. An older index is brought to schema 6 on open, notes and
  all; after that, Light 0.2.1 and Cyberbrain before 0.10 no longer open it. The update also
  takes `rusqlite` to 0.41.1 and `libsqlite3-sys` to 0.39.1, which fixes RUSTSEC-2026-0459.
- **A store that does not open is no longer a silent session start.** The session-start hook
  printed one line on stderr and gave the model nothing, so it worked as if the project had no
  rules. It now tells the model that rings 0 and 1 are missing and why, and stderr carries the
  whole error. An index newer than Light says to upgrade Light; nothing says to delete it any
  more, since the index belongs to the store and a newer Cyberbrain may keep more in it.
- **Only your own store is found by walking up.** Every command took the first
  `.cyberbrain/notes` above the working directory, whoever it belonged to, and the hook put its
  rings 0 and 1 into the session as the operator's rules; one left in `/tmp` or a shared parent
  directory was enough. The store found that way must now be owned by your account and writable
  by nobody else (group write only for your own primary group), and the walk stops at your home
  directory and at a file system boundary. A refused store is named; `--store` still opens it.
  On Windows ownership is not checked.
- **Outside material and outdated notes are marked in the text.** `cbl recall`, `cbl recall
  --id` and the MCP `recall` and `recall_id` the model reads now say when a note is tagged
  `trust:untrusted` and where it came from, when another note replaces it, and when it is no
  longer or not yet valid. Only `--json` said so before.
- **`cbl install` keeps the mode of `settings.json`** (a 0600 file came back 0644, its backup
  too) **and leaves a link alone**: a `settings.json` that is a symbolic link was replaced by a
  plain file. It now stops and says so, rather than writing through the link into a file
  outside the project.
- **Rings 0 and 1 are left out of an agent's recall only where a hook injects them.** The MCP
  `recall`, and `cbl recall` run by an agent, assumed the session-start hook had put them into
  the context; without it, and in every subagent, nothing had, and the agent met the
  operator's rules nowhere. They are now left out only where the project's settings carry a
  SessionStart and a SubagentStart hook of Light or Cyberbrain. `cbl install` writes the new
  SubagentStart entry (`cbl hook subagent-start`); run it again in projects installed before.
- **An agent's recall finds ring 2 behind many resident matches.** Rings 0 and 1 were dropped
  after the candidates had been cut to 50, so enough resident blocks with the query's words
  returned no hits at all. They are now left out inside the search.
- **Resident notes over the token cap are named to the model**, in the same closing line as
  the ones over the character limit, instead of on stderr only.
- **`scan --full` and `doctor --fix` finish when a name is used twice.** Two files of one name
  in two rings stopped the scan at the second, and after `--full` the index was left half
  empty. The second is now skipped and named, and every other note is indexed.
- **Smaller things.** `n` in `light.toml` is what `recall` returns by default (it was ignored);
  `cbl init` on an existing store keeps and reports the cap in its `light.toml`; writing a
  resident note too long for the session start warns; the MCP server answers a line that is
  not UTF-8 instead of ending, answers a JSON value that is not an object with an error instead
  of nothing, refuses `n` as a string and an empty `body`, and keeps an error's whole chain;
  control characters in note text are written out instead of reaching the terminal.

## [0.2.1] - 2026-10-31

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
- **`recall` searches rings 2 to 4 when an agent asks**, which is what the MCP tool and the
  session-start text always said. Both returned rings 0 and 1 as well, usually first: a second
  copy of what the agent was given at session start. The MCP `recall`, and `cbl recall` run by
  an agent (`CLAUDECODE` or `CYBERBRAIN_AGENT` set), now leave them out and say so; `--ring 0`
  or `--ring 1` still searches them, and a person at the terminal still gets every ring. The
  full Cyberbrain draws the line in the same place.
- **`cbl install` is careful with a settings file that is yours.** It removed every empty hook
  list, also ones it never wrote, and sorted your keys alphabetically. `--undo` rewrote the file
  even with nothing to remove, and each write replaced `settings.json.bak`, so after an install
  and an undo the original was gone. Now a run that changes nothing writes nothing, only its own
  empty lists are tidied away, the keys stay in your order, a backup never replaces an earlier
  one (`settings.json.bak`, then `.bak.1` …), and the file is written through a temporary file
  and a rename.
- **Two writes of the same new note at once no longer split file and index.** Both saw no
  note, both wrote the file, and the second one failed after its file had already replaced the
  first one's; `cbl doctor` then found the store out of step. `write`, `scan` and `forget` now
  take a lock on the store (`.cyberbrain/.light.lock`), and the second write updates the note
  the first one created.
- **The release workflow is harder to tamper with.** Every action is pinned to a commit
  instead of a tag that can be moved (Dependabot keeps the pins current), only the job that
  attaches the files to the release may write to the repository, and the release build no
  longer uses the build cache it shares with ordinary CI runs.
- **Licences and advisories are checked in CI, every week.** There was no `deny.toml`, so
  `cargo deny check` failed on the first MIT crate, and CI never ran it. The file now lists
  the licences the tree actually holds (FSL-1.1-ALv2 only for our own three crates), and a CI
  job runs the check on every push and once a week without one.

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
