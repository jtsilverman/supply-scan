use super::PackageMetadata;
use serde_json::Value;
use std::time::Duration;

pub async fn fetch_metadata(
    client: &reqwest::Client,
    name: &str,
) -> Option<PackageMetadata> {
    let url = format!("https://pypi.org/pypi/{}/json", name);
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

    // Parse publish dates from releases
    let (first_published, latest_published) = parse_release_dates(&body);

    // Maintainer count: PyPI JSON API only exposes the author field
    let has_author = body
        .pointer("/info/author")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let maintainer_count = if has_author { 1 } else { 0 };

    Some(PackageMetadata {
        exists: true,
        first_published,
        latest_published,
        maintainer_count,
        has_install_scripts: false,
        install_scripts: vec![],
    })
}

fn parse_release_dates(body: &Value) -> (Option<String>, Option<String>) {
    let releases = match body.get("releases").and_then(|r| r.as_object()) {
        Some(r) => r,
        None => return (None, None),
    };

    let mut all_dates: Vec<String> = Vec::new();

    for (_version, files) in releases {
        if let Some(files) = files.as_array() {
            for file in files {
                if let Some(date) = file
                    .get("upload_time_iso_8601")
                    .and_then(|v| v.as_str())
                {
                    all_dates.push(date.to_string());
                }
            }
        }
    }

    all_dates.sort();

    let first = all_dates.first().cloned();
    let last = all_dates.last().cloned();
    (first, last)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_requests() {
        let client = make_client();
        let meta = fetch_metadata(&client, "requests").await;
        assert!(meta.is_some(), "requests should exist on PyPI");
        let meta = meta.unwrap();
        assert!(meta.exists);
        assert!(meta.first_published.is_some());
        assert!(meta.latest_published.is_some());
    }
}
