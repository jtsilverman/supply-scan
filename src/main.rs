mod parsers;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "supply-scan")]
#[command(about = "Scan project dependencies for malicious, typosquatted, and AI-hallucinated packages")]
#[command(version)]
struct Cli {
    /// Project directory to scan (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Exit 1 if any CRITICAL findings (for CI/pre-commit)
    #[arg(long)]
    pre_commit: bool,

    /// Force ecosystem detection (npm or pypi)
    #[arg(long)]
    ecosystem: Option<String>,

    /// Skip registry checks, only do local analysis
    #[arg(long)]
    no_network: bool,

    /// Show all packages, not just flagged ones
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    eprintln!(
        "supply-scan: scanning {} ...",
        cli.path.display()
    );
}
