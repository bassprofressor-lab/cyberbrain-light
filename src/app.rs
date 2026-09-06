//! The whole of Light, minus the things that talk to the outside: the store, the index, and
//! the six commands that operate on them.
//!
//! Original work, copyright 2026 Krynex Labs, licensed FSL-1.1-ALv2.

use anyhow::{Context, Result, anyhow, bail};
use cyberbrain_core::blocks::{MAX_BLOCK_TOKENS, blocks_of};
use cyberbrain_core::links::link_targets;
use cyberbrain_core::store::Store;
use cyberbrain_core::types::{Frontmatter, Note, NoteId, NoteKind, RecallResult, Ring};
use cyberbrain_core::{Citation, frontmatter};
use cyberbrain_index::{Index, IndexStats, RecallOptions, content_hash};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// The directory a store lives in, inside a project.
pub const STORE_DIR: &str = ".cyberbrain";
/// Light's own configuration file. It sits beside the full product's `cyberbrain.toml` and
/// never touches it: a store that has been used with both should not have one tool's
/// defaults silently rewritten by the other.
pub const CONFIG_FILE: &str = "light.toml";

/// Everything Light can be configured to do. If a knob is not here, it is not a knob: the
/// point of this product is that there is nothing to set up.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LightConfig {
    /// Token cap for rings 0 and 1 together, which ride along in every session.
    pub resident_cap_tokens: usize,
    /// Hits returned by `recall` unless told otherwise.
    pub n: usize,
    /// Candidates BM25 considers before the ranking is cut to `n`.
    pub k_lex: usize,
}

impl Default for LightConfig {
    fn default() -> Self {
        Self {
            resident_cap_tokens: 8192,
            n: 8,
            k_lex: 50,
        }
    }
}

impl LightConfig {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(CONFIG_FILE);
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text)
                .with_context(|| format!("{} is not readable as configuration", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    pub fn write(&self, root: &Path) -> Result<PathBuf> {
        let path = root.join(CONFIG_FILE);
        let text = format!(
            "# Cyberbrain Light. Three settings, and you can delete the file to get the\n\
             # defaults back.\n\n\
             # Rings 0 and 1 are injected into every session; this caps what that costs.\n\
             resident_cap_tokens = {}\n\n\
             # Hits returned by `cbl recall`.\n\
             n = {}\n\n\
             # Candidates BM25 weighs before the list is cut to `n`.\n\
             k_lex = {}\n",
            self.resident_cap_tokens, self.n, self.k_lex
        );
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }
}

/// Walk up from `from` looking for a store. A command works anywhere inside a project, which
/// is the difference between a tool you use and a tool you remember to cd for.
pub fn find_store(explicit: Option<PathBuf>, from: &Path) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    for dir in from.ancestors() {
        let candidate = dir.join(STORE_DIR);
        if candidate.join("notes").is_dir() {
            return Ok(candidate);
        }
    }
    bail!(
        "no store found in {} or any directory above it; run `cbl init` in your project",
        from.display()
    )
}

pub struct App {
    pub store: Store,
    pub index: Index,
    pub cfg: LightConfig,
}

impl App {
    /// Open an existing store.
    pub fn open(root: &Path) -> Result<Self> {
        let cfg = LightConfig::load(root)?;
        let store = Store::with_cap(root, cfg.resident_cap_tokens)
            .with_context(|| format!("opening the store at {}", root.display()))?;
        let index = Index::open(&store.db_path())
            .with_context(|| format!("opening the index at {}", store.db_path().display()))?;
        Ok(Self { store, index, cfg })
    }

