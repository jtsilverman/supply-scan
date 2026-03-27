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
/// Skips comments (#), empty lines, `-e`/`-r` flags, and URL lines.
fn parse_requirements_txt(content: &str) -> Vec<Package> {
    let mut packages = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Skip pip flags (-e, -r, -i, -f, --index-url, etc.) and URL lines
        if line.starts_with('-') || line.starts_with("http://") || line.starts_with("https://") {
            continue;
        }

        // Strip inline comments
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }

        // Strip environment markers (everything after ';')
        let line = line.split(';').next().unwrap().trim();

        // Strip extras (e.g., requests[security] → requests)
        let line = if let Some(bracket_pos) = line.find('[') {
            if let Some(close_pos) = line.find(']') {
                let mut s = String::with_capacity(line.len());
                s.push_str(&line[..bracket_pos]);
                s.push_str(&line[close_pos + 1..]);
                std::borrow::Cow::Owned(s)
            } else {
                std::borrow::Cow::Borrowed(line)
            }
        } else {
            std::borrow::Cow::Borrowed(line)
        };
        let line = line.as_ref();

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

            // Strip environment markers (everything after ';')
            let s = s.split(';').next().unwrap().trim();

            // Strip extras (e.g., requests[security] → requests)
            let s = if let Some(bracket_pos) = s.find('[') {
                if let Some(close_pos) = s.find(']') {
                    let mut buf = String::with_capacity(s.len());
                    buf.push_str(&s[..bracket_pos]);
                    buf.push_str(&s[close_pos + 1..]);
                    std::borrow::Cow::Owned(buf)
                } else {
                    std::borrow::Cow::Borrowed(s)
                }
            } else {
                std::borrow::Cow::Borrowed(s)
            };
            let s = s.as_ref();

            // Split on version specifiers: ==, >=, <=, ~=, !=, >, <
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
    fn test_extras_stripped() {
        let content = "requests[security]==2.31.0\nflask[async]>=2.0\n";
        let pkgs = parse_requirements_txt(content);
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].name, "requests");
        assert_eq!(pkgs[0].version, "2.31.0");
        assert_eq!(pkgs[1].name, "flask");
    }

    #[test]
    fn test_environment_markers_stripped() {
        let content = "flask>=2.0; python_version>=\"3.8\"\n";
        let pkgs = parse_requirements_txt(content);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].name, "flask");
        assert_eq!(pkgs[0].version, ">=2.0");
    }

    #[test]
    fn test_editable_and_url_lines_skipped() {
        let content = "\
-e git+https://github.com/user/repo.git#egg=myrepo
https://files.example.com/package.tar.gz
-r other-requirements.txt
-i https://pypi.org/simple
numpy
";
        let pkgs = parse_requirements_txt(content);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].name, "numpy");
    }

    #[test]
    fn test_pyproject_extras_and_markers() {
        let content = r#"
[project]
name = "myapp"
dependencies = [
    "requests[security]>=2.31.0",
    "flask>=2.0; python_version>='3.8'",
]
"#;
        let pkgs = parse_pyproject_toml(content);
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].name, "requests");
        assert_eq!(pkgs[1].name, "flask");
        assert_eq!(pkgs[1].version, ">=2.0");
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
