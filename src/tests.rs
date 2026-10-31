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

/// A store can be shared with the full Cyberbrain, which knows fields Light does not edit:
/// a retention period, a department, a supersession, a validity window. An edit with
/// `cbl write` must keep them. Before 0.2.0 it set `retention` to nothing on every edit.
#[test]
fn an_edit_keeps_the_fields_light_does_not_edit() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "pg18", "first body");
    let mut note = app.store.read("pg18").unwrap();
    let t = jiff::Timestamp::from_second(1_790_000_000).unwrap();
    note.front.retention = Some("P2Y".into());
    note.front.bereich = Some("betrieb".into());
    note.front.supersedes = vec!["pg17".into()];
    note.front.valid_from = Some(t);
    app.store.write(&note).unwrap();

    write(&mut app, 2, "pg18", "second body");
    let after = app.store.read("pg18").unwrap();
    assert!(after.body.contains("second body"));
    assert_eq!(after.front.retention.as_deref(), Some("P2Y"));
    assert_eq!(after.front.bereich.as_deref(), Some("betrieb"));
    assert_eq!(after.front.supersedes, vec!["pg17".to_string()]);
    assert_eq!(after.front.valid_from, Some(t));
}

/// L2 (2026-10-31): tags carry provenance (`trust:untrusted`, `src:…`), and recall marks a hit
/// untrusted from them. An edit without `--tags`, and every MCP `write`, set them to nothing,
/// so one overwrite — also by the model — laundered a note marked as from outside.
#[test]
fn an_edit_keeps_the_tags_and_never_drops_the_provenance() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    let req = |body: &str, tags: &[&str]| WriteRequest {
        ring: Ring::External,
        kind: NoteKind::Knowledge,
        name: "import-mail".into(),
        body: body.into(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
    };
    app.write_note(req(
        "Rechnung mit neuer IBAN.",
        &["trust:untrusted", "src:mail:4f2a", "rechnung"],
    ))
    .unwrap();

    // No tags given: everything stays.
    app.write_note(req("Rechnung mit neuer IBAN. (Nachtrag)", &[]))
        .unwrap();
    let tags = app.store.read("import-mail").unwrap().front.tags;
    assert_eq!(tags, ["trust:untrusted", "src:mail:4f2a", "rechnung"]);

    // Tags given: they replace the others, but not the provenance, and `trust:trusted` does
    // not undo an untrusted mark.
    app.write_note(req("Rechnung.", &["trust:trusted", "zahlung"]))
        .unwrap();
    let tags = app.store.read("import-mail").unwrap().front.tags;
    assert!(tags.contains(&"zahlung".to_string()), "{tags:?}");
    assert!(!tags.contains(&"rechnung".to_string()), "{tags:?}");
    assert!(!tags.contains(&"trust:trusted".to_string()), "{tags:?}");
    assert!(tags.contains(&"trust:untrusted".to_string()), "{tags:?}");
    assert!(tags.contains(&"src:mail:4f2a".to_string()), "{tags:?}");

    // The MCP `write` has no tags at all; it must not take them away either.
    let mut s = mcp_server(&root);
    let w = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"write","arguments":
            {"ring":4,"kind":"knowledge","name":"import-mail","body":"Rechnung, jetzt sauber."}}}"#,
    )
    .unwrap();
    assert_eq!(w["result"]["isError"], false, "{w}");
    let app = App::open(&root).unwrap();
    let tags = app.store.read("import-mail").unwrap().front.tags;
    assert!(tags.contains(&"trust:untrusted".to_string()), "{tags:?}");
    let hits = app.recall("rechnung sauber", 5, None).unwrap().hits;
    assert_eq!(hits.len(), 1);
    assert!(
        hits[0].untrusted.is_some(),
        "recall still says where it came from"
    );
}

