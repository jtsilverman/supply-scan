use std::path::Path;

use super::{Ecosystem, Package};

/// Parse package.json from the given directory, extracting dependencies and devDependencies.
/// Returns an empty vec if the file is missing or malformed.
pub fn parse(dir: &Path) -> Vec<Package> {
    let path = dir.join("package.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    parse_str(&content)
}

fn parse_str(content: &str) -> Vec<Package> {
    let val: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut packages = Vec::new();

    for key in &["dependencies", "devDependencies"] {
        if let Some(obj) = val.get(key).and_then(|v| v.as_object()) {
            for (name, version) in obj {
                packages.push(Package {
                    name: name.clone(),
                    version: match version.as_str() {
                        Some(v) => v.to_string(),
                        None => {
                            eprintln!("Warning: non-string version for package '{}', skipping", name);
                            continue;
                        }
                    },
                    ecosystem: Ecosystem::Npm,
                });
            }
        }
    }

    packages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deps_and_dev_deps() {
        let json = r#"{
            "dependencies": {
                "express": "^4.18.0",
                "lodash": "~4.17.21"
            },
            "devDependencies": {
                "jest": "^29.0.0"
            }
        }"#;
        let pkgs = parse_str(json);
        assert_eq!(pkgs.len(), 3);
        assert!(pkgs.iter().all(|p| p.ecosystem == Ecosystem::Npm));
        assert!(pkgs.iter().any(|p| p.name == "express" && p.version == "^4.18.0"));
        assert!(pkgs.iter().any(|p| p.name == "lodash" && p.version == "~4.17.21"));
        assert!(pkgs.iter().any(|p| p.name == "jest" && p.version == "^29.0.0"));
    }

    #[test]
    fn test_parse_empty_deps() {
        let json = r#"{
            "dependencies": {},
            "devDependencies": {}
        }"#;
        let pkgs = parse_str(json);
        assert!(pkgs.is_empty());
    }

    #[test]
    fn test_parse_missing_fields() {
        let json = r#"{ "name": "my-app", "version": "1.0.0" }"#;
        let pkgs = parse_str(json);
        assert!(pkgs.is_empty());
    }

    #[test]
    fn test_parse_invalid_json() {
        let pkgs = parse_str("not json at all");
        assert!(pkgs.is_empty());
    }
}
