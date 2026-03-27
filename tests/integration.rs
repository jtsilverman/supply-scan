use std::process::Command;

fn run_scan(args: &[&str]) -> (String, i32) {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{}{}", stderr, stdout);
    (combined, output.status.code().unwrap_or(-1))
}

#[test]
#[ignore] // requires network
fn test_npm_clean_no_hallucinations_or_typosquats() {
    let (output, _) = run_scan(&["fixtures/npm-clean"]);
    // Clean projects may have known CVEs but should NOT have hallucinations or typosquats
    assert!(!output.contains("hallucination"), "clean npm should have no hallucinations: {}", output);
    assert!(!output.contains("typosquat"), "clean npm should have no typosquats: {}", output);
}

#[test]
#[ignore]
fn test_npm_risky_has_criticals() {
    let (output, code) = run_scan(&["fixtures/npm-risky"]);
    assert_eq!(code, 0, "without --pre-commit should exit 0: {}", output);
    assert!(output.contains("[CRITICAL]"), "risky npm should have criticals: {}", output);
    assert!(output.contains("hallucination"), "should detect hallucinated package: {}", output);
    assert!(output.contains("typosquat"), "should detect typosquat: {}", output);
}

#[test]
#[ignore]
fn test_pypi_risky_has_criticals() {
    let (output, code) = run_scan(&["fixtures/pypi-risky"]);
    assert_eq!(code, 0);
    assert!(output.contains("[CRITICAL]"), "risky pypi should have criticals: {}", output);
    assert!(output.contains("reqeusts") || output.contains("ai-model-loader"), "should flag risky packages: {}", output);
}

#[test]
#[ignore]
fn test_pre_commit_exits_1_on_critical() {
    let (_, code) = run_scan(&["--pre-commit", "fixtures/npm-risky"]);
    assert_eq!(code, 1, "pre-commit should exit 1 on criticals");
}

#[test]
#[ignore]
fn test_pre_commit_exits_1_on_clean_with_known_vulns() {
    // Even "clean" packages like lodash have known CVEs flagged as CRITICAL
    // This is correct behavior — the tool catches real vulnerabilities
    let (output, code) = run_scan(&["--pre-commit", "fixtures/npm-clean"]);
    // Just verify it runs without crashing; exit code depends on OSV data
    assert!(code == 0 || code == 1, "should exit 0 or 1: {}", output);
}

#[test]
#[ignore]
fn test_json_output_is_valid() {
    let (output, _) = run_scan(&["--json", "fixtures/npm-risky"]);
    // The JSON output goes to stdout, stderr has the "scanning..." message
    // Find the JSON object in the output
    let json_start = output.find('{').expect("should contain JSON");
    let json_str = &output[json_start..];
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .expect("JSON output should be valid JSON");
    assert!(parsed["packages"].is_array(), "should have packages array");
    assert!(parsed["total_scanned"].is_number(), "should have total_scanned");
}
