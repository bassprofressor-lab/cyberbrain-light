//! Tests for the whole of Light. Every one of them runs against a real store in a temporary
//! directory: this product is small enough that there is no excuse for testing it in pieces.

use crate::app::{App, LightConfig, WriteRequest, find_store};
use crate::cli::HookEvent;
use crate::{hook, install};
use cyberbrain_core::types::{NoteKind, Ring};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn store(cap: usize) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join(".cyberbrain");
    App::init(&root, cap).expect("init");
    (dir, root)
}

fn write(app: &mut App, ring: u8, name: &str, body: &str) {
    app.write_note(WriteRequest {
        ring: Ring::try_from(ring).unwrap(),
        kind: NoteKind::Knowledge,
        name: name.into(),
        body: body.into(),
        tags: vec![],
    })
    .expect("write");
}

// ----- store and writing -------------------------------------------------------------

#[test]
fn a_note_written_is_a_note_found() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(
        &mut app,
        2,
        "pg18-moves-pgdata",
        "The postgres:18 image puts the data directory somewhere else.",
    );
    let result = app.recall("postgres data directory", 5, None).unwrap();
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].note_name, "pg18-moves-pgdata");

    // The citation on the hit resolves back to the note it came from. A hit whose citation
    // does not resolve is the one failure this product may never have.
    let expanded = app.expand(&result.hits[0].citation).unwrap();
    assert_eq!(expanded.name, "pg18-moves-pgdata");
    assert!(expanded.body.contains("postgres:18"));
}

#[test]
fn rewriting_a_name_keeps_its_id_and_its_creation_date() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "note", "first body");
    let first = app.store.read("note").unwrap().front;
    write(&mut app, 2, "note", "second body");
    let second = app.store.read("note").unwrap().front;
    assert_eq!(first.id, second.id, "the id survives an update");
    assert_eq!(first.created, second.created);
    assert!(second.updated >= first.updated);
    assert_eq!(app.index.stats().unwrap().notes, 1, "not a second note");
}

#[test]
fn moving_a_note_between_rings_is_refused_rather_than_done_quietly() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "note", "body");
    let err = app
        .write_note(WriteRequest {
            ring: Ring::Invariant,
            kind: NoteKind::Knowledge,
            name: "note".into(),
            body: "body".into(),
            tags: vec![],
        })
        .unwrap_err()
        .to_string();
    assert!(err.contains("already exists in ring 2"), "{err}");
    assert_eq!(app.store.read("note").unwrap().front.ring, Ring::Knowledge);
}

#[test]
fn a_bad_name_never_becomes_a_path() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    for bad in ["../escape", "with space", "UPPER", ""] {
        assert!(
            app.write_note(WriteRequest {
                ring: Ring::Knowledge,
                kind: NoteKind::Knowledge,
                name: bad.into(),
                body: "body".into(),
                tags: vec![],
            })
            .is_err(),
            "`{bad}` was accepted as a note name"
        );
    }
}

// ----- recall ------------------------------------------------------------------------

#[test]
fn recall_says_what_it_is_not_doing() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "note", "body about postgres");
    let result = app.recall("postgres", 5, None).unwrap();
    assert!(
        result.caveats.iter().any(|c| c.contains("lexical")),
        "a result has to say the search was literal: {:?}",
        result.caveats
    );
    assert!(
        !result.caveats.iter().any(|c| c.contains("no embedder")),
        "Light has no embedder to be missing; that caveat reads like a misconfiguration"
    );
}

#[test]
fn a_ring_filter_restricts_the_ranking() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "knowledge-note", "shared word here");
    write(&mut app, 3, "session-note", "shared word here");
    let all = app.recall("shared word", 8, None).unwrap();
    assert_eq!(all.hits.len(), 2);
    let only = app.recall("shared word", 8, Some(Ring::Session)).unwrap();
    assert_eq!(only.hits.len(), 1);
    assert_eq!(only.hits[0].note_name, "session-note");
}

#[test]
fn an_unresolvable_citation_is_an_error_and_not_an_empty_answer() {
    let (_d, root) = store(8192);
    let app = App::open(&root).unwrap();
    assert!(app.expand("not-a-citation").is_err());
    assert!(app.expand("r2-000000000000").is_err());
}

// ----- scan --------------------------------------------------------------------------

