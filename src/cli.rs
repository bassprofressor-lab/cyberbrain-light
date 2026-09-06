//! The command line. Small on purpose: every command here is one a person types while
//! working, and nothing is here that only exists to configure something else.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "cbl",
    version,
    about = "Cyberbrain Light: cited, trust-tiered memory for AI coding agents",
    long_about = "Cyberbrain Light: cited, trust-tiered memory for AI coding agents.\n\n\
                  Notes are plain Markdown in your repository. Search is lexical (SQLite \
                  FTS5) and every hit carries a citation that resolves back to the block it \
                  came from. No model to place, no compliance subsystem, no network.",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Store to operate on. Defaults to `.cyberbrain` found from the working directory
    /// upwards, so a command works anywhere inside a project.
    #[arg(long, global = true, env = "CYBERBRAIN_STORE")]
    pub store: Option<PathBuf>,

    /// Machine-readable output. Every command that prints anything supports it.
    #[arg(long, global = true)]
    pub json: bool,

    /// Print less. Errors still go to stderr.
    #[arg(long, short, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Create a store in the current directory.
    Init {
        /// Token cap for the resident rings 0 and 1.
        #[arg(long, default_value_t = 8192)]
        cap: usize,
    },
    /// Write a note. Writing an existing name updates it and keeps its id.
    Write {
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=4))]
        ring: u8,
        #[arg(long)]
        kind: Kind,
        #[arg(long)]
        name: String,
        /// The note body. `-` reads it from standard input.
        #[arg(long)]
        body: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Rebuild the index from the notes tree.
    Scan {
        /// Discard the index and rebuild everything.
        #[arg(long)]
        full: bool,
    },
    /// Search the store.
    Recall {
        /// The query. Omit when using --id.
        query: Option<String>,
        /// Expand a citation to its full note instead of searching.
        #[arg(long, value_name = "CITATION")]
        id: Option<String>,
        /// How many hits to return.
        #[arg(long, short, default_value_t = 8)]
        n: usize,
        /// Restrict to exactly this ring.
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=4))]
        ring: Option<u8>,
    },
    /// Delete a note, its blocks and its index rows.
    Forget {
        /// Note name.
        name: String,
    },
    /// What the store holds and what the index knows.
    Status,
    /// Report the store's footprint and tidy it up.
    Doctor {
        /// Also compact the database and rebuild what is stale.
        #[arg(long)]
        fix: bool,
    },
    /// Answer an agent lifecycle hook. Reads the event payload on stdin.
    Hook {
        #[arg(value_enum)]
        event: HookEvent,
    },
    /// Write the hook entries into a project's `.claude/settings.json`.
    Install {
        /// Project directory. Defaults to the current one.
        #[arg(long)]
        project: Option<PathBuf>,
        /// Remove the entries instead of writing them.
        #[arg(long)]
        undo: bool,
    },
    /// Speak the Model Context Protocol over stdio.
    Mcp,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Kind {
    Knowledge,
    Bug,
    Lesson,
    Decision,
    Reference,
    Session,
}

impl From<Kind> for cyberbrain_core::types::NoteKind {
    fn from(k: Kind) -> Self {
        use cyberbrain_core::types::NoteKind as N;
        match k {
            Kind::Knowledge => N::Knowledge,
            Kind::Bug => N::Bug,
            Kind::Lesson => N::Lesson,
            Kind::Decision => N::Decision,
            Kind::Reference => N::Reference,
            Kind::Session => N::Session,
        }
    }
}

/// The two events where a memory earns its keep: the start of a session, and the moment the
/// context is about to be thrown away. The full product hooks more; Light hooks the two that
/// change what the agent knows.
#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum HookEvent {
    SessionStart,
    PreCompact,
}

impl HookEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            HookEvent::SessionStart => "session-start",
            HookEvent::PreCompact => "pre-compact",
        }
    }
}
