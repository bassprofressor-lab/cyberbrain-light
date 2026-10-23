//! `cbl install`: put the hook entry into a project's `.claude/settings.json`, and take it
//! out again.
//!
//! Since 0.2.0 that is SessionStart only. The PreCompact entry spoke to the human through
//! `systemMessage`, which Claude Code discards for that event: it did nothing. A run removes
//! an entry an older version wrote there.
//!
//! The file belongs to the user, not to this tool. Everything that is not ours is copied
//! through untouched, our entries are marked `_managedBy` so they can be found again, and a
//! backup of the previous file is written before anything is replaced.

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

    let hooks = root
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .context("`hooks` in settings.json is not an object")?;

    for harness_event in RETIRED {
        if let Some(list) = hooks.get_mut(harness_event).and_then(Value::as_array_mut) {
            let before = list.len();
            list.retain(|v| v.get("_managedBy").and_then(Value::as_str) != Some(MANAGED_BY));
            if list.len() < before {
                report.removed.push(harness_event.to_string());
            }
        }
    }

    for (harness_event, arg) in EVENTS {
        let list = hooks
            .entry(harness_event)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .with_context(|| format!("`hooks.{harness_event}` is not a list"))?;

        let ours = |v: &Value| v.get("_managedBy").and_then(Value::as_str) == Some(MANAGED_BY);
        let before = list.len();
        let existing = list.iter().find(|v| ours(v)).cloned();
        list.retain(|v| !ours(v));
        let dropped = before - list.len();

        if undo {
            if dropped > 0 {
                report.removed.push(harness_event.to_string());
            }
            continue;
        }
        let entry = json!({
            "matcher": "",
            "hooks": [{ "type": "command", "command": binary, "args": ["hook", arg], "timeout": HOOK_TIMEOUT_SECS }],
            "_managedBy": MANAGED_BY,
        });
        let same = existing.as_ref() == Some(&entry);
        list.push(entry);
        if same {
            report.unchanged.push(harness_event.to_string());
        } else {
            report.added.push(harness_event.to_string());
        }
    }

    // Leave no empty shells behind: an `"hooks": {"PreCompact": []}` after an undo is litter.
    hooks.retain(|_, v| !v.as_array().is_some_and(|a| a.is_empty()));
    let empty = hooks.is_empty();
    if empty {
        root.remove("hooks");
    }

    if report.added.is_empty() && report.removed.is_empty() && !undo {
        return Ok(report);
    }

    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    if path.exists() {
        let backup = path.with_extension("json.bak");
        std::fs::copy(&path, &backup)
            .with_context(|| format!("backing up {} first", path.display()))?;
        report.backup = Some(backup);
    }
    let text = serde_json::to_string_pretty(&Value::Object(root))? + "\n";
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(report)
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