#[test]
fn scan_is_incremental_and_notices_what_left() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "one", "body one");
    write(&mut app, 2, "two", "body two [[one]]");

    let first = app.scan(false).unwrap();
    assert_eq!(
        (first.added, first.updated, first.unchanged),
        (0, 0, 2),
        "a write already indexed the note, so a scan right after has nothing to do"
    );

    std::fs::write(
        app.store.resolve("one").unwrap(),
        std::fs::read_to_string(app.store.resolve("one").unwrap())
            .unwrap()
            .replace("body one", "body one, edited by hand"),
    )
    .unwrap();
    let second = app.scan(false).unwrap();
    assert_eq!((second.updated, second.unchanged), (1, 1));

    std::fs::remove_file(app.store.resolve("one").unwrap()).unwrap();
    let third = app.scan(false).unwrap();
    assert_eq!(third.removed, 1);
    assert_eq!(app.index.stats().unwrap().notes, 1);
    assert_eq!(
        app.index.stats().unwrap().dangling_links,
        1,
        "the link from `two` now points at nothing and says so"
    );
}

#[test]
fn a_full_scan_rebuilds_from_the_files_alone() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "one", "body one");
    app.index.clear().unwrap();
    assert_eq!(app.index.stats().unwrap().notes, 0, "index emptied");
    let report = app.scan(true).unwrap();
    assert_eq!(report.added, 1);
    assert_eq!(app.recall("body one", 5, None).unwrap().hits.len(), 1);
}

#[test]
fn scan_writes_derived_links_back_into_the_file() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "target", "the target");
    let path = app.store.ring_dir(Ring::Knowledge).join("source.md");
    // A note written by hand, without the links line the tool maintains.
    std::fs::write(
        &path,
        "---\nid: 01J0000000000000000000000A\nname: source\nring: 2\nkind: knowledge\n\
         created: 2026-09-06T00:00:00Z\nupdated: 2026-09-06T00:00:00Z\n---\n\nSee [[target]].\n",
    )
    .unwrap();
    let report = app.scan(false).unwrap();
    assert_eq!(report.links_written_back, 1);
    assert!(std::fs::read_to_string(&path).unwrap().contains("- target"));
}

// ----- forget ------------------------------------------------------------------------

#[test]
fn forget_removes_the_file_and_the_index_rows() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "gone", "the body that goes away");
    let f = app.forget("gone").unwrap();
    assert_eq!(f.blocks, 1);
    assert!(!f.path.exists(), "the file is gone");
    assert_eq!(app.index.stats().unwrap().notes, 0);
    assert!(
        app.recall("the body that goes away", 5, None)
            .unwrap()
            .hits
            .is_empty(),
        "a forgotten note is not findable"
    );
    assert!(app.forget("gone").is_err(), "forgetting twice is an error");
}

// ----- doctor ------------------------------------------------------------------------

#[test]
fn doctor_reports_drift_and_fix_repairs_it() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "note", "body");
    assert!(
        app.doctor(false).unwrap().findings.is_empty(),
        "a healthy store reports nothing"
    );

    // Break it the way a person does: edit the file behind the tool's back.
    let path = app.store.resolve("note").unwrap();
    std::fs::write(
        &path,
        std::fs::read_to_string(&path).unwrap() + "\nedited by hand\n",
    )
    .unwrap();
    let broken = app.doctor(false).unwrap();
    assert!(
        broken.findings.iter().any(|f| f.contains("differ")),
        "{:?}",
        broken.findings
    );

    let fixed = app.doctor(true).unwrap();
    assert!(fixed.repaired.is_some());
    assert!(
        app.doctor(false).unwrap().findings.is_empty(),
        "and stays fixed"
    );
}

// ----- hooks -------------------------------------------------------------------------

#[test]
fn session_start_injects_the_resident_rings_with_citations() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 0, "invariants", "Never deploy on a Friday.");
    write(
        &mut app,
        2,
        "knowledge",
        "Not resident, must not be injected.",
    );
    drop(app);

    let out = hook::run(HookEvent::SessionStart, &root, "{}").unwrap();
    let v: Value = serde_json::from_str(&out.stdout).unwrap();
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(ctx.contains("Never deploy on a Friday."));
    assert!(!ctx.contains("Not resident"), "ring 2 is not injected");
    assert!(ctx.contains("[r0-"), "every block carries its citation");
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "SessionStart");
}

