//! The two lifecycle hooks. Both are written to the same rule: a hook may be useless, but it
//! may never take the session down with it. Every failure path here ends in exit code 0 with
//! an explanation on stderr, where the harness shows it to the human and not to the model.

use crate::app::App;
use crate::cli::HookEvent;
use anyhow::Result;
use cyberbrain_core::blocks::{MAX_BLOCK_TOKENS, approx_tokens, blocks_of};
use cyberbrain_core::types::Ring;
use serde_json::{Value, json};
use std::path::Path;

/// What the harness calls the event in `hookSpecificOutput`.
fn harness_name(event: HookEvent) -> &'static str {
    match event {
        HookEvent::SessionStart => "SessionStart",
        HookEvent::PreCompact => "PreCompact",
    }
}

pub struct HookOutput {
    pub stdout: String,
    pub notes: Vec<String>,
}

/// Read the payload leniently. A payload that does not parse is not an error: the hook does
/// its work with what it has and says on stderr what was wrong with the input.
fn source_of(stdin: &str, notes: &mut Vec<String>) -> Option<String> {
    let text = stdin.trim();
    if text.is_empty() {
        return None;
    }
    match serde_json::from_str::<Value>(text) {
        Ok(v) => v
            .get("source")
            .or_else(|| v.get("trigger"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        Err(e) => {
            notes.push(format!("stdin is not JSON ({e}); proceeding without it"));
            None
        }
    }
}

pub fn run(event: HookEvent, store_root: &Path, stdin: &str) -> Result<HookOutput> {
    let mut notes = Vec::new();
    let source = source_of(stdin, &mut notes);

    let app = match App::open(store_root) {
        Ok(a) => a,
        Err(e) => {
            // No store is the normal state of a project that has not opted in. Say so once,
            // quietly, and let the session continue.
            return Ok(HookOutput {
                stdout: String::new(),
                notes: vec![format!("no usable store at {}: {e}", store_root.display())],
            });
        }
    };

    match event {
        HookEvent::SessionStart => {
            let (context, mut more) = resident_context(&app, source.as_deref())?;
            notes.append(&mut more);
            if context.is_empty() {
                return Ok(HookOutput {
                    stdout: String::new(),
                    notes,
                });
            }
            Ok(HookOutput {
                stdout: json!({
                    "hookSpecificOutput": {
                        "hookEventName": harness_name(event),
                        "additionalContext": context,
                    }
                })
                .to_string(),
                notes,
            })
        }
        HookEvent::PreCompact => {
            // Compaction is the moment a session forgets. Nothing can be injected here, so
            // the hook addresses the human: what was not written down is about to be gone.
            let stats = app.index.stats()?;
            Ok(HookOutput {
                stdout: json!({
                    "systemMessage": format!(
                        "Cyberbrain Light: the context is being compacted. {} note(s) are in the \
                         store and will be back at the next session start; anything learned in \
                         this one and not written with `cbl write` will not.",
                        stats.notes
                    )
                })
                .to_string(),
                notes,
            })
        }
    }
}

/// Claude Code shows a hook's `additionalContext` in full up to 10,000 characters; past that
/// the model gets a 2,000 character preview and a file path. Kept under it with room for the
/// closing line that names what did not fit (0.2.0).
const CONTEXT_CHAR_CAP: usize = 9_400;

/// Rings 0 and 1, rendered with their citations, cut at the token cap and at the character
/// cap, whichever comes first.
///
/// The notes come from `Store::list`, so `.cyberbrainignore` applies here as it does to
/// `status` and `scan`. Listing walks the tree without reading it; only rings 0 and 1 are
/// opened, so what this costs is still bounded by the cap. An invariant that cannot be read
/// is reported rather than skipped in silence.
fn resident_context(app: &App, source: Option<&str>) -> Result<(String, Vec<String>)> {
    let mut notes = Vec::new();
    let mut out = String::new();
    let mut tokens = 0usize;
    let cap = app.store.resident_cap();
    let mut capped = false;
    let mut count = 0usize;
    let mut chars = 0usize;
    let mut too_long: Vec<String> = Vec::new();

    // Through the store's listing, which honours `.cyberbrainignore` (L4, 2026-10-31): a
    // resident note the operator hid is the one that must not ride along any more.
    let listing = app.store.list()?;
    for ring in [Ring::Invariant, Ring::Protocol] {
        let paths: Vec<_> = listing
            .entries
            .iter()
            .filter(|e| e.ring == ring)
            .map(|e| e.path.clone())
            .collect();

        for path in paths {
            let note = match app.store.read_path(&path) {
                Ok(n) => n,
                Err(e) => {
                    notes.push(format!(
                        "{}: unreadable, not injected ({e})",
                        path.display()
                    ));
                    continue;
                }
            };
            let cost = approx_tokens(&note.body) as usize;
            if tokens + cost > cap {
                capped = true;
                continue;
            }

            let mut piece = String::new();
            if out.is_empty() {
                piece.push_str(
                    "# Cyberbrain Light\n\nRings 0 and 1 are resident: they apply to \
                     everything in this project. Every block below carries its citation; \
                     quote it when you rely on it, and expand one with `cbl recall --id \
                     <citation>`. Everything else is behind `cbl recall \"<question>\"`, \
                     which searches rings 2 to 4 by keyword.\n\n",
                );
            }
            piece.push_str(&format!(
                "## r{} `{}` ({})\n\n",
                ring.as_u8(),
                note.front.name,
                note.front.updated.strftime("%Y-%m-%d")
            ));
            let (blocks, _) = blocks_of(&note, MAX_BLOCK_TOKENS);
            for b in blocks {
                piece.push_str(&format!("[{}]\n{}\n\n", b.citation, b.text.trim()));
            }
            if chars + piece.chars().count() > CONTEXT_CHAR_CAP {
                too_long.push(format!("r{}/{}", ring.as_u8(), note.front.name));
                continue;
            }
            chars += piece.chars().count();
            tokens += cost;
            count += 1;
            out.push_str(&piece);
        }
    }

    if !too_long.is_empty() {
        // Said to the model, not only on stderr: an invariant it cannot see must at least
        // be one it knows it is missing.
        out.push_str(&format!(
            "Not shown, because Claude Code shows at most 10,000 characters here: {}. \
             Open them under notes/r0 and notes/r1 before acting on anything they might cover.\n",
            too_long.join(", ")
        ));
        notes.push(format!(
            "{} resident note(s) did not fit the 10,000 characters Claude Code shows: {}",
            too_long.len(),
            too_long.join(", ")
        ));
    }
    if capped {
        notes.push(format!(
            "the resident cap of {cap} tokens was reached; some notes in r0/r1 were not injected"
        ));
    }
    if let Some(s) = source {
        notes.push(format!("session-start source: {s}"));
    }
    notes.push(format!("{count} resident note(s), ~{tokens} tokens"));
    Ok((out, notes))
}
