# Changelog

What changed, in the words of someone who has to explain it to a user rather than to a
compiler.

This file is also the record of when each version shipped, which the licence needs: under
[FSL-1.1-ALv2](LICENSE.md) every release turns Apache-2.0 two years after **its own** release
date, so that date has to survive somewhere more durable than a tag that can be moved.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[semantic versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The first version. Not published yet: no tag, nothing on crates.io.

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