    /// Create the layout and the config, then open it. Running it twice is not an error and
    /// keeps whatever is already there.
    pub fn init(root: &Path, cap: usize) -> Result<(Self, bool)> {
        let existed = root.join("notes").is_dir();
        let store = Store::create(root, cap)
            .with_context(|| format!("creating the store at {}", root.display()))?;
        let cfg = LightConfig {
            resident_cap_tokens: cap,
            ..LightConfig::default()
        };
        if !root.join(CONFIG_FILE).exists() {
            cfg.write(root)?;
        }
        let index = Index::open(&store.db_path())?;
        Ok((Self { store, index, cfg }, existed))
    }

    // ----- write ---------------------------------------------------------------------

    pub fn write_note(&mut self, req: WriteRequest) -> Result<Written> {
        let name = req.name.trim().to_string();
        frontmatter::validate_name(&name).map_err(|why| anyhow!("name `{name}`: {why}"))?;

        let existing = match self.store.read(&name) {
            Ok(n) => Some(n),
            Err(cyberbrain_core::Error::NoSuchNote(_)) => None,
            Err(e) => return Err(e.into()),
        };
        if let Some(cur) = &existing
            && cur.front.ring != req.ring
        {
            // Moving a note between rings changes what it is allowed to override. That is a
            // decision, not a side effect of a `write`, so it is refused rather than done.
            bail!(
                "`{name}` already exists in ring {} and this write says ring {}; \
                 `cbl forget {name}` first if the move is what you mean",
                cur.front.ring.as_u8(),
                req.ring.as_u8()
            );
        }

        let now = jiff::Timestamp::now();
        let mut tags: Vec<String> = Vec::new();
        for t in req.tags {
            let t = t.trim().to_string();
            if !t.is_empty() && !tags.contains(&t) {
                tags.push(t);
            }
        }
        let front = Frontmatter {
            id: existing
                .as_ref()
                .map(|n| n.front.id)
                .unwrap_or_else(NoteId::generate),
            name: name.clone(),
            ring: req.ring,
            kind: req.kind,
            created: existing.as_ref().map(|n| n.front.created).unwrap_or(now),
            updated: now,
            tags,
            links: link_targets(&req.body),
            retention: None,
            pii: Default::default(),
        };
        let note = Note {
            front,
            body: req.body,
            path: PathBuf::new(),
        };
        let bytes = frontmatter::render(&note.front, &note.body)?.len();
        let path = self.store.write(&note)?;
        let note = Note { path, ..note };

        let (blocks, oversized) = blocks_of(&note, MAX_BLOCK_TOKENS);
        let outcome = self.index.upsert_note(&note, &blocks, None)?;
        Ok(Written {
            id: note.front.id,
            name,
            ring: note.front.ring,
            path: note.path,
            bytes,
            created: existing.is_none(),
            blocks: outcome.blocks,
            links: outcome.links,
            oversized: oversized.len(),
        })
    }

    // ----- scan ----------------------------------------------------------------------

    /// Rebuild the index from the notes tree. Incremental by default: a note whose content
    /// hash is unchanged is left alone. `--full` throws the index away first, which is safe
    /// because the index is derived from files you can read without this tool.
    pub fn scan(&mut self, full: bool) -> Result<ScanReport> {
        let started = std::time::Instant::now();
        if full {
            self.index.clear()?;
        }
        let listing = self.store.list()?;
        let mut report = ScanReport {
            skipped: listing
                .skipped
                .iter()
                .map(|s| format!("{}: {}", s.path.display(), s.reason))
                .collect(),
            ..Default::default()
        };

        let mut seen: HashSet<NoteId> = HashSet::new();
        for entry in &listing.entries {
            let mut note = match self.store.read_path(&entry.path) {
                Ok(n) => n,
                Err(e) => {
                    report
                        .skipped
                        .push(format!("{}: {e}", entry.path.display()));
                    continue;
                }
            };
            if !seen.insert(note.front.id) {
                report.skipped.push(format!(
                    "{}: id {} is already used by another note; ids are unique",
                    entry.path.display(),
                    note.front.id
                ));
                continue;
            }

            // Links are derived from the body, never hand-maintained, so scan writes them back.
            let targets = link_targets(&note.body);
            if targets != note.front.links {
                note.front.links = targets;
                if self.store.write(&note).is_ok() {
                    report.links_written_back += 1;
                }
            }

            let hash = content_hash(&note);
            if !full
                && let Some((known, _)) = self.index.fingerprint(&note.front.id)?
                && known == hash
            {
                report.unchanged += 1;
                continue;
            }
            let (blocks, oversized) = blocks_of(&note, MAX_BLOCK_TOKENS);
            report.oversized += oversized.len();
            let outcome = self.index.upsert_note(&note, &blocks, None)?;
            if outcome.replaced {
                report.updated += 1;
            } else {
                report.added += 1;
            }
            report.blocks += outcome.blocks;
        }

        // Notes that left the tree leave the index with them.
        for record in self.index.notes()? {
            if !seen.contains(&record.front.id) {
                self.index.delete_note(&record.front.id)?;
                report.removed += 1;
            }
        }

        report.stats = Some(self.index.stats()?);
        report.ms = started.elapsed().as_millis() as u64;
        Ok(report)
    }