#[test]
fn the_resident_cap_holds_and_is_reported() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 0, "big", &"word ".repeat(4000));
    drop(app);

    // Same store, a cap too small for the note: it is left out and the human is told.
    let cfg = LightConfig {
        resident_cap_tokens: 10,
        ..LightConfig::default()
    };
    cfg.write(&root).unwrap();
    let out = hook::run(HookEvent::SessionStart, &root, "{}").unwrap();
    assert!(
        out.stdout.is_empty(),
        "nothing fitted, so nothing is injected"
    );
    assert!(
        out.notes.iter().any(|n| n.contains("resident cap")),
        "{:?}",
        out.notes
    );
}

#[test]
fn a_hook_survives_a_payload_that_is_not_json() {
    let (_d, root) = store(8192);
    let out = hook::run(HookEvent::SessionStart, &root, "not json at all").unwrap();
    assert!(out.notes.iter().any(|n| n.contains("not JSON")));
}

#[test]
fn a_hook_on_a_directory_that_is_not_a_store_says_so_and_carries_on() {
    let dir = tempfile::tempdir().unwrap();
    let out = hook::run(HookEvent::SessionStart, dir.path(), "{}").unwrap();
    assert!(out.stdout.is_empty());
    assert!(out.notes.iter().any(|n| n.contains("no usable store")));
}

#[test]
fn pre_compact_addresses_the_human_and_names_the_count() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "one", "body");
    drop(app);
    let out = hook::run(HookEvent::PreCompact, &root, r#"{"trigger":"auto"}"#).unwrap();
    let v: Value = serde_json::from_str(&out.stdout).unwrap();
    let msg = v["systemMessage"].as_str().unwrap();
    assert!(msg.contains("1 note(s)"), "{msg}");
}

// ----- install -----------------------------------------------------------------------

#[test]
fn install_keeps_what_it_did_not_write_and_undoes_only_its_own() {
    let dir = tempfile::tempdir().unwrap();
    let settings = dir.path().join(".claude/settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(
        &settings,
        r#"{"permissions":{"allow":["Bash(ls:*)"]},
            "hooks":{"SessionStart":[{"matcher":"","hooks":[{"type":"command","command":"/usr/bin/true"}]}]}}"#,
    )
    .unwrap();

    let r = install::run(dir.path(), false).unwrap();
    assert_eq!(r.added.len(), 2);
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    assert_eq!(v["permissions"]["allow"][0], "Bash(ls:*)");
    assert_eq!(
        v["hooks"]["SessionStart"].as_array().unwrap().len(),
        2,
        "the foreign SessionStart hook is still there next to ours"
    );

    // Running it twice changes nothing.
    let again = install::run(dir.path(), false).unwrap();
    assert!(again.added.is_empty());
    assert_eq!(again.unchanged.len(), 2);

    let undone = install::run(dir.path(), true).unwrap();
    assert_eq!(undone.removed.len(), 2);
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    assert_eq!(
        v["hooks"]["SessionStart"].as_array().unwrap().len(),
        1,
        "the foreign hook survives the undo"
    );
    assert!(v["hooks"]["PreCompact"].is_null());
}

#[test]
fn install_refuses_a_settings_file_it_cannot_parse() {
    let dir = tempfile::tempdir().unwrap();
    let settings = dir.path().join(".claude/settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(&settings, "{ this is not json").unwrap();
    let err = install::run(dir.path(), false).unwrap_err().to_string();
    assert!(err.contains("not valid JSON"), "{err}");
    assert_eq!(
        std::fs::read_to_string(&settings).unwrap(),
        "{ this is not json",
        "and leaves it exactly as it was"
    );
}

// ----- config and store discovery -----------------------------------------------------

#[test]
fn a_missing_config_is_the_default_and_a_wrong_one_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        LightConfig::load(dir.path()).unwrap().resident_cap_tokens,
        8192
    );
    std::fs::write(dir.path().join(crate::app::CONFIG_FILE), "nonsense = 1\n").unwrap();
    assert!(
        LightConfig::load(dir.path()).is_err(),
        "an unknown key is a typo, not a feature"
    );
}

