//! `cbl install`: put the hook entry into a project's `.claude/settings.json`, and take it
//! out again.
//!
//! Since 0.2.0 that is SessionStart only. The PreCompact entry spoke to the human through
//! `systemMessage`, which Claude Code discards for that event: it did nothing. A run removes
//! an entry an older version wrote there.
//!
//! The file belongs to the user, not to this tool. Everything that is not ours is copied
//! through untouched and in its order, our entries are marked `_managedBy` so they can be
//! found again, and a backup of the previous file is written before anything is replaced —
//! beside the earlier ones, never over them. A run that changes nothing writes nothing.

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};

pub const MANAGED_BY: &str = "cyberbrain-light";
const EVENTS: [(&str, &str); 1] = [("SessionStart", "session-start")];
/// Events an older version wrote an entry for and this one no longer does.
const RETIRED: [&str; 1] = ["PreCompact"];
/// Seconds before Claude Code gives up on the hook. Without it the harness waits 600 s.
pub const HOOK_TIMEOUT_SECS: u64 = 30;

#[derive(Debug)]
pub struct InstallReport {
    pub path: PathBuf,
    pub backup: Option<PathBuf>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
    pub binary: String,
}

pub fn run(project: &Path, undo: bool) -> Result<InstallReport> {
    let dir = project.join(".claude");
    let path = dir.join("settings.json");
    let binary = current_binary();

    let mut root: Map<String, Value> = match std::fs::read_to_string(&path) {
        Ok(text) if !text.trim().is_empty() => serde_json::from_str(&text).with_context(|| {
            format!(
                "{} is not valid JSON; fix or move it, this tool will not overwrite it",
                path.display()
            )
        })?,
        Ok(_) => Map::new(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if undo {
                bail!("{} does not exist; nothing to undo", path.display());
            }
            Map::new()
        }
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display()))?,
    };

    let mut report = InstallReport {
        path: path.clone(),
        backup: None,
        added: Vec::new(),
        removed: Vec::new(),
        unchanged: Vec::new(),
        binary: binary.clone(),
    };

    let had_hooks = root.contains_key("hooks");
    if undo && !had_hooks {
        return Ok(report);
    }
    let hooks = root
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .context("`hooks` in settings.json is not an object")?;
    // The lists this run created or took an entry out of. Only those may be tidied away when
    // they end up empty (L7, 2026-10-31): an empty list somebody else wrote is theirs.
    let mut touched: Vec<&str> = Vec::new();

    for harness_event in RETIRED {
        if let Some(list) = hooks.get_mut(harness_event).and_then(Value::as_array_mut) {
            let before = list.len();
            list.retain(|v| v.get("_managedBy").and_then(Value::as_str) != Some(MANAGED_BY));
            if list.len() < before {
                report.removed.push(harness_event.to_string());
                touched.push(harness_event);
            }
        }
    }

    for (harness_event, arg) in EVENTS {
        if undo && !hooks.contains_key(harness_event) {
            continue;
        }
        if !hooks.contains_key(harness_event) {
            touched.push(harness_event);
        }
        let list = hooks
            .entry(harness_event)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .with_context(|| format!("`hooks.{harness_event}` is not a list"))?;

        let ours = |v: &Value| v.get("_managedBy").and_then(Value::as_str) == Some(MANAGED_BY);
        let before = list.len();
        let existing = list.iter().find(|v| ours(v)).cloned();
        let entry = json!({
            "matcher": "",
            "hooks": [{ "type": "command", "command": binary, "args": ["hook", arg], "timeout": HOOK_TIMEOUT_SECS }],
            "_managedBy": MANAGED_BY,
        });

        if undo {
            list.retain(|v| !ours(v));
            if list.len() < before {
                report.removed.push(harness_event.to_string());
                touched.push(harness_event);
            }
            continue;
        }
        // Ours and already exactly right, once: nothing to do, and nothing moves.
        if list.iter().filter(|v| ours(v)).count() == 1 && existing.as_ref() == Some(&entry) {
            report.unchanged.push(harness_event.to_string());
            continue;
        }
        list.retain(|v| !ours(v));
        list.push(entry);
        report.added.push(harness_event.to_string());
    }

    // Leave no empty shells of our own behind: an `"hooks": {"PreCompact": []}` after an undo
    // is litter. Somebody else's empty list stays.
    hooks.retain(|k, v| {
        !(touched.contains(&k.as_str()) && v.as_array().is_some_and(|a| a.is_empty()))
    });
    if hooks.is_empty() && (!had_hooks || !touched.is_empty()) {
        root.remove("hooks");
    }

    // Nothing changed, nothing written: no rewrite, no backup (L7). `--undo` with nothing to
    // undo used to write the file and replace the backup anyway.
    if report.added.is_empty() && report.removed.is_empty() {
        return Ok(report);
    }

    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    if path.exists() {
        report.backup = Some(backup(&path)?);
    }
    let text = serde_json::to_string_pretty(&Value::Object(root))? + "\n";
    // Through a temporary file and a rename: a crash halfway leaves the old file, not half of
    // the new one.
    cyberbrain_core::store::write_atomic(&path, text.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(report)
}

/// Copy `path` beside itself before it is replaced, and never over an earlier copy (L7): an
/// install and an undo used to leave only the state before the undo, and the original was
/// gone. The first copy is `settings.json.bak`, later ones `settings.json.bak.1`, `.2` …; a
/// copy with the same bytes is reused instead of made again.
fn backup(path: &Path) -> Result<PathBuf> {
    let current =
        std::fs::read(path).with_context(|| format!("reading {} to back it up", path.display()))?;
    let base = path.with_extension("json.bak");
    for n in 0u32.. {
        let candidate = if n == 0 {
            base.clone()
        } else {
            let mut name = base.clone().into_os_string();
            name.push(format!(".{n}"));
            PathBuf::from(name)
        };
        match std::fs::read(&candidate) {
            Ok(bytes) if bytes == current => return Ok(candidate),
            Ok(_) => continue,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                std::fs::write(&candidate, &current)
                    .with_context(|| format!("backing up {} first", path.display()))?;
                return Ok(candidate);
            }
            Err(e) => return Err(e).with_context(|| format!("reading {}", candidate.display())),
        }
    }
    unreachable!("u32 backups")
}

/// The absolute path of the running binary, so the hook keeps working when the agent's
/// working directory or `PATH` is not what it was at install time.
fn current_binary() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "cbl".to_string())
}