    // ----- recall --------------------------------------------------------------------

    pub fn recall(&self, query: &str, n: usize, ring: Option<Ring>) -> Result<RecallResult> {
        let opts = RecallOptions {
            n,
            k_lex: self.cfg.k_lex,
            // No embedder exists in this product, so no candidates are asked for.
            k_sem: 0,
            ring,
            ..RecallOptions::default()
        };
        let mut result = self.index.recall(query, None, &opts)?;
        // The index says "no embedder configured", which is true of the store and wrong
        // about the tool: nothing here is misconfigured. Say what is actually the case.
        result.caveats.retain(|c| !c.starts_with("semantic search"));
        result.caveats.push(
            "search is lexical: it matches the words in the text, not the meaning. \
             Cyberbrain adds semantic search; Light does not have it."
                .into(),
        );
        Ok(result)
    }

    /// Expand a citation to the block it points at, plus the note it came from.
    pub fn expand(&self, citation: &str) -> Result<Expanded> {
        let cit: Citation = citation.parse().map_err(|_| {
            anyhow!("`{citation}` is not a citation; they look like r2-a91f2c33e1bd")
        })?;
        let (block, record) = self
            .index
            .resolve(&cit)?
            .ok_or_else(|| anyhow!("no block carries the citation `{citation}`"))?;
        let note = self.store.read(&record.front.name)?;
        Ok(Expanded {
            citation: cit.to_string(),
            name: record.front.name.clone(),
            ring: record.front.ring,
            block: block.text,
            body: note.body,
        })
    }

    // ----- forget --------------------------------------------------------------------

    /// Delete a note and everything derived from it. Light keeps no erasure record: that is
    /// a compliance artefact and this product does not claim to produce them.
    pub fn forget(&mut self, name: &str) -> Result<Forgotten> {
        let note = self.store.read(name)?;
        let erasure = self.index.delete_note(&note.front.id)?;
        let path = self.store.remove(name)?;
        Ok(Forgotten {
            name: name.to_string(),
            ring: note.front.ring,
            path,
            blocks: erasure.counts.blocks,
            links: erasure.counts.links_out,
        })
    }

    // ----- status and doctor ---------------------------------------------------------

    pub fn status(&self) -> Result<Status> {
        let listing = self.store.list()?;
        let mut by_ring = [0usize; 5];
        for e in &listing.entries {
            by_ring[e.ring.as_u8() as usize] += 1;
        }
        Ok(Status {
            root: self.store.root().to_path_buf(),
            notes: listing.entries.len(),
            by_ring,
            skipped: listing.skipped.len(),
            resident_tokens: self.store.resident_tokens()?,
            resident_cap: self.store.resident_cap(),
            stats: self.index.stats()?,
            db_bytes: std::fs::metadata(self.store.db_path())
                .map(|m| m.len())
                .unwrap_or(0),
        })
    }

