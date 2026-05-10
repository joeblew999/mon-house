use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod build;
mod chunks;
mod config;
mod gen;
mod http;
mod fonts;
mod idempotency;
mod includes;
mod new;
#[cfg(feature = "container")]
mod serve;
mod themes;
mod translate;
mod vfs;
#[cfg(feature = "local")]
mod watch;

pub use config::Config;

#[derive(Parser)]
#[command(name = "quick-tool", about = "Font management and markdown translation for the quick/ spec pipeline")]
struct Cli {
    #[command(flatten)]
    config: Config,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Font management (download, test, search)
    Fonts {
        #[command(subcommand)]
        cmd: FontsCmd,
    },
    /// Translate EN spec file(s) to Thai (skips unchanged files)
    Translate {
        /// Specific file(s) to translate. If omitted, translates all [A-Z]*.md files.
        files: Vec<PathBuf>,
    },
    /// Build EN + Thai PDFs. No idempotency — mise sources/outputs handles that.
    Build {
        /// Spec stem to build (e.g. GATE). Omit to build all specs.
        name: Option<String>,
    },
    /// Translate then build a single spec (combine of translate + build for one file).
    /// Usage: mise run one -- GATE  →  quick-tool one GATE
    One {
        /// Spec stem (e.g. GATE — without .md extension)
        name: String,
    },
    /// HTTP compile server — runs inside the CF Container alongside the PipelineAgent DO.
    /// Accepts POST /compile {name, content} → PDF bytes.  GET /health → liveness probe.
    #[cfg(feature = "container")]
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Run all `data/*.nu` generators to refresh `_partials/*.md` quantity
    /// tables from `data/*.json`. Idempotent — second run with no input
    /// change writes zero files. Used by `mise run gen` and by `watch`.
    Gen,
    /// Create a new spec file from TEMPLATE.md
    New {
        /// Spec name in UPPER CASE (e.g. DECK)
        name: String,
    },
    /// Remove the out/ directory
    Clean,
    /// Print the BLAKE3 hex hash of a file's bytes (handy for inspecting `.th.md.cache.json` keys)
    Hash {
        /// File to hash
        path: PathBuf,
    },
    /// Theme registry — list, switch, test, and check Typst themes
    Themes {
        #[command(subcommand)]
        cmd: ThemesCmd,
    },
    /// Watch specs/, scripts/, and resources/images/ for changes; on every save
    /// run translate + build for the affected files. Local-only.
    #[cfg(feature = "local")]
    Watch,
}

#[derive(Subcommand)]
enum ThemesCmd {
    /// List all available themes from registry.toml, showing which is active
    List,
    /// Show the currently active theme
    Current,
    /// Switch active theme (rewrites scripts/theme.typ)
    Switch {
        /// Theme name (e.g. minimal, compact, default)
        name: String,
    },
    /// Compile a test PDF with the current (or named) theme
    Test {
        /// Theme name to test. If omitted, tests the active theme.
        #[arg(long)]
        name: Option<String>,
        /// Test ALL themes in the registry
        #[arg(long)]
        all: bool,
    },
    /// Check theme wrapper, registry entry, and compile health
    Check,
}

#[derive(Subcommand)]
enum FontsCmd {
    /// Download fonts declared in theme-file into font-dir
    Download,
    /// Health check: files present, typst can load them, stamp valid
    Test,
    /// Verify all three idempotency layers behave correctly
    Idempotency,
    /// Run test then idempotency checks in sequence (cross-platform; used by mise fonts:test)
    TestAll,
    /// Search GWFH registry by font name
    Search {
        /// Font name to search for (e.g. "noto sans thai")
        query: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let cfg = cli.config;
    let result: Result<()> = match cli.command {
        Commands::Fonts { cmd } => match cmd {
            FontsCmd::Download => fonts::cmd_download(&cfg),
            FontsCmd::Test => fonts::cmd_test(&cfg),
            FontsCmd::Idempotency => fonts::cmd_idempotency(&cfg),
            FontsCmd::TestAll => fonts::cmd_test(&cfg)
                .and_then(|_| fonts::cmd_idempotency(&cfg)),
            FontsCmd::Search { query } => fonts::cmd_search(&cfg, &query.join(" ")),
        },
        Commands::Translate { files } => translate::cmd_translate(&cfg, files),
        Commands::Build { name } => build::cmd_build(&cfg, name),
        Commands::One { name } => {
            let path = cfg.specs_dir.join(format!("{name}.md"));
            translate::cmd_translate(&cfg, vec![path])
                .and_then(|_| build::cmd_build(&cfg, Some(name)))
        }
        #[cfg(feature = "container")]
        Commands::Serve { port } => serve::cmd_serve(&cfg, port),
        Commands::Gen => gen::cmd_gen(&cfg),
        Commands::New { name } => new::cmd_new(&cfg, &name),
        Commands::Clean => build::cmd_clean(&cfg),
        Commands::Hash { path } => {
            match std::fs::read(&path) {
                Ok(bytes) => {
                    println!("{}", idempotency::blake3_hex(&bytes));
                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("read {:?}: {}", path, e)),
            }
        }
        Commands::Themes { cmd } => match cmd {
            ThemesCmd::List => themes::cmd_list(&cfg),
            ThemesCmd::Current => themes::cmd_current(&cfg),
            ThemesCmd::Switch { name } => themes::cmd_switch(&cfg, &name),
            ThemesCmd::Test { name, all } => themes::cmd_test(&cfg, name.as_deref(), all),
            ThemesCmd::Check => themes::cmd_check(&cfg),
        },
        #[cfg(feature = "local")]
        Commands::Watch => watch::cmd_watch(&cfg),
    };
    if let Err(e) = result {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}