/// L8 (2026-10-31): two writes of the same new name at once both saw no note, both wrote the
/// file, and the second then failed on the index's name check — after its file had replaced
/// the first one's. File and index disagreed, and the one that "failed" was the one on disk.
#[test]
fn concurrent_writes_of_one_new_name_leave_file_and_index_agreeing() {
    let (_d, root) = store(8192);
    for round in 0..10 {
        let name = format!("race-{round}");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(6));
        let handles: Vec<_> = (0..6)
            .map(|i| {
                let (root, name, barrier) = (root.clone(), name.clone(), barrier.clone());
                std::thread::spawn(move || {
                    let mut app = App::open(&root).unwrap();
                    barrier.wait();
                    app.write_note(WriteRequest {
                        ring: Ring::Knowledge,
                        kind: NoteKind::Knowledge,
                        name,
                        body: format!("body {i}"),
                        tags: vec![],
                    })
                    .map(|w| w.created)
                    .map_err(|e| e.to_string())
                })
            })
            .collect();
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(
            results.iter().all(Result::is_ok),
            "round {round}: {results:?}"
        );
        assert_eq!(
            results.iter().filter(|r| *r == &Ok(true)).count(),
            1,
            "round {round}: one created it, the others updated it: {results:?}"
        );
    }
    let mut app = App::open(&root).unwrap();
    let d = app.doctor(false).unwrap();
    assert!(d.findings.is_empty(), "{:?}", d.findings);
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

/// L3 (2026-10-31): a note `read_path` refuses — a typo in its header, a field of a newer
/// Cyberbrain — was dropped from the index although the file is still there, so "skipped"
/// meant "deleted". A note hidden by `.cyberbrainignore` leaves the index, but is said to be
/// ignored, not to be gone.
#[test]
fn scan_keeps_an_unreadable_note_and_names_an_ignored_one() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 2, "wichtig", "postgres läuft auf Port 5432.");
    write(&mut app, 2, "altes-protokoll", "überholter stand");

    let path = app.store.resolve("wichtig").unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replacen("ring: 2", "ring: 2\nquelle: handbuch", 1),
    )
    .unwrap();
    assert!(
        app.store.read_path(&path).is_err(),
        "the header is now unreadable"
    );
    std::fs::write(root.join(".cyberbrainignore"), "altes-protokoll.md\n").unwrap();

    let report = app.scan(false).unwrap();
    assert_eq!(report.removed, 0, "nothing left the tree");
    assert_eq!(report.ignored, 1, "{:?}", report.skipped);
    assert!(
        report
            .skipped
            .iter()
            .any(|s| s.contains("wichtig") && s.contains("kept")),
        "{:?}",
        report.skipped
    );
    assert!(path.is_file());
    assert_eq!(
        app.recall("postgres", 5, None).unwrap().hits.len(),
        1,
        "the unreadable note is still found"
    );
    assert!(app.recall("überholter", 5, None).unwrap().hits.is_empty());

    // doctor says what is the case, and --fix does not make it worse.
    let d = app.doctor(true).unwrap();
    assert!(
        d.findings.iter().any(|f| f.contains("cannot be read")),
        "{:?}",
        d.findings
    );
    assert!(
        !d.findings.iter().any(|f| f.contains("no longer exist")),
        "{:?}",
        d.findings
    );
    assert_eq!(app.recall("postgres", 5, None).unwrap().hits.len(), 1);
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

/// L4 (2026-10-31): the hook read notes/r0 and notes/r1 straight from the directory, past
/// `.cyberbrainignore`. A resident note the operator hid — overruled, so hidden — was still
/// injected into every session while `status` said it was gone.
#[test]
fn session_start_honours_cyberbrainignore() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 1, "neues-protokoll", "Deploy only from main.");
    write(
        &mut app,
        1,
        "altes-protokoll",
        "ALTES PROTOKOLL: deploy from anywhere.",
    );
    std::fs::write(root.join(".cyberbrainignore"), "altes-protokoll.md\n").unwrap();
    assert_eq!(app.status().unwrap().by_ring[1], 1, "status sees one");
    drop(app);

    let out = hook::run(HookEvent::SessionStart, &root, "{}").unwrap();
    let v: Value = serde_json::from_str(&out.stdout).unwrap();
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(ctx.contains("Deploy only from main."));
    assert!(!ctx.contains("ALTES PROTOKOLL"), "{ctx}");
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

/// 0.2.0: under the token cap is not enough. Claude Code shows `additionalContext` in full up
/// to 10,000 characters and a 2,000 character preview past that, so the rings stop at the
/// character limit, and what was left out is named where the model reads it.
#[test]
fn session_start_stays_under_the_characters_claude_code_shows() {
    let (_d, root) = store(100_000);
    let mut app = App::open(&root).unwrap();
    for i in 0..8 {
        write(
            &mut app,
            1,
            &format!("protocol-{i}"),
            &format!("Step {i}: {}", "keep it short. ".repeat(110)),
        );
    }
    drop(app);
    let out = hook::run(HookEvent::SessionStart, &root, "{}").unwrap();
    let v: Value = serde_json::from_str(&out.stdout).unwrap();
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        ctx.chars().count() <= 10_000,
        "{} characters",
        ctx.chars().count()
    );
    assert!(
        ctx.contains("## r1 `protocol-0`"),
        "the first notes are there"
    );
    assert!(
        ctx.contains("Not shown") && ctx.contains("r1/protocol-7"),
        "{ctx}"
    );
    assert!(
        out.notes.iter().any(|n| n.contains("did not fit")),
        "{:?}",
        out.notes
    );
}