    /// The footprint, the integrity checks, and — with `--fix` — the two repairs that are
    /// safe to make without asking: reindex what drifted, and compact the database.
    pub fn doctor(&mut self, fix: bool) -> Result<Doctor> {
        let mut findings = self.index.integrity()?;
        let listing = self.store.list()?;

        let indexed: HashSet<NoteId> = self
            .index
            .notes()?
            .into_iter()
            .map(|n| n.front.id)
            .collect();
        let mut on_disk: HashSet<NoteId> = HashSet::new();
        let mut drifted = 0usize;
        for entry in &listing.entries {
            if let Ok(note) = self.store.read_path(&entry.path) {
                on_disk.insert(note.front.id);
                match self.index.fingerprint(&note.front.id)? {
                    Some((hash, _)) if hash == content_hash(&note) => {}
                    _ => drifted += 1,
                }
            }
        }
        let orphaned = indexed.difference(&on_disk).count();
        if drifted > 0 {
            findings.push(format!("{drifted} note(s) on disk differ from the index"));
        }
        if orphaned > 0 {
            findings.push(format!(
                "{orphaned} note(s) in the index no longer exist on disk"
            ));
        }

        let mut repaired = None;
        if fix && (drifted > 0 || orphaned > 0) {
            repaired = Some(self.scan(false)?);
        }
        let before = self.db_bytes();
        if fix {
            self.index.vacuum()?;
        }
        Ok(Doctor {
            notes: listing.entries.len(),
            notes_bytes: listing
                .entries
                .iter()
                .filter_map(|e| std::fs::metadata(&e.path).ok().map(|m| m.len()))
                .sum(),
            db_bytes_before: before,
            db_bytes_after: self.db_bytes(),
            findings,
            repaired,
        })
    }

    fn db_bytes(&self) -> u64 {
        std::fs::metadata(self.store.db_path())
            .map(|m| m.len())
            .unwrap_or(0)
    }
}

// ----- request and report types ------------------------------------------------------

pub struct WriteRequest {
    pub ring: Ring,
    pub kind: NoteKind,
    pub name: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Written {
    pub id: NoteId,
    pub name: String,
    pub ring: Ring,
    #[serde(serialize_with = "cyberbrain_core::path_serde::slash")]
    pub path: PathBuf,
    pub bytes: usize,
    pub created: bool,
    pub blocks: usize,
    pub links: usize,
    pub oversized: usize,
}

#[derive(Serialize, Default)]
pub struct ScanReport {
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub removed: usize,
    pub blocks: usize,
    pub oversized: usize,
    pub links_written_back: usize,
    pub skipped: Vec<String>,
    pub ms: u64,
    pub stats: Option<IndexStats>,
}

#[derive(Serialize)]
pub struct Expanded {
    pub citation: String,
    pub name: String,
    pub ring: Ring,
    pub block: String,
    pub body: String,
}

#[derive(Serialize)]
pub struct Forgotten {
    pub name: String,
    pub ring: Ring,
    #[serde(serialize_with = "cyberbrain_core::path_serde::slash")]
    pub path: PathBuf,
    pub blocks: usize,
    pub links: usize,
}

#[derive(Serialize)]
pub struct Status {
    #[serde(serialize_with = "cyberbrain_core::path_serde::slash")]
    pub root: PathBuf,
    pub notes: usize,
    pub by_ring: [usize; 5],
    pub skipped: usize,
    pub resident_tokens: usize,
    pub resident_cap: usize,
    pub stats: IndexStats,
    pub db_bytes: u64,
}

#[derive(Serialize)]
pub struct Doctor {
    pub notes: usize,
    pub notes_bytes: u64,
    pub db_bytes_before: u64,
    pub db_bytes_after: u64,
    pub findings: Vec<String>,
    pub repaired: Option<ScanReport>,
}
