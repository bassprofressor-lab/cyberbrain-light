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

/// Rings 0 and 1, rendered with their citations, cut at the token cap.
///
/// Read straight from `notes/r0` and `notes/r1` rather than through `Store::list`, so the
/// cost is bounded by the cap and not by the size of the store. An invariant that cannot be
/// read is reported rather than skipped in silence.
fn resident_context(app: &App, source: Option<&str>) -> Result<(String, Vec<String>)> {
    let mut notes = Vec::new();
    let mut out = String::new();
    let mut tokens = 0usize;
    let cap = app.store.resident_cap();
    let mut capped = false;
    let mut count = 0usize;

    for ring in [Ring::Invariant, Ring::Protocol] {
        let dir = app.store.ring_dir(ring);
        let mut paths: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|e| e == "md"))
                .filter(|p| {
                    !p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with('.'))
                })
                .collect(),
            Err(_) => continue,
        };
        paths.sort();

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
            tokens += cost;
            count += 1;

            if out.is_empty() {
                out.push_str(
                    "# Cyberbrain Light\n\nRings 0 and 1 are resident: they apply to \
                     everything in this project. Every block below carries its citation; \
                     quote it when you rely on it, and expand one with `cbl recall --id \
                     <citation>`. Everything else is behind `cbl recall \"<question>\"`, \
                     which searches rings 2 to 4 by keyword.\n\n",
                );
            }
            out.push_str(&format!(
                "## r{} `{}` ({})\n\n",
                ring.as_u8(),
                note.front.name,
                note.front.updated.strftime("%Y-%m-%d")
            ));
            let (blocks, _) = blocks_of(&note, MAX_BLOCK_TOKENS);
            for b in blocks {
                out.push_str(&format!("[{}]\n{}\n\n", b.citation, b.text.trim()));
            }
        }
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