/// L5 (2026-10-31): the closing line that names what did not fit was appended after the
/// character check and not counted. One large note just under the cap and eight left-out
/// notes with long names took the context to 10,373 characters, past what Claude Code shows.
#[test]
fn the_closing_line_counts_against_the_character_cap() {
    let (_d, root) = store(100_000);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 0, "gross", &"x".repeat(8_850));
    for i in 0..8 {
        let name = format!("protokoll-{i}-{}", "lang".repeat(25));
        assert!(name.len() >= 110);
        write(
            &mut app,
            1,
            &name,
            &format!("Schritt {i}. {}", "kurz. ".repeat(40)),
        );
    }
    drop(app);
    let out = hook::run(HookEvent::SessionStart, &root, "{}").unwrap();
    let v: Value = serde_json::from_str(&out.stdout).unwrap();
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        ctx.chars().count() <= 10_000,
        "{} characters",
        ctx.chars().count()
    );
    assert!(
        ctx.contains("Not shown"),
        "and it still says what is missing"
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
    assert_eq!(r.added.len(), 1);
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
    assert_eq!(again.unchanged.len(), 1);

    let undone = install::run(dir.path(), true).unwrap();
    assert_eq!(undone.removed.len(), 1);
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    assert_eq!(
        v["hooks"]["SessionStart"].as_array().unwrap().len(),
        1,
        "the foreign hook survives the undo"
    );
    assert!(v["hooks"]["PreCompact"].is_null());
}

/// 0.2.0: the PreCompact entry did nothing (Claude Code discards that event's
/// `systemMessage`). A run takes out the one an older version wrote, leaves a foreign one,
/// and writes a timeout on the SessionStart entry so a stuck hook cannot hold the session
/// for the harness default of 600 s.
#[test]
fn install_retires_its_old_pre_compact_entry_and_sets_a_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let settings = dir.path().join(".claude/settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(
        &settings,
        r#"{"hooks":{"PreCompact":[
            {"matcher":"","hooks":[{"type":"command","command":"/old/cbl","args":["hook","pre-compact"]}],"_managedBy":"cyberbrain-light"},
            {"matcher":"","hooks":[{"type":"command","command":"/theirs"}]}]}}"#,
    )
    .unwrap();
    let r = install::run(dir.path(), false).unwrap();
    assert!(
        r.removed.contains(&"PreCompact".to_string()),
        "{:?}",
        r.removed
    );
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    let pc = v["hooks"]["PreCompact"].as_array().unwrap();
    assert_eq!(pc.len(), 1);
    assert_eq!(pc[0]["hooks"][0]["command"], "/theirs");
    assert_eq!(
        v["hooks"]["SessionStart"][0]["hooks"][0]["timeout"],
        install::HOOK_TIMEOUT_SECS
    );
}

/// L7 (2026-10-31): `install` removed every empty hook list, also ones it never wrote, and
/// sorted the user's keys alphabetically (serde_json without `preserve_order`).
#[test]
fn install_leaves_foreign_empty_lists_and_the_key_order_alone() {
    let dir = tempfile::tempdir().unwrap();
    let settings = dir.path().join(".claude/settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(
        &settings,
        r#"{"permissions":{"allow":[]},"model":"opus","hooks":{"PreToolUse":[],
            "SessionStart":[{"matcher":"","hooks":[{"type":"command","command":"/theirs"}]}]}}"#,
    )
    .unwrap();
    install::run(dir.path(), false).unwrap();
    let text = std::fs::read_to_string(&settings).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    assert!(v["hooks"]["PreToolUse"].is_array(), "{text}");
    let at = |k: &str| text.find(&format!("\"{k}\"")).unwrap();
    assert!(
        at("permissions") < at("model") && at("model") < at("hooks"),
        "{text}"
    );
    assert!(at("PreToolUse") < at("SessionStart"), "{text}");

    install::run(dir.path(), true).unwrap();
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    assert!(v["hooks"]["PreToolUse"].is_array(), "survives the undo too");
}

