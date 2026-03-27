mod checks;
mod parsers;
mod popular;
mod registry;
mod report;
mod scanner;

use clap::Parser;
use parsers::Ecosystem;
use std::path::PathBuf;
use std::process;

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

    // Detect ecosystem
    let ecosystems: Vec<Ecosystem> = if let Some(ref eco) = cli.ecosystem {
        match eco.to_lowercase().as_str() {
            "npm" => vec![Ecosystem::Npm],
            "pypi" => vec![Ecosystem::PyPI],
            other => {
                eprintln!("Error: unknown ecosystem '{}'. Use 'npm' or 'pypi'.", other);
                process::exit(1);
            }
        }
    } else {
        let mut detected = Vec::new();
        if cli.path.join("package.json").exists() {
            detected.push(Ecosystem::Npm);
        }
        if cli.path.join("requirements.txt").exists() || cli.path.join("pyproject.toml").exists() {
            detected.push(Ecosystem::PyPI);
        }
        detected
    };

    if ecosystems.is_empty() {
        eprintln!("Error: no package.json, requirements.txt, or pyproject.toml found in {}", cli.path.display());
        process::exit(1);
    }

    // Parse dependencies
    let mut packages = Vec::new();
    for eco in &ecosystems {
        match eco {
            Ecosystem::Npm => packages.extend(parsers::npm::parse(&cli.path)),
            Ecosystem::PyPI => packages.extend(parsers::pypi::parse(&cli.path)),
        }
    }

    if packages.is_empty() {
        eprintln!("Error: no dependencies found in {}", cli.path.display());
        process::exit(1);
    }

    eprintln!("supply-scan: found {} packages", packages.len());

    // Scan
    let report = scanner::scan(packages, cli.no_network).await;

    // Output
    if cli.json {
        report::print_json(&report);
    } else {
        report::print_terminal(&report, cli.verbose);
    }

    // Pre-commit exit code
    if cli.pre_commit && report.critical_count > 0 {
        process::exit(1);
    }
}
