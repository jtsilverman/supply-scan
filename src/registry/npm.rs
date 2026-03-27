use super::PackageMetadata;
use serde_json::Value;
use std::time::Duration;

const SUSPICIOUS_SCRIPTS: &[&str] = &["preinstall", "postinstall", "preuninstall", "install"];

pub async fn fetch_metadata(
    client: &reqwest::Client,
    name: &str,
) -> Option<PackageMetadata> {
    let url = format!("https://registry.npmjs.org/{}", name);
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return None;
    }

    let body: Value = resp.json().await.ok()?;

    // Parse publish dates from the "time" object
    let (first_published, latest_published) = parse_publish_dates(&body);

    // Maintainer count
    let maintainer_count = body
        .get("maintainers")
        .and_then(|m| m.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    // Check for suspicious install scripts in the latest version
    let (has_install_scripts, install_scripts) = check_install_scripts(&body);

    Some(PackageMetadata {
        exists: true,
        first_published,
        latest_published,
        maintainer_count,
        has_install_scripts,
        install_scripts,
    })
}

fn parse_publish_dates(body: &Value) -> (Option<String>, Option<String>) {
    let time = match body.get("time").and_then(|t| t.as_object()) {
        Some(t) => t,
        None => return (None, None),
    };

    // Filter out "created" and "modified" meta-keys, keep only version timestamps
    let mut dates: Vec<&str> = time
        .iter()
        .filter(|(k, _)| *k != "created" && *k != "modified")
        .filter_map(|(_, v)| v.as_str())
        .collect();

    dates.sort();

    let first = dates.first().map(|s| s.to_string());
    let last = dates.last().map(|s| s.to_string());
    (first, last)
}

fn check_install_scripts(body: &Value) -> (bool, Vec<String>) {
    // Get the latest version tag
    let latest_tag = body
        .pointer("/dist-tags/latest")
        .and_then(|v| v.as_str());

    let latest_tag = match latest_tag {
        Some(t) => t,
        None => return (false, vec![]),
    };

    // Get the scripts object for the latest version
    let scripts = body
        .get("versions")
        .and_then(|v| v.get(latest_tag))
        .and_then(|v| v.get("scripts"))
        .and_then(|v| v.as_object());

    let scripts = match scripts {
        Some(s) => s,
        None => return (false, vec![]),
    };

    let found: Vec<String> = SUSPICIOUS_SCRIPTS
        .iter()
        .filter(|name| scripts.contains_key(**name))
        .map(|name| name.to_string())
        .collect();

    let has = !found.is_empty();
    (has, found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_express() {
        let client = make_client();
        let meta = fetch_metadata(&client, "express").await;
        assert!(meta.is_some(), "express should exist on npm");
        let meta = meta.unwrap();
        assert!(meta.exists);
        assert!(meta.maintainer_count > 0);
        assert!(meta.first_published.is_some());
        assert!(meta.latest_published.is_some());
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_nonexistent() {
        let client = make_client();
        let meta =
            fetch_metadata(&client, "this-package-definitely-does-not-exist-xyz123").await;
        assert!(meta.is_none(), "nonexistent package should return None");
    }
}