/// L7: `--undo` wrote the file even when it removed nothing, and every write replaced
/// `settings.json.bak`, so install then undo left the state before the undo as the only
/// backup and the original was gone.
#[test]
fn install_writes_only_what_changed_and_never_overwrites_a_backup() {
    let dir = tempfile::tempdir().unwrap();
    let claude = dir.path().join(".claude");
    let settings = claude.join("settings.json");
    std::fs::create_dir_all(&claude).unwrap();
    let original = "{\n  \"model\": \"opus\"\n}\n";
    std::fs::write(&settings, original).unwrap();

    let r = install::run(dir.path(), false).unwrap();
    assert!(r.backup.is_some());
    let undone = install::run(dir.path(), true).unwrap();
    assert_eq!(undone.removed.len(), 1);
    assert_eq!(
        std::fs::read_to_string(claude.join("settings.json.bak")).unwrap(),
        original,
        "the backup of the original survives the undo"
    );

    let files = |d: &Path| std::fs::read_dir(d).unwrap().count();
    let before = (std::fs::read_to_string(&settings).unwrap(), files(&claude));
    let again = install::run(dir.path(), true).unwrap();
    assert!(again.removed.is_empty());
    assert!(again.backup.is_none(), "nothing changed, nothing backed up");
    assert_eq!(
        (std::fs::read_to_string(&settings).unwrap(), files(&claude)),
        before,
        "an undo with nothing to undo touches nothing"
    );
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

fn modern(id: u32, method: &str, version: &str, extra: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"{method}","params":{{{extra}"_meta":{{"io.modelcontextprotocol/protocolVersion":"{version}","io.modelcontextprotocol/clientInfo":{{"name":"claude-code","version":"2.1.309"}},"io.modelcontextprotocol/clientCapabilities":{{}}}}}}}}"#
    )
}

/// MCP 2026-07-28 (0.2.0): Claude Code 2.1.292+ probes `server/discover` first and stays
/// stateless when it answers; no `initialize` at all.
#[test]
fn mcp_speaks_2026_07_28_without_a_handshake() {
    let (_d, root) = store(8192);
    let mut s = mcp_server(&root);
    let d = rpc(&mut s, &modern(1, "server/discover", "2026-07-28", "")).unwrap();
    let r = &d["result"];
    assert_eq!(r["resultType"], "complete", "{d}");
    assert!(
        r["supportedVersions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "2026-07-28")
    );
    assert!(
        r["supportedVersions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "2025-11-25")
    );
    assert_eq!(
        r["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
        "cyberbrain-light"
    );

    let l = rpc(&mut s, &modern(2, "tools/list", "2026-07-28", "")).unwrap();
    assert_eq!(l["result"]["resultType"], "complete");
    assert!(l["result"]["ttlMs"].as_u64().unwrap() > 0);
    assert_eq!(l["result"]["cacheScope"], "public");

    let c = rpc(
        &mut s,
        &modern(
            3,
            "tools/call",
            "2026-07-28",
            r#""name":"status","arguments":{},"#,
        ),
    )
    .unwrap();
    assert_eq!(c["result"]["resultType"], "complete", "{c}");
    assert_eq!(c["result"]["isError"], false, "{c}");

    let bad = rpc(&mut s, &modern(4, "tools/list", "2099-01-01", "")).unwrap();
    assert_eq!(bad["error"]["code"], -32022, "{bad}");
    assert_eq!(bad["error"]["data"]["requested"], "2099-01-01");

    // The fallback Claude Code uses when the probe fails is answered in kind.
    let init = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":5,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}"#,
    )
    .unwrap();
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
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

/// L6 (2026-10-31): `ring` was cut with `as u8`, so 258 was ring 2 and 259 ring 3. A ring the
/// model did not mean is an error, not a guess.
#[test]
fn mcp_refuses_a_ring_out_of_range_instead_of_wrapping_it() {
    let (_d, root) = store(8192);
    let mut s = mcp_server(&root);
    let w = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"write","arguments":
            {"ring":258,"kind":"knowledge","name":"wrap-test","body":"wrapped around"}}}"#,
    )
    .unwrap();
    assert_eq!(w["result"]["isError"], true, "{w}");
    assert!(!root.join("notes/r2/wrap-test.md").exists());

    let r = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"recall","arguments":
            {"query":"anything","ring":259}}}"#,
    )
    .unwrap();
    assert_eq!(r["result"]["isError"], true, "{r}");
    let text = r["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("0 to 4"), "{text}");
}

