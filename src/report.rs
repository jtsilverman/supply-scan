use crate::checks::RiskLevel;
use crate::scanner::ScanReport;
use colored::Colorize;

pub fn print_terminal(report: &ScanReport, verbose: bool) {
    let has_findings = report.packages.iter().any(|p| !p.findings.is_empty());

    for pkg in &report.packages {
        if pkg.findings.is_empty() && !verbose {
            continue;
        }

        let header = format!("{} {} ({})", pkg.name, pkg.version, format!("{:?}", pkg.ecosystem).to_lowercase());
        println!("\n{}", header.bold());

        if pkg.findings.is_empty() {
            println!("  {}", "No issues".green());
            continue;
        }

        for finding in &pkg.findings {
            let badge = match finding.level {
                RiskLevel::Critical => "[CRITICAL]".red().bold(),
                RiskLevel::Warning => "[WARNING]".yellow().bold(),
                RiskLevel::Info => "[INFO]".blue().bold(),
            };
            println!("  {} {}", badge, finding.description);
        }
    }

    println!();
    println!(
        "Scanned {} packages: {} critical, {} warnings, {} info",
        report.total_scanned, report.critical_count, report.warning_count, report.info_count
    );

    if !has_findings {
        println!("{}", "No issues found".green());
    }
}

pub fn print_json(report: &ScanReport) {
    match serde_json::to_string_pretty(report) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing report: {}", e),
    }
}
