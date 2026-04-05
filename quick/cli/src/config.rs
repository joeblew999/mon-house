/// Configuration — single source of truth for all configurable paths and settings.
///
/// Every directory the binary reads from or writes to is a field here, backed
/// by a `QUICK_*` env var.  No module may open, glob, or write a file using a
/// literal path string — all paths flow through `Config`.
///
/// Resolver methods (`resolved_*`) handle optional fields that derive from a
/// parent directory.  Precedence (clap-enforced):
///   CLI flag > env var > compiled default
use std::path::PathBuf;

use clap::Args;

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
    #[arg(long, env = "QUICK_OUT_DIR", default_value = "out", global = true)]
    pub out_dir: PathBuf,

    /// Directory containing EN spec .md files.
    #[arg(long, env = "QUICK_SPECS_DIR", default_value = "specs", global = true)]
    pub specs_dir: PathBuf,

    /// Template file used by `new` to scaffold a spec (defaults to TEMPLATE.md at project root)
    #[arg(long, env = "QUICK_TEMPLATE_FILE", global = true)]
    pub template_file: Option<PathBuf>,

    /// Claude model used by the API translation backend.
    /// Has no effect when the CLI backend is used (ANTHROPIC_API_KEY absent, local feature only).
    #[arg(long, env = "QUICK_CLAUDE_MODEL", default_value = "claude-opus-4-6", global = true)]
    pub claude_model: String,

    /// Anthropic API key. When set, translation uses the Messages API directly (works everywhere,
    /// including Cloudflare). When absent and the `local` feature is enabled, falls back to the
    /// claude CLI subprocess.
    #[arg(long, env = "ANTHROPIC_API_KEY", global = true)]
    pub anthropic_api_key: Option<String>,
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

    /// Resolved template file: explicit --template-file > TEMPLATE.md at project root
    pub fn resolved_template_file(&self) -> PathBuf {
        self.template_file.clone().unwrap_or_else(|| PathBuf::from("TEMPLATE.md"))
    }

    /// Path where the build stamp is written after a successful full build.
    pub fn build_stamp(&self) -> PathBuf {
        self.out_dir.join(".build-stamp")
    }

    /// Path where the fonts stamp is written after a successful font download.
    pub fn done_file(&self) -> PathBuf {
        self.resolved_font_dir().join(".done")
    }

    pub fn parsed_weights(&self) -> Vec<u32> {
        self.weights
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }
}
