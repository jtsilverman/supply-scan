use crate::checks::{self, Finding, RiskLevel};
use crate::parsers::{Ecosystem, Package};
use crate::registry;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Serialize)]
pub struct PackageReport {
    pub name: String,
    pub version: String,
    pub ecosystem: Ecosystem,
    pub findings: Vec<Finding>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Serialize)]
pub struct ScanReport {
    pub packages: Vec<PackageReport>,
    pub total_scanned: usize,
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
}

pub async fn scan(packages: Vec<Package>, no_network: bool) -> ScanReport {
    let semaphore = Arc::new(Semaphore::new(10));
    let client = reqwest::Client::new();

    let mut handles = Vec::new();

    for pkg in packages {
        let sem = semaphore.clone();
        let client = client.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            scan_package(&client, pkg, no_network).await
        }));
    }

    let mut reports = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(report) => reports.push(report),
            Err(_) => {} // task panicked, skip
        }
    }

    let total_scanned = reports.len();
    let mut critical_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;

    for report in &reports {
        for finding in &report.findings {
            match finding.level {
                RiskLevel::Critical => critical_count += 1,
                RiskLevel::Warning => warning_count += 1,
                RiskLevel::Info => info_count += 1,
            }
        }
    }

    ScanReport {
        packages: reports,
        total_scanned,
        critical_count,
        warning_count,
        info_count,
    }
}

async fn scan_package(client: &reqwest::Client, pkg: Package, no_network: bool) -> PackageReport {
    let mut findings = Vec::new();

    let (metadata, vulns) = if no_network {
        (None, Vec::new())
    } else {
        let meta = match pkg.ecosystem {
            Ecosystem::Npm => registry::npm::fetch_metadata(client, &pkg.name).await,
            Ecosystem::PyPI => registry::pypi::fetch_metadata(client, &pkg.name).await,
        };

        let osv_ecosystem = match pkg.ecosystem {
            Ecosystem::Npm => "npm",
            Ecosystem::PyPI => "PyPI",
        };
        let vulns =
            registry::osv::query_vulnerabilities(client, &pkg.name, osv_ecosystem).await;

        (meta, vulns)
    };

    // 1. Existence check
    findings.extend(checks::existence::check(&metadata));

    // 2. Typosquat check
    findings.extend(checks::typosquat::check(&pkg.name, pkg.ecosystem));

    // 3. Signal checks
    findings.extend(checks::signals::check(&metadata));

    // 4. Vulnerability check
    findings.extend(checks::vulnerability::check(&vulns));

    // Compute highest risk level
    let risk_level = findings
        .iter()
        .map(|f| f.level)
        .max()
        .unwrap_or(RiskLevel::Info);

    PackageReport {
        name: pkg.name,
        version: pkg.version,
        ecosystem: pkg.ecosystem,
        findings,
        risk_level,
    }
}
