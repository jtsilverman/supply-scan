use crate::registry::PackageMetadata;
use super::{Finding, RiskLevel};

/// Parse a YYYY-MM-DD date string into (year, month, day).
fn parse_date(s: &str) -> Option<(i64, u32, u32)> {
    // Handle full ISO 8601 datetime by taking only the date part
    let date_part = if s.len() >= 10 { &s[..10] } else { s };
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year = parts[0].parse::<i64>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let day = parts[2].parse::<u32>().ok()?;
    Some((year, month, day))
}

/// Approximate days since epoch for simple comparison.
fn approx_days(year: i64, month: u32, day: u32) -> i64 {
    year * 365 + (year / 4) - (year / 100) + (year / 400)
        + match month {
            1 => 0,
            2 => 31,
            3 => 59,
            4 => 90,
            5 => 120,
            6 => 151,
            7 => 181,
            8 => 212,
            9 => 243,
            10 => 273,
            11 => 304,
            12 => 334,
            _ => 0,
        } as i64
        + day as i64
}

fn is_within_30_days(date_str: &str) -> bool {
    let (pub_y, pub_m, pub_d) = match parse_date(date_str) {
        Some(d) => d,
        None => return false,
    };

    // Get current time using UNIX_EPOCH
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Convert epoch seconds to approximate date
    // 86400 seconds per day, epoch is 1970-01-01
    let total_days = (now / 86400) as i64;
    // Approximate: 1970 + total_days/365.25
    let now_year = 1970 + total_days / 365;
    let remaining = total_days - (now_year - 1970) * 365 - ((now_year - 1) / 4 - 1970 / 4)
        + ((now_year - 1) / 100 - 1970 / 100)
        - ((now_year - 1) / 400 - 1970 / 400);

    let now_days = approx_days(now_year, 1, 1) + remaining;
    let pub_days = approx_days(pub_y, pub_m, pub_d);

    (now_days - pub_days).abs() <= 30
}

pub fn check(metadata: &Option<PackageMetadata>) -> Vec<Finding> {
    let meta = match metadata {
        Some(m) => m,
        None => return vec![],
    };

    let mut findings = Vec::new();

    if meta.has_install_scripts {
        findings.push(Finding {
            level: RiskLevel::Warning,
            check: "signals".to_string(),
            description: format!("Package has install scripts: {}", meta.install_scripts.join(", ")),
        });
    }

    if meta.maintainer_count == 1 {
        findings.push(Finding {
            level: RiskLevel::Info,
            check: "signals".to_string(),
            description: "Single maintainer".to_string(),
        });
    }

    if let Some(ref date) = meta.latest_published {
        if is_within_30_days(date) {
            findings.push(Finding {
                level: RiskLevel::Warning,
                check: "signals".to_string(),
                description: "Package published within last 30 days".to_string(),
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_scripts_warning() {
        let meta = PackageMetadata {
            exists: true,
            first_published: Some("2020-01-01".to_string()),
            latest_published: Some("2024-01-01".to_string()),
            maintainer_count: 5,
            has_install_scripts: true,
            install_scripts: vec!["preinstall".to_string(), "postinstall".to_string()],
        };
        let findings = check(&Some(meta));
        assert!(findings.iter().any(|f| f.level == RiskLevel::Warning
            && f.description.contains("install scripts")
            && f.description.contains("preinstall")));
    }

    #[test]
    fn single_maintainer_info() {
        let meta = PackageMetadata {
            exists: true,
            first_published: Some("2020-01-01".to_string()),
            latest_published: Some("2024-01-01".to_string()),
            maintainer_count: 1,
            has_install_scripts: false,
            install_scripts: vec![],
        };
        let findings = check(&Some(meta));
        assert!(findings.iter().any(|f| f.level == RiskLevel::Info
            && f.description.contains("Single maintainer")));
    }

    #[test]
    fn recent_publish_warning() {
        // Use today's date to guarantee it's within 30 days
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let days = now / 86400;
        // Rough conversion back to YYYY-MM-DD
        let mut y = 1970i64;
        let mut remaining = days as i64;
        loop {
            let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
            if remaining < days_in_year {
                break;
            }
            remaining -= days_in_year;
            y += 1;
        }
        let month_days = [31, if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 29 } else { 28 },
            31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut m = 1u32;
        for &md in &month_days {
            if remaining < md {
                break;
            }
            remaining -= md;
            m += 1;
        }
        let d = remaining + 1;
        let today = format!("{:04}-{:02}-{:02}", y, m, d);

        let meta = PackageMetadata {
            exists: true,
            first_published: Some("2020-01-01".to_string()),
            latest_published: Some(today),
            maintainer_count: 5,
            has_install_scripts: false,
            install_scripts: vec![],
        };
        let findings = check(&Some(meta));
        assert!(findings.iter().any(|f| f.level == RiskLevel::Warning
            && f.description.contains("published within last 30 days")));
    }

    #[test]
    fn none_metadata_returns_empty() {
        let findings = check(&None);
        assert!(findings.is_empty());
    }
}
