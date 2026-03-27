use super::Vulnerability;
use serde_json::{json, Value};
use std::time::Duration;

pub async fn query_vulnerabilities(
    client: &reqwest::Client,
    name: &str,
    ecosystem: &str,
) -> Vec<Vulnerability> {
    let body = json!({
        "package": {
            "name": name,
            "ecosystem": ecosystem,
        }
    });

    let resp = client
        .post("https://api.osv.dev/v1/query")
        .json(&body)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let data: Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let vulns = match data.get("vulns").and_then(|v| v.as_array()) {
        Some(v) => v,
        None => return vec![],
    };

    vulns
        .iter()
        .filter_map(|v| {
            let id = v.get("id")?.as_str()?.to_string();
            let summary = v
                .get("summary")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            // Try severity from the severity array first, then database_specific
            let severity = extract_severity(v);

            Some(Vulnerability {
                id,
                summary,
                severity,
            })
        })
        .collect()
}

fn extract_severity(vuln: &Value) -> Option<String> {
    // Try the top-level severity array (CVSS-based)
    if let Some(sevs) = vuln.get("severity").and_then(|s| s.as_array()) {
        if let Some(first) = sevs.first() {
            if let Some(score) = first.get("score").and_then(|s| s.as_str()) {
                return Some(score.to_string());
            }
        }
    }

    // Try database_specific.severity
    if let Some(sev) = vuln
        .pointer("/database_specific/severity")
        .and_then(|s| s.as_str())
    {
        return Some(sev.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    #[tokio::test]
    #[ignore]
    async fn test_query_express_npm() {
        let client = make_client();
        let vulns = query_vulnerabilities(&client, "express", "npm").await;
        assert!(
            !vulns.is_empty(),
            "express should have known vulnerabilities"
        );
        assert!(!vulns[0].id.is_empty());
    }
}