/// L12 (2026-10-31): the MCP `recall` says "keyword search over rings 2 to 4", and the
/// session-start text says the same of `cbl recall`, but both answered from rings 0 and 1
/// too — a second copy of what the agent already holds, ranked first. As in the full product:
/// an agent gets rings 2 to 4 and a caveat that says so, `ring` 0 or 1 still searches them.
#[test]
fn mcp_recall_searches_rings_2_to_4_unless_asked_for_a_resident_ring() {
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(
        &mut app,
        0,
        "wolfpack-regel",
        "wolfpack bleibt abgeschaltet.",
    );
    write(
        &mut app,
        2,
        "wolfpack-stand",
        "wolfpack wurde abgeschaltet.",
    );
    drop(app);
    let mut s = mcp_server(&root);
    let r = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"recall","arguments":
            {"query":"wolfpack abgeschaltet"}}}"#,
    )
    .unwrap();
    let text = r["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("wolfpack-stand"), "{text}");
    assert!(!text.contains("wolfpack-regel"), "{text}");
    assert!(text.contains("rings 0/1"), "and says why: {text}");

    let r0 = rpc(
        &mut s,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"recall","arguments":
            {"query":"wolfpack abgeschaltet","ring":0}}}"#,
    )
    .unwrap();
    let text = r0["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("wolfpack-regel"), "{text}");
}

#[test]
fn an_agent_at_the_command_line_is_told_by_its_environment() {
    use crate::app::agent_from_env;
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.to_string())
        }
    };
    assert!(agent_from_env(env(&[("CLAUDECODE", "1")])));
    assert!(agent_from_env(env(&[("CYBERBRAIN_AGENT", "seo")])));
    assert!(!agent_from_env(env(&[("CLAUDECODE", " ")])));
    assert!(!agent_from_env(env(&[])));

    // A person at the terminal still gets ring 0; an agent gets ring 2 and the caveat.
    let (_d, root) = store(8192);
    let mut app = App::open(&root).unwrap();
    write(&mut app, 0, "regel", "freitags kein deploy");
    write(&mut app, 2, "stand", "freitags kein deploy seit märz");
    assert_eq!(
        app.recall("freitags deploy", 8, None).unwrap().hits.len(),
        2
    );
    app.agent = true;
    let r = app.recall("freitags deploy", 8, None).unwrap();
    assert_eq!(r.hits.len(), 1);
    assert_eq!(r.hits[0].note_name, "stand");
    assert_eq!(
        app.recall("freitags deploy", 8, Some(Ring::Invariant))
            .unwrap()
            .hits
            .len(),
        1
    );
}

// ----- workflows ----------------------------------------------------------------------

/// The workflows, read as text. `None` where they do not exist, which is the crate as
/// published: it ships `src/` and not `.github/`.
fn workflow(name: &str) -> Option<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".github/workflows")
        .join(name);
    std::fs::read_to_string(path).ok()
}

/// L9 (2026-10-31): actions were referenced by tag (`@v7`, `@stable`), which whoever controls
/// the action can move, and the release workflow gave its build job — the one running those
/// actions — a token that writes to the repository.
#[test]
fn every_action_is_pinned_and_only_publish_may_write() {
    for name in ["ci.yml", "release.yml"] {
        let Some(text) = workflow(name) else { continue };
        for line in text
            .lines()
            .filter(|l| l.trim_start().starts_with("- uses:"))
        {
            let reference = line
                .split('@')
                .nth(1)
                .and_then(|r| r.split_whitespace().next())
                .unwrap_or("");
            assert!(
                reference.len() == 40 && reference.chars().all(|c| c.is_ascii_hexdigit()),
                "{name}: not pinned to a commit: {}",
                line.trim()
            );
        }
        for (i, _) in text.match_indices("contents: write") {
            let job = text[..i]
                .lines()
                .rev()
                .find(|l| l.starts_with("  ") && !l.starts_with("   ") && l.ends_with(':'))
                .unwrap_or("");
            assert_eq!(
                job.trim(),
                "publish:",
                "{name}: a write token outside publish"
            );
        }
    }
}