#[test]
fn a_command_works_from_a_subdirectory() {
    let (dir, root) = store(8192);
    let deep = dir.path().join("src/deeply/nested");
    std::fs::create_dir_all(&deep).unwrap();
    assert_eq!(find_store(None, &deep).unwrap(), root);

    let elsewhere = tempfile::tempdir().unwrap();
    assert!(find_store(None, elsewhere.path()).is_err());
    assert_eq!(
        find_store(Some(PathBuf::from("/explicit")), Path::new("/tmp")).unwrap(),
        PathBuf::from("/explicit"),
        "--store wins over the search"
    );
}

#[test]
fn init_twice_keeps_the_notes_and_the_config() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(".cyberbrain");
    let (mut app, existed) = App::init(&root, 1234).unwrap();
    assert!(!existed);
    write(&mut app, 2, "kept", "body");
    drop(app);

    let (app, existed) = App::init(&root, 9999).unwrap();
    assert!(existed, "the second init sees the store that is there");
    assert!(app.store.read("kept").is_ok());
    assert_eq!(
        LightConfig::load(&root).unwrap().resident_cap_tokens,
        1234,
        "and does not overwrite a config the user may have edited"
    );
}

// ----- mcp ---------------------------------------------------------------------------

fn rpc(server: &mut crate::mcp::Server, line: &str) -> Option<Value> {
    server
        .handle_line(line)
        .map(|r| serde_json::from_str(&r).expect("a response is JSON"))
}

fn mcp_server(root: &Path) -> crate::mcp::Server {
    crate::mcp::Server {
        store_root: root.to_path_buf(),
    }
}

#[test]
fn mcp_answers_the_handshake_and_lists_four_tools() {
    let (_d, root) = store(8192);
    let mut s = mcp_server(&root);
    let init = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#,
    )
    .unwrap();
    assert_eq!(
        init["result"]["protocolVersion"], "2025-03-26",
        "a supported version the client asked for is the one it gets"
    );
    assert_eq!(init["result"]["serverInfo"]["name"], "cyberbrain-light");

    let asked_nonsense = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#,
    )
    .unwrap();
    assert_eq!(
        asked_nonsense["result"]["protocolVersion"],
        crate::mcp::LATEST_PROTOCOL,
        "an unknown version is answered with what we do speak, not with an error"
    );

    let tools = rpc(&mut s, r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#).unwrap();
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["recall", "recall_id", "write", "status"]);
}

#[test]
fn mcp_notifications_get_no_answer_and_unknown_methods_get_an_error() {
    let (_d, root) = store(8192);
    let mut s = mcp_server(&root);
    assert!(
        rpc(
            &mut s,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
        )
        .is_none(),
        "a notification has no id and must not be answered"
    );
    let err = rpc(&mut s, r#"{"jsonrpc":"2.0","id":9,"method":"nope"}"#).unwrap();
    assert_eq!(err["error"]["code"], -32601);
    let parse = rpc(&mut s, "{not json").unwrap();
    assert_eq!(parse["error"]["code"], -32700);
}

#[test]
fn mcp_writes_and_reads_the_same_store_and_refuses_the_operator_rings() {
    let (_d, root) = store(8192);
    let mut s = mcp_server(&root);
    let wrote = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"write","arguments":
            {"ring":2,"kind":"lesson","name":"from-the-agent","body":"A lesson learned in a session."}}}"#,
    )
    .unwrap();
    assert_eq!(wrote["result"]["isError"], false);

    let found = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"recall","arguments":
            {"query":"lesson learned"}}}"#,
    )
    .unwrap();
    let text = found["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("from-the-agent"), "{text}");

    let refused = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"write","arguments":
            {"ring":0,"kind":"decision","name":"sneaky","body":"An invariant I gave myself."}}}"#,
    )
    .unwrap();
    assert_eq!(
        refused["result"]["isError"], true,
        "an agent may not write ring 0"
    );
    assert!(
        !root.join("notes/r0/sneaky.md").exists(),
        "and nothing was written"
    );

    // A failing tool is reported to the model, not to the transport: the session goes on.
    let bad = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"recall_id","arguments":
            {"citation":"r2-000000000000"}}}"#,
    )
    .unwrap();
    assert_eq!(bad["result"]["isError"], true);
    assert!(bad["error"].is_null());
}
