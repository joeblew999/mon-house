use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

mod build;
mod fonts;
mod new;
mod translate;
mod watch;

/// Global configuration for quick-tool.
///
/// ## How mise passes variables to this binary
///
/// This follows the standard pattern for mise-compatible Rust tools
/// (used by mise itself, fd, ripgrep, bat, and others):
///
/// 1. `mise.toml` declares values in the `[env]` section:
///    ```toml
///    [env]
///    QUICK_FONT_DIR = "fonts"
///    QUICK_THEME_FILE = "scripts/theme.typ"
///    ```
///    mise sets these as real environment variables for every subprocess it spawns.
///
/// 2. This struct reads them via clap's `env =` attribute.
///    clap resolves precedence automatically:
///    **CLI flag  >  env var  >  compiled default**
///
/// 3. To override for your machine without touching mise.toml:
///    ```toml
///    # mise.local.toml  (gitignored)
///    [env]
///    QUICK_FONT_DIR = "/tmp/my-fonts"
///    ```
///
/// 4. When published as a mise plugin (`mise use cargo:quick-tool`),
///    users configure it the same way — no plugin-specific API needed.
///    The `env =` attribute means `--help` shows both the flag name
///    and the env var name, so the interface is self-documenting.
#[derive(Args, Clone, Debug)]
pub struct Config {
    /// Directory where .ttf files are stored
    #[arg(long, env = "QUICK_FONT_DIR", default_value = "fonts", global = true)]
    pub font_dir: PathBuf,

    /// Single source of truth for the font stack
    #[arg(long, env = "QUICK_THEME_FILE", default_value = "scripts/theme.typ", global = true)]
    pub theme_file: PathBuf,

    /// Google Web Fonts Helper API base URL
    #[arg(long, env = "QUICK_GWFH_API", default_value = "https://gwfh.mranftl.com/api/fonts", global = true)]
    pub gwfh_api: String,

    /// Comma-separated font weights to download (e.g. "400,700")
    #[arg(long, env = "QUICK_WEIGHTS", default_value = "400,700", global = true)]
    pub weights: String,
}

impl Config {
    pub fn parsed_weights(&self) -> Vec<u32> {
        self.weights
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }

    pub fn done_file(&self) -> PathBuf {
        self.font_dir.join(".done")
    }
}

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
    /// Watch *.md and scripts/theme.typ; re-run `mise run fonts && mise run all` on change.
    /// Replaces the watchexec external tool. All mise idempotency layers are preserved.
    Watch,
    /// Create a new spec file from TEMPLATE.md
    New {
        /// Spec name in UPPER CASE (e.g. DECK)
        name: String,
    },
    /// Remove the out/ directory
    Clean,
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
        Commands::Translate { files } => translate::cmd_translate(files),
        Commands::Build { name } => build::cmd_build(&cfg, name),
        Commands::One { name } => {
            let path = std::path::PathBuf::from(format!("{name}.md"));
            translate::cmd_translate(vec![path])
                .and_then(|_| build::cmd_build(&cfg, Some(name)))
        }
        Commands::Watch => watch::cmd_watch(),
        Commands::New { name } => new::cmd_new(&name),
        Commands::Clean => build::cmd_clean(),
    };
    if let Err(e) = result {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}
