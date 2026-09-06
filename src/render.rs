//! Human output. Short lines, numbers where a number is the answer, and never a claim the
//! command did not check.

use crate::app::{Doctor, Expanded, Forgotten, ScanReport, Status, Written};
use cyberbrain_core::types::RecallResult;

pub fn hits_text(result: &RecallResult) -> String {
    let mut out = String::new();
    if result.hits.is_empty() {
        out.push_str("no hits\n");
    }
    let top = result.hits.first().map(|h| h.score).unwrap_or(1.0);
    for (i, h) in result.hits.iter().enumerate() {
        let share = if top > 0.0 {
            h.score / top * 100.0
        } else {
            0.0
        };
        out.push_str(&format!(
            "{}. {}  r{}  {}  ({share:.0}% of top)\n",
            i + 1,
            h.citation,
            h.ring.as_u8(),
            h.note_name,
        ));
        for line in h.text.lines().take(4) {
            out.push_str(&format!("     {line}\n"));
        }
    }
    for c in &result.conflicts {
        out.push_str(&format!("conflict: {c:?}\n"));
    }
    for c in &result.caveats {
        out.push_str(&format!("caveat: {c}\n"));
    }
    out
}

pub fn written_text(w: &Written) -> String {
    format!(
        "{} {} in ring r{} as {} ({} bytes, {} block(s), {} link(s)){}",
        if w.created { "wrote" } else { "updated" },
        w.name,
        w.ring.as_u8(),
        w.path.display(),
        w.bytes,
        w.blocks,
        w.links,
        if w.oversized > 0 {
            format!(
                "\n{} block(s) over the token limit were kept whole",
                w.oversized
            )
        } else {
            String::new()
        }
    )
}

pub fn scan_text(r: &ScanReport) -> String {
    let mut out = format!(
        "{} added, {} updated, {} unchanged, {} removed; {} block(s) in {} ms",
        r.added, r.updated, r.unchanged, r.removed, r.blocks, r.ms
    );
    if r.links_written_back > 0 {
        out.push_str(&format!(
            "\nlinks written back into {} note(s)",
            r.links_written_back
        ));
    }
    if r.oversized > 0 {
        out.push_str(&format!(
            "\n{} block(s) over the token limit were kept whole",
            r.oversized
        ));
    }
    for s in &r.skipped {
        out.push_str(&format!("\nskipped {s}"));
    }
    if let Some(st) = &r.stats {
        out.push_str(&format!(
            "\nindex now: {} notes, {} blocks, {} links ({} dangling)",
            st.notes, st.blocks, st.links, st.dangling_links
        ));
    }
    out
}

pub fn status_text(s: &Status) -> String {
    format!(
        "store: {}\n\
         notes on disk: {} (r0 {}, r1 {}, r2 {}, r3 {}, r4 {}); {} skipped\n\
         resident rings 0+1: ~{} of {} tokens\n\
         index: schema v{}, {} notes, {} blocks, {} links ({} dangling), {} KB\n\
         search: lexical only, by design. Cyberbrain is the one with a model.",
        s.root.display(),
        s.notes,
        s.by_ring[0],
        s.by_ring[1],
        s.by_ring[2],
        s.by_ring[3],
        s.by_ring[4],
        s.skipped,
        s.resident_tokens,
        s.resident_cap,
        s.stats.schema_version,
        s.stats.notes,
        s.stats.blocks,
        s.stats.links,
        s.stats.dangling_links,
        s.db_bytes / 1024,
    )
}

pub fn doctor_text(d: &Doctor) -> String {
    let mut out = format!(
        "notes: {} files, {} KB\nindex: {} KB",
        d.notes,
        d.notes_bytes / 1024,
        d.db_bytes_before / 1024
    );
    if d.db_bytes_after != d.db_bytes_before {
        out.push_str(&format!(
            " -> {} KB after compaction",
            d.db_bytes_after / 1024
        ));
    }
    if d.findings.is_empty() {
        out.push_str("\nnothing to report");
    }
    for f in &d.findings {
        out.push_str(&format!("\n- {f}"));
    }
    if let Some(r) = &d.repaired {
        out.push_str(&format!(
            "\nrepaired: {}",
            scan_text(r).replace('\n', "\n  ")
        ));
    }
    out
}

pub fn forgotten_text(f: &Forgotten) -> String {
    format!(
        "forgot {} from r{}: {} removed, {} block(s), {} link(s)",
        f.name,
        f.ring.as_u8(),
        f.path.display(),
        f.blocks,
        f.links
    )
}

pub fn expanded_text(e: &Expanded) -> String {
    format!(
        "{}  r{}  {}\n\n{}",
        e.citation,
        e.ring.as_u8(),
        e.name,
        e.body.trim()
    )
}
