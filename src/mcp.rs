//! Model Context Protocol over stdio: one JSON-RPC message per line, requests answered in
//! the order they arrive.
//!
//! Four tools, which is the whole surface an agent needs: search, expand a citation, write a
//! note, and ask what the store holds. Everything else a person does at the command line.

use crate::app::App;
use anyhow::Result;
use cyberbrain_core::types::{NoteKind, Ring};
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub const LATEST_PROTOCOL: &str = "2025-06-18";
pub const SUPPORTED_PROTOCOLS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "Cyberbrain Light is this project's memory. Search it with \
    `recall` before reading files or guessing; every hit carries a citation such as \
    r2-a91f2c33e1bd that `recall_id` expands to the full note. Record what you learn with \
    `write`: ring 2 for project knowledge, ring 3 for a session record. Rings 0 and 1 \
    belong to the operator — propose their text, never write them. Search is by keyword, \
    not by meaning, so use the words that would be in the note.";

/// Serve until stdin closes. Errors inside a request become JSON-RPC errors; only a broken
/// stdout ends the loop.
pub fn serve(store_root: PathBuf) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut server = Server { store_root };
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = server.handle_line(&line) {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

pub(crate) struct Server {
    pub(crate) store_root: PathBuf,
}

impl Server {
    /// `None` for a notification, which by the protocol gets no answer at all.
    pub(crate) fn handle_line(&mut self, line: &str) -> Option<String> {
        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                return Some(
                    error_response(&Value::Null, -32700, &format!("parse error: {e}")).to_string(),
                );
            }
        };
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let is_notification = msg.get("id").is_none();

        let result = match method {
            "initialize" => Ok(self.initialize(&params)),
            "notifications/initialized" | "notifications/cancelled" => Ok(Value::Null),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(tool_list()),
            "tools/call" => self.call(&params),
            other => Err((-32601, format!("unknown method `{other}`"))),
        };
        if is_notification {
            return None;
        }
        Some(match result {
            Ok(value) => json!({ "jsonrpc": "2.0", "id": id, "result": value }).to_string(),
            Err((code, message)) => error_response(&id, code, &message).to_string(),
        })
    }

    fn initialize(&self, params: &Value) -> Value {
        let asked = params
            .get("protocolVersion")
            .and_then(Value::as_str)
            .unwrap_or("");
        let version = if SUPPORTED_PROTOCOLS.contains(&asked) {
            asked
        } else {
            LATEST_PROTOCOL
        };
        json!({
            "protocolVersion": version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": {
                "name": "cyberbrain-light",
                "title": "Cyberbrain Light",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "instructions": INSTRUCTIONS,
        })
    }

    fn call(&mut self, params: &Value) -> std::result::Result<Value, (i64, String)> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or((-32602, "tools/call needs a `name`".to_string()))?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));
        // A tool that fails is not a protocol error: the model is told what went wrong and
        // gets to try something else, which is what `isError` is for.
        match self.dispatch(name, &args) {
            Ok(text) => Ok(json!({
                "content": [{ "type": "text", "text": text }],
                "isError": false,
            })),
            Err(e) => Ok(json!({
                "content": [{ "type": "text", "text": e.to_string() }],
                "isError": true,
            })),
        }
    }

    fn dispatch(&mut self, name: &str, args: &Value) -> Result<String> {
        let root: &Path = &self.store_root;
        let s = |k: &str| {
            args.get(k)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        match name {
            "recall" => {
                let app = App::open(root)?;
                let n = args.get("n").and_then(Value::as_u64).unwrap_or(8) as usize;
                let ring = args
                    .get("ring")
                    .and_then(Value::as_u64)
                    .map(|r| Ring::try_from(r as u8))
                    .transpose()?;
                let result = app.recall(&s("query"), n.clamp(1, 50), ring)?;
                Ok(crate::render::hits_text(&result))
            }
            "recall_id" => {
                let app = App::open(root)?;
                let e = app.expand(&s("citation"))?;
                Ok(format!(
                    "{} r{} `{}`\n\n{}",
                    e.citation,
                    e.ring.as_u8(),
                    e.name,
                    e.body.trim()
                ))
            }
            "write" => {
                let mut app = App::open(root)?;
                let ring_n = args
                    .get("ring")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow::anyhow!("write needs `ring` (2, 3 or 4)"))?;
                let ring = Ring::try_from(ring_n as u8)?;
                if ring.is_resident() {
                    anyhow::bail!(
                        "rings 0 and 1 belong to the operator; propose the text instead of \
                         writing it, or ask them to run `cbl write --ring {} …` themselves",
                        ring.as_u8()
                    );
                }
                let kind = match s("kind").as_str() {
                    "bug" => NoteKind::Bug,
                    "lesson" => NoteKind::Lesson,
                    "decision" => NoteKind::Decision,
                    "reference" => NoteKind::Reference,
                    "session" => NoteKind::Session,
                    "" | "knowledge" => NoteKind::Knowledge,
                    other => anyhow::bail!("unknown kind `{other}`"),
                };
                let w = app.write_note(crate::app::WriteRequest {
                    ring,
                    kind,
                    name: s("name"),
                    body: s("body"),
                    tags: Vec::new(),
                })?;
                Ok(format!(
                    "{} `{}` in r{} ({} block(s))",
                    if w.created { "wrote" } else { "updated" },
                    w.name,
                    w.ring.as_u8(),
                    w.blocks
                ))
            }
            "status" => {
                let app = App::open(root)?;
                let st = app.status()?;
                Ok(crate::render::status_text(&st))
            }
            other => anyhow::bail!("unknown tool `{other}`"),
        }
    }
}

fn tool_list() -> Value {
    json!({ "tools": [
        {
            "name": "recall",
            "title": "Search the project memory",
            "description": "Keyword search over rings 2 to 4. Returns hits with a citation, \
                             a ring and a score, plus what was not checked. Use the words \
                             that would appear in the note; there is no semantic search.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "the question, in the words a note would use" },
                    "n": { "type": "integer", "description": "hits to return (default 8)" },
                    "ring": { "type": "integer", "description": "restrict to exactly this ring" },
                },
                "required": ["query"],
            },
        },
        {
            "name": "recall_id",
            "title": "Expand a citation",
            "description": "Give back the whole note behind a citation such as r2-a91f2c33e1bd.",
            "inputSchema": {
                "type": "object",
                "properties": { "citation": { "type": "string" } },
                "required": ["citation"],
            },
        },
        {
            "name": "write",
            "title": "Record what was learned",
            "description": "Write a note. Ring 2 is project knowledge, ring 3 a session \
                             record, ring 4 unverified material. Rings 0 and 1 are the \
                             operator's and are refused here.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ring": { "type": "integer", "description": "2, 3 or 4" },
                    "kind": { "type": "string", "description": "knowledge, bug, lesson, decision, reference or session" },
                    "name": { "type": "string", "description": "kebab-case slug, unique in the store" },
                    "body": { "type": "string", "description": "Markdown. Link other notes with [[their-name]]." },
                },
                "required": ["ring", "kind", "name", "body"],
            },
        },
        {
            "name": "status",
            "title": "What the store holds",
            "description": "Note counts per ring, the index, and the resident token budget.",
            "inputSchema": { "type": "object", "properties": {} },
        },
    ]})
}

fn error_response(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
