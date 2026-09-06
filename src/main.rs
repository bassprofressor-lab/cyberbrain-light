//! Cyberbrain Light. One binary, lexical search, no model and no compliance subsystem.
//!
//! Original work, copyright 2026 Krynex Labs, licensed FSL-1.1-ALv2. It shares no source
//! code with any memory tool other than Cyberbrain, whose crates it is built on and whose
//! author owns both.

mod app;
mod cli;
mod hook;
mod install;
mod mcp;
mod render;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use app::{App, STORE_DIR, WriteRequest, find_store};
use clap::Parser;
use cli::{Cli, Cmd};
use cyberbrain_core::types::Ring;
use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    // A hook may never fail the session it serves. Everything it can go wrong at — a broken
    // store, a panic in this code — ends here as exit 0 with a line on stderr.
    if let Cmd::Hook { event } = cli.cmd {
        let outcome = std::panic::catch_unwind(|| run_hook(event, cli.store.clone()))
            .unwrap_or_else(|_| {
                Err(anyhow::anyhow!(
                    "the hook panicked; the session is unaffected"
                ))
            });
        if let Err(e) = outcome {
            eprintln!("cbl hook: {e:#}");
        }
        return ExitCode::SUCCESS;
    }

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("cbl: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_hook(event: cli::HookEvent, store: Option<std::path::PathBuf>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    // A missing store is not an error here: most projects do not have one.
    let root = match find_store(store, &cwd) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);
    let out = hook::run(event, &root, &stdin)?;
    if !out.stdout.is_empty() {
        println!("{}", out.stdout);
    }
    for note in out.notes {
        eprintln!("cbl {}: {note}", event.as_str());
    }
    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let say = |text: String| {
        if !cli.quiet {
            println!("{text}");
        }
    };
    let emit = |value: serde_json::Value| -> Result<()> {
        println!("{}", serde_json::to_string_pretty(&value)?);
        Ok(())
    };

    match cli.cmd {
        Cmd::Init { cap } => {
            let root = cli.store.unwrap_or_else(|| cwd.join(STORE_DIR));
            let (app, existed) = App::init(&root, cap)?;
            let text = format!(
                "{} store at {}\nnext: cbl write --ring 2 --kind knowledge --name first-note \
                 --body 'what you learned', then cbl recall 'what you learned'",
                if existed { "kept" } else { "created" },
                app.store.root().display()
            );
            if cli.json {
                emit(serde_json::json!({
                    "store": app.store.root().display().to_string(),
                    "created": !existed,
                    "resident_cap_tokens": cap,
                }))?;
            } else {
                say(text);
            }
        }

        Cmd::Write {
            ring,
            kind,
            name,
            body,
            tags,
        } => {
            let root = find_store(cli.store, &cwd)?;
            let mut app = App::open(&root)?;
            let body = if body == "-" {
                let mut s = String::new();
                std::io::stdin()
                    .read_to_string(&mut s)
                    .context("reading the note body from stdin")?;
                s
            } else {
                body
            };
            let w = app.write_note(WriteRequest {
                ring: Ring::try_from(ring)?,
                kind: kind.into(),
                name,
                body,
                tags,
            })?;
            if cli.json {
                emit(serde_json::to_value(&w)?)?;
            } else {
                say(render::written_text(&w));
            }
        }

        Cmd::Scan { full } => {
            let root = find_store(cli.store, &cwd)?;
            let mut app = App::open(&root)?;
            let report = app.scan(full)?;
            if cli.json {
                emit(serde_json::to_value(&report)?)?;
            } else {
                say(render::scan_text(&report));
            }
        }

        Cmd::Recall { query, id, n, ring } => {
            let root = find_store(cli.store, &cwd)?;
            let app = App::open(&root)?;
            match (id, query) {
                (Some(citation), _) => {
                    let e = app.expand(&citation)?;
                    if cli.json {
                        emit(serde_json::to_value(&e)?)?;
                    } else {
                        say(render::expanded_text(&e));
                    }
                }
                (None, Some(q)) => {
                    let ring = ring.map(Ring::try_from).transpose()?;
                    let result = app.recall(&q, n, ring)?;
                    if cli.json {
                        emit(serde_json::json!({
                            "hits": result.hits,
                            "caveats": result.caveats,
                        }))?;
                    } else {
                        print!("{}", render::hits_text(&result));
                    }
                }
                (None, None) => anyhow::bail!("give a query, or --id to expand a citation"),
            }
        }

        Cmd::Forget { name } => {
            let root = find_store(cli.store, &cwd)?;
            let mut app = App::open(&root)?;
            let f = app.forget(&name)?;
            if cli.json {
                emit(serde_json::to_value(&f)?)?;
            } else {
                say(render::forgotten_text(&f));
            }
        }

        Cmd::Status => {
            let root = find_store(cli.store, &cwd)?;
            let app = App::open(&root)?;
            let s = app.status()?;
            if cli.json {
                emit(serde_json::to_value(&s)?)?;
            } else {
                say(render::status_text(&s));
            }
        }

        Cmd::Doctor { fix } => {
            let root = find_store(cli.store, &cwd)?;
            let mut app = App::open(&root)?;
            let d = app.doctor(fix)?;
            let clean = d.findings.is_empty();
            if cli.json {
                emit(serde_json::to_value(&d)?)?;
            } else {
                say(render::doctor_text(&d));
            }
            // Something to report and nothing done about it is a non-zero exit, so a nightly
            // `cbl doctor` can fail instead of printing into the void.
            if !clean && !fix {
                std::process::exit(1);
            }
        }

        Cmd::Install { project, undo } => {
            let project = project.unwrap_or(cwd);
            let r = install::run(&project, undo)?;
            if cli.json {
                emit(serde_json::json!({
                    "settings": r.path.display().to_string(),
                    "backup": r.backup.as_ref().map(|p| p.display().to_string()),
                    "added": r.added,
                    "removed": r.removed,
                    "unchanged": r.unchanged,
                    "command": r.binary,
                }))?;
            } else if undo {
                say(format!(
                    "removed {} hook(s) from {}",
                    r.removed.len(),
                    r.path.display()
                ));
            } else {
                say(format!(
                    "{} hook(s) in {} pointing at {}{}",
                    r.added.len() + r.unchanged.len(),
                    r.path.display(),
                    r.binary,
                    match &r.backup {
                        Some(b) => format!("\nprevious file kept as {}", b.display()),
                        None => String::new(),
                    }
                ));
            }
        }

        Cmd::Mcp => {
            let root = find_store(cli.store, &cwd)?;
            mcp::serve(root)?;
        }

        Cmd::Hook { .. } => unreachable!("handled in main"),
    }
    Ok(())
}
