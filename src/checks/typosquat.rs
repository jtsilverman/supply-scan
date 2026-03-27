use strsim::levenshtein;
use crate::parsers::Ecosystem;
use crate::popular;
use super::{Finding, RiskLevel};

pub fn check(name: &str, ecosystem: Ecosystem) -> Vec<Finding> {
    let popular_list = match ecosystem {
        Ecosystem::Npm => popular::npm_packages(),
        Ecosystem::PyPI => popular::pypi_packages(),
    };

    // If the package itself is in the popular list, it's not a typosquat
    if popular_list.contains(&name) {
        return vec![];
    }

    let mut findings = Vec::new();
    for &popular in popular_list {
        let dist = levenshtein(name, popular);
        if dist >= 1 && dist <= 2 {
            findings.push(Finding {
                level: RiskLevel::Warning,
                check: "typosquat".to_string(),
                description: format!(
                    "Similar to popular package '{}' (edit distance: {}) — possible typosquat",
                    popular, dist
                ),
            });
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expresss_flags_for_npm() {
        let findings = check("expresss", Ecosystem::Npm);
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("express")));
        assert_eq!(findings[0].level, RiskLevel::Warning);
    }

    #[test]
    fn requests_does_not_flag_for_pypi() {
        let findings = check("requests", Ecosystem::PyPI);
        assert!(findings.is_empty());
    }

    #[test]
    fn reqeusts_flags_for_pypi() {
        let findings = check("reqeusts", Ecosystem::PyPI);
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("requests")));
    }

    #[test]
    fn totally_unique_does_not_flag() {
        let findings = check("totally-unique-xyz", Ecosystem::Npm);
        assert!(findings.is_empty());
    }
}
