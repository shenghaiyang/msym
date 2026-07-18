use clap::Parser;

/// Download Material Symbols Compose code from Google Fonts.
#[derive(Parser)]
#[command(name = "msym")]
pub struct Cli {
    /// Path to the config file
    #[arg(default_value = "msym.toml")]
    pub config: String,

    /// Force re-download all icons, even if already present
    #[arg(short, long)]
    pub force: bool,

    /// Number of concurrent downloads (1–32)
    #[arg(short = 'j', long, default_value = "4", value_parser = clap::value_parser!(u32).range(1..=32)
    )]
    pub jobs: u32,

    /// Verbose output (show URLs, retry details, etc.)
    #[arg(short, long)]
    pub verbose: bool,
}
