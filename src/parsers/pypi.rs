use std::path::Path;

use super::{Ecosystem, Package};

/// Parse Python dependencies from a directory.
/// Tries requirements.txt first, then falls back to pyproject.toml.
/// Returns an empty vec if neither file exists.
pub fn parse(dir: &Path) -> Vec<Package> {
    let req_path = dir.join("requirements.txt");
    if let Ok(content) = std::fs::read_to_string(&req_path) {
        return parse_requirements_txt(&content);
    }

    let toml_path = dir.join("pyproject.toml");
    if let Ok(content) = std::fs::read_to_string(&toml_path) {
        return parse_pyproject_toml(&content);
    }

    Vec::new()
}

/// Parse a requirements.txt string.
/// Handles: `pkg==1.0`, `pkg>=1.0`, `pkg~=1.0`, `pkg!=1.0`, `pkg<=1.0`, `pkg<1.0`, `pkg>1.0`, bare `pkg`.
/// Skips comments (#) and empty lines.
fn parse_requirements_txt(content: &str) -> Vec<Package> {
    let mut packages = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Strip inline comments
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }

        // Split on version specifiers: ==, >=, <=, ~=, !=, >, <
        let (name, version) = if let Some(pos) = line.find("==") {
            (&line[..pos], &line[pos + 2..])
        } else if let Some(pos) = line.find(">=") {
            (&line[..pos], &line[pos..])
        } else if let Some(pos) = line.find("<=") {
            (&line[..pos], &line[pos..])
        } else if let Some(pos) = line.find("~=") {
            (&line[..pos], &line[pos..])
        } else if let Some(pos) = line.find("!=") {
            (&line[..pos], &line[pos..])
        } else if let Some(pos) = line.find('>') {
            (&line[..pos], &line[pos..])
        } else if let Some(pos) = line.find('<') {
            (&line[..pos], &line[pos..])
        } else {
            (line, "*")
        };

        let name = name.trim();
        let version = version.trim();
        if !name.is_empty() {
            packages.push(Package {
                name: name.to_string(),
                version: version.to_string(),
                ecosystem: Ecosystem::PyPI,
            });
        }
    }

    packages
}

/// Parse [project] dependencies from a pyproject.toml string.
fn parse_pyproject_toml(content: &str) -> Vec<Package> {
    let table: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let deps = match table
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut packages = Vec::new();
    for dep in deps {
        if let Some(s) = dep.as_str() {
            let s = s.trim();
            // Same parsing logic as requirements.txt for individual specifiers
            let (name, version) = if let Some(pos) = s.find("==") {
                (&s[..pos], &s[pos + 2..])
            } else if let Some(pos) = s.find(">=") {
                (&s[..pos], &s[pos..])
            } else if let Some(pos) = s.find("<=") {
                (&s[..pos], &s[pos..])
            } else if let Some(pos) = s.find("~=") {
                (&s[..pos], &s[pos..])
            } else if let Some(pos) = s.find("!=") {
                (&s[..pos], &s[pos..])
            } else if let Some(pos) = s.find('>') {
                (&s[..pos], &s[pos..])
            } else if let Some(pos) = s.find('<') {
                (&s[..pos], &s[pos..])
            } else {
                (s, "*")
            };

            let name = name.trim();
            let version = version.trim();
            if !name.is_empty() {
                packages.push(Package {
                    name: name.to_string(),
                    version: version.to_string(),
                    ecosystem: Ecosystem::PyPI,
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
    fn test_requirements_txt_various_formats() {
        let content = "\
requests==2.31.0
flask>=2.0
numpy
# this is a comment

pandas~=1.5.0
scipy>1.9
";
        let pkgs = parse_requirements_txt(content);
        assert_eq!(pkgs.len(), 5);
        assert!(pkgs.iter().all(|p| p.ecosystem == Ecosystem::PyPI));
        assert!(pkgs.iter().any(|p| p.name == "requests" && p.version == "2.31.0"));
        assert!(pkgs.iter().any(|p| p.name == "flask" && p.version == ">=2.0"));
        assert!(pkgs.iter().any(|p| p.name == "numpy" && p.version == "*"));
        assert!(pkgs.iter().any(|p| p.name == "pandas" && p.version == "~=1.5.0"));
        assert!(pkgs.iter().any(|p| p.name == "scipy" && p.version == ">1.9"));
    }

    #[test]
    fn test_requirements_txt_comments_and_blanks() {
        let content = "\
# Full line comment
   # indented comment


requests==1.0
";
        let pkgs = parse_requirements_txt(content);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].name, "requests");
    }

    #[test]
    fn test_requirements_txt_inline_comment() {
        let content = "requests==2.31.0  # HTTP library\n";
        let pkgs = parse_requirements_txt(content);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].version, "2.31.0");
    }

    #[test]
    fn test_pyproject_toml_parsing() {
        let content = r#"
[project]
name = "myapp"
dependencies = [
    "requests>=2.31.0",
    "flask==2.3.0",
    "numpy",
]
"#;
        let pkgs = parse_pyproject_toml(content);
        assert_eq!(pkgs.len(), 3);
        assert!(pkgs.iter().all(|p| p.ecosystem == Ecosystem::PyPI));
        assert!(pkgs.iter().any(|p| p.name == "requests" && p.version == ">=2.31.0"));
        assert!(pkgs.iter().any(|p| p.name == "flask" && p.version == "2.3.0"));
        assert!(pkgs.iter().any(|p| p.name == "numpy" && p.version == "*"));
    }

    #[test]
    fn test_pyproject_toml_missing_deps() {
        let content = r#"
[project]
name = "myapp"
"#;
        let pkgs = parse_pyproject_toml(content);
        assert!(pkgs.is_empty());
    }

    #[test]
    fn test_pyproject_toml_invalid() {
        let pkgs = parse_pyproject_toml("not valid toml {{{}");
        assert!(pkgs.is_empty());
    }
}
