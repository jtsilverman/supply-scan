use crate::registry::PackageMetadata;
use super::{Finding, RiskLevel};

pub fn check(metadata: &Option<PackageMetadata>) -> Vec<Finding> {
    if metadata.is_none() {
        return vec![Finding {
            level: RiskLevel::Critical,
            check: "hallucination".to_string(),
            description: "Package does not exist in registry — possible AI hallucination (slopsquatting)".to_string(),
        }];
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_metadata_returns_critical() {
        let findings = check(&None);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].level, RiskLevel::Critical);
        assert_eq!(findings[0].check, "hallucination");
        assert!(findings[0].description.contains("slopsquatting"));
    }

    #[test]
    fn some_metadata_returns_empty() {
        let meta = PackageMetadata {
            exists: true,
            first_published: Some("2020-01-01".to_string()),
            latest_published: Some("2024-01-01".to_string()),
            maintainer_count: 5,
            has_install_scripts: false,
            install_scripts: vec![],
        };
        let findings = check(&Some(meta));
        assert!(findings.is_empty());
    }
}
