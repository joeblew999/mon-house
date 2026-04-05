use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

mod build;
mod fonts;
mod new;
mod themes;
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
    /// Root directory for runtime assets (fonts, images).
    /// Override to point at an S3 mount or shared volume without changing anything else.
    #[arg(long, env = "QUICK_RESOURCES_DIR", default_value = "resources", global = true)]
    pub resources_dir: PathBuf,

    /// Directory where .ttf files are stored (defaults to <resources_dir>/fonts)
    #[arg(long, env = "QUICK_FONT_DIR", global = true)]
    pub font_dir: Option<PathBuf>,

    /// Directory where spec images are stored (defaults to <resources_dir>/images)
    #[arg(long, env = "QUICK_IMAGES_DIR", global = true)]
    pub images_dir: Option<PathBuf>,

    /// Directory containing theme.typ and themes/ (scripts, Typst sources).
    /// Override to point at a shared or versioned scripts location.
    #[arg(long, env = "QUICK_SCRIPTS_DIR", default_value = "scripts", global = true)]
    pub scripts_dir: PathBuf,

    /// Active theme wrapper file (defaults to <scripts_dir>/theme.typ)
    #[arg(long, env = "QUICK_THEME_FILE", global = true)]
    pub theme_file: Option<PathBuf>,

    /// Google Web Fonts Helper API base URL
    #[arg(long, env = "QUICK_GWFH_API", default_value = "https://gwfh.mranftl.com/api/fonts", global = true)]
    pub gwfh_api: String,

    /// Comma-separated font weights to download (e.g. "400,700")
    #[arg(long, env = "QUICK_WEIGHTS", default_value = "400,700", global = true)]
    pub weights: String,

    /// Directory where generated PDFs are written.
    /// Override to point at an S3 mount or shared volume.
    #[arg(long, env = "QUICK_OUT_DIR", default_value = "out", global = true)]
    pub out_dir: PathBuf,

    /// Directory containing EN spec .md files and TEMPLATE.md.
    #[arg(long, env = "QUICK_SPECS_DIR", default_value = "specs", global = true)]
    pub specs_dir: PathBuf,
}

impl Config {
    /// Resolved font directory: explicit --font-dir > <resources_dir>/fonts
    pub fn resolved_font_dir(&self) -> PathBuf {
        self.font_dir.clone().unwrap_or_else(|| self.resources_dir.join("fonts"))
    }

    /// Resolved images directory: explicit --images-dir > <resources_dir>/images
    pub fn resolved_images_dir(&self) -> PathBuf {
        self.images_dir.clone().unwrap_or_else(|| self.resources_dir.join("images"))
    }

    /// Resolved theme wrapper: explicit --theme-file > <scripts_dir>/theme.typ
    pub fn resolved_theme_file(&self) -> PathBuf {
        self.theme_file.clone().unwrap_or_else(|| self.scripts_dir.join("theme.typ"))
    }

    /// Build-stamp path — written after a successful full build.
    pub fn build_stamp(&self) -> PathBuf {
        self.out_dir.join(".build-stamp")
    }

    pub fn parsed_weights(&self) -> Vec<u32> {
        self.weights
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }

    pub fn done_file(&self) -> PathBuf {
        self.resolved_font_dir().join(".done")
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
    /// Watch *.md and scripts/theme.typ; runs fonts → translate → build directly (no mise needed).
    /// All three idempotency layers are preserved in Rust: hash checks, stamp mtime, per-file.
    Watch,
    /// Create a new spec file from TEMPLATE.md
    New {
        /// Spec name in UPPER CASE (e.g. DECK)
        name: String,
    },
    /// Remove the out/ directory
    Clean,
    /// Theme registry — list, switch, test, and check Typst themes
    Themes {
        #[command(subcommand)]
        cmd: ThemesCmd,
    },
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
        Commands::Watch => watch::cmd_watch(&cfg),
        Commands::New { name } => new::cmd_new(&cfg, &name),
        Commands::Clean => build::cmd_clean(&cfg),
        Commands::Themes { cmd } => match cmd {
            ThemesCmd::List => themes::cmd_list(&cfg),
            ThemesCmd::Current => themes::cmd_current(&cfg),
            ThemesCmd::Switch { name } => themes::cmd_switch(&cfg, &name),
            ThemesCmd::Test { name, all } => themes::cmd_test(&cfg, name.as_deref(), all),
            ThemesCmd::Check => themes::cmd_check(&cfg),
        },
    };
    if let Err(e) = result {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}
