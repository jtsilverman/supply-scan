# AI Supply Chain Scanner — MVP Spec

## Overview

Rust CLI that scans a project's dependency tree and flags packages that look suspicious — typosquats of popular packages, known malicious packages, packages with risky install scripts, and packages that may have been hallucinated by an AI code generator. The AI hallucination angle ("slopsquatting") is the twist: LLMs hallucinate fake package names ~20% of the time, and attackers register those names with malware. No existing open-source CLI focuses on this.

## Scope

- **Timebox:** 2 days
- **Building:**
  - Parse `package.json` (npm) and `requirements.txt` / `pyproject.toml` (PyPI) dependency files
  - Check each package against OSV.dev for known vulnerabilities
  - Detect typosquats via Levenshtein distance against a curated list of top 500 npm + PyPI packages
  - Flag suspicious signals: very new packages (<30 days), low download counts, install scripts in package.json, single maintainer
  - Check for likely AI hallucinations: packages that don't exist in the registry at all
  - Output: colored terminal report with risk levels (CRITICAL / WARNING / INFO) per package
  - `--json` flag for machine-readable output
  - `--pre-commit` mode that exits non-zero if any CRITICAL findings (for CI integration)
- **Not building:** Web UI, GitHub Actions integration, runtime analysis, binary inspection, auto-fix, PyPI download stats (requires BigQuery), recursive transitive dependency resolution
- **Ship target:** crates.io + GitHub release binaries

## Stack

- **Language:** Rust (security tooling credibility, fast, already proven with dep-diet)
- **Key crates:** reqwest (HTTP), serde/serde_json (parsing), clap (CLI), tokio (async), colored (terminal output), strsim (edit distance)
- **APIs:** npm registry (free, no auth), PyPI JSON API (free, no auth), OSV.dev (free, no auth)
- **Why Rust:** Security tools should be fast and self-contained. Single binary distribution. Portfolio already has dep-diet in Rust but different domain (dependency analysis vs security scanning).

## Architecture

### File Structure
```
supply-chain-scanner/
  Cargo.toml
  src/
    main.rs           — CLI entry point, arg parsing
    scanner.rs        — Core scan orchestration
    parsers/
      mod.rs
      npm.rs          — Parse package.json
      pypi.rs         — Parse requirements.txt + pyproject.toml
    checks/
      mod.rs
      existence.rs    — Registry existence check (hallucination detection)
      typosquat.rs    — Edit distance against popular packages
      vulnerability.rs — OSV.dev lookup
      signals.rs      — Age, maintainer count, install scripts
    registry/
      mod.rs
      npm.rs          — npm registry API client
      pypi.rs         — PyPI API client
      osv.rs          — OSV.dev API client
    report.rs         — Terminal + JSON output formatting
    popular.rs        — Embedded list of top 500 npm + PyPI packages
  tests/
    integration.rs    — End-to-end scan tests with fixture projects
  fixtures/
    npm-clean/        — Clean npm project (no issues)
    npm-risky/        — npm project with typosquats + suspicious deps
    pypi-clean/       — Clean Python project
    pypi-risky/       — Python project with fake/suspicious packages
  README.md
```

### Data Flow
1. Detect project type by scanning for `package.json`, `requirements.txt`, or `pyproject.toml`
2. Parse dependencies into a unified `Package { name, version, ecosystem }` list
3. Run checks concurrently (tokio tasks):
   a. **Existence check:** HEAD request to registry — does this package exist at all?
   b. **Typosquat check:** Levenshtein distance < 3 against top 500 packages
   c. **Vulnerability check:** POST to OSV.dev batch API
   d. **Signal check:** Fetch full metadata, flag age < 30 days, install scripts, single maintainer
4. Aggregate findings into `ScanReport` with per-package risk levels
5. Render report (terminal or JSON)

### CLI Interface
```
supply-scan [OPTIONS] [PATH]

Arguments:
  [PATH]  Project directory to scan (default: current directory)

Options:
  --json          Output as JSON
  --pre-commit    Exit 1 if any CRITICAL findings
  --ecosystem     Force ecosystem (npm|pypi), otherwise auto-detect
  --no-network    Skip registry checks, only do local analysis
  -v, --verbose   Show all packages, not just flagged ones
```

### Risk Levels
- **CRITICAL:** Package doesn't exist in registry (hallucinated), known malicious (OSV), or exact match to known typosquat
- **WARNING:** Edit distance 1-2 from a popular package, age < 30 days, has install scripts, single maintainer
- **INFO:** Known vulnerability with available fix, deprecated package

## Task List

### Phase 1: Project Setup + Parsers

#### Task 1.1: Project scaffold and CLI
**Files:** `Cargo.toml` (create), `src/main.rs` (create)
**Do:** Initialize Rust project with clap for CLI arg parsing. Define the CLI interface (path, --json, --pre-commit, --ecosystem, --verbose flags). Print parsed args and exit.
**Validate:** `cargo build && cargo run -- --help` shows usage

#### Task 1.2: Dependency parsers
**Files:** `src/parsers/mod.rs` (create), `src/parsers/npm.rs` (create), `src/parsers/pypi.rs` (create)
**Do:** Parse `package.json` (dependencies + devDependencies), `requirements.txt` (name==version lines), and `pyproject.toml` ([project.dependencies] array). Return `Vec<Package>` where Package is `{ name, version, ecosystem }`. Handle missing files gracefully.
**Validate:** Unit tests with inline fixture strings pass for all 3 formats. `cargo test parsers`

### Phase 2: Registry Clients

#### Task 2.1: npm + PyPI registry clients
**Files:** `src/registry/mod.rs` (create), `src/registry/npm.rs` (create), `src/registry/pypi.rs` (create)
**Do:** Implement async HTTP clients. npm: GET `https://registry.npmjs.org/{name}` — extract `time` (publish dates), `maintainers`, `scripts` (from latest version). PyPI: GET `https://pypi.org/pypi/{name}/json` — extract `releases` (dates), `info.author`, `info.requires_dist`. Both return `Option<PackageMetadata>` (None = package doesn't exist). Include 5-second timeout and retry once.
**Validate:** `cargo test registry` — tests hit live APIs for `express` (npm) and `requests` (pypi), verify metadata fields parse correctly

#### Task 2.2: OSV.dev client
**Files:** `src/registry/osv.rs` (create)
**Do:** POST to `https://api.osv.dev/v1/query` with `{"package": {"name": ..., "ecosystem": "npm"|"PyPI"}}`. Parse response into `Vec<Vulnerability>` with id, summary, severity. Support batch queries (up to 1000).
**Validate:** `cargo test osv` — query `express` returns known vulnerabilities

### Phase 3: Security Checks

#### Task 3.1: Existence + hallucination check
**Files:** `src/checks/mod.rs` (create), `src/checks/existence.rs` (create)
**Do:** For each package, check if the registry client returned None. If so, flag as CRITICAL with reason "Package does not exist in registry — possible AI hallucination". This is the core differentiator.
**Validate:** Test with a known non-existent package name, verify CRITICAL finding

#### Task 3.2: Typosquat detection
**Files:** `src/checks/typosquat.rs` (create), `src/popular.rs` (create)
**Do:** Embed a list of top 200 npm + 200 PyPI packages (hardcoded). For each scanned package, compute Levenshtein distance against all popular packages in same ecosystem. Flag WARNING if distance 1-2 and the package is NOT the popular package itself. Include the similar popular package name in the finding.
**Validate:** Test: "expresss" flags as typosquat of "express", "requests" does NOT flag, "reqeusts" flags as typosquat of "requests"

#### Task 3.3: Signal checks (age, scripts, maintainers)
**Files:** `src/checks/signals.rs` (create)
**Do:** Using metadata from registry clients: flag WARNING if package published < 30 days ago, flag WARNING if `scripts.preinstall` or `scripts.postinstall` exist (npm), flag INFO if single maintainer. Combine signals — multiple warnings on same package escalate to CRITICAL.
**Validate:** Test with mock metadata structs

#### Task 3.4: Vulnerability check
**Files:** `src/checks/vulnerability.rs` (create)
**Do:** Map OSV.dev results to findings. CRITICAL if severity high/critical, WARNING if medium, INFO if low. Include CVE ID and summary in finding.
**Validate:** Test with known vulnerable package version

### Phase 4: Scan Orchestration + Report

#### Task 4.1: Scanner orchestration
**Files:** `src/scanner.rs` (create), `src/main.rs` (modify)
**Do:** Wire everything together. Detect project type, parse deps, run all checks concurrently with tokio, collect findings into `ScanReport { packages: Vec<PackageReport> }` where each has the package info + findings. Wire into main.rs CLI.
**Validate:** `cargo run -- fixtures/npm-risky` produces output with findings

#### Task 4.2: Report formatting
**Files:** `src/report.rs` (create)
**Do:** Terminal output: colored risk badges (red CRITICAL, yellow WARNING, blue INFO), package name, finding description. Summary line at end: "X packages scanned, Y critical, Z warnings". JSON mode: structured output matching terminal content. Pre-commit mode: exit code 1 if any CRITICAL.
**Validate:** `cargo run -- fixtures/npm-risky` shows colored output, `cargo run -- --json fixtures/npm-risky` outputs valid JSON, `cargo run -- --pre-commit fixtures/npm-risky` exits 1

### Phase 5: Integration Tests + Fixtures

#### Task 5.1: Test fixtures
**Files:** `fixtures/npm-clean/package.json` (create), `fixtures/npm-risky/package.json` (create), `fixtures/pypi-clean/requirements.txt` (create), `fixtures/pypi-risky/requirements.txt` (create)
**Do:** Create test fixture projects. npm-clean: express, lodash, react. npm-risky: expresss (typosquat), ai-helper-utils-xyz (non-existent/hallucinated), event-stream@3.3.6 (known compromised). pypi-clean: requests, flask, numpy. pypi-risky: reqeusts (typosquat), ai-model-loader-xyz (non-existent), pyyaml (with known vulns).
**Validate:** All fixture files parse correctly

#### Task 5.2: End-to-end integration tests
**Files:** `tests/integration.rs` (create)
**Do:** Run full scan against each fixture directory. Assert: clean fixtures produce 0 CRITICAL findings, risky fixtures produce >= 2 CRITICAL findings (non-existent + typosquat). Test --json output parses as valid JSON. Test --pre-commit exits 1 on risky fixtures.
**Validate:** `cargo test --test integration` passes

### Phase 6: Ship

#### Task 6.1: README + publish
**Files:** `README.md` (create)
**Do:** Write README with: problem statement (AI hallucination supply chain attacks), demo output screenshot, install instructions (cargo install + binary download), usage examples, what it checks, how it works, the hard part (typosquat detection). Add CI badge placeholder. Push to GitHub as `jtsilverman/supply-scan`.
**Validate:** `gh repo view jtsilverman/supply-scan` returns repo info, README renders on GitHub

## The One Hard Thing

**Typosquat detection with low false positives.** Levenshtein distance alone produces too many false positives (e.g., "cors" and "core" are distance 1 but both legitimate). The approach: only flag if (a) edit distance <= 2 AND (b) the similar package is in the top-200 popularity list AND (c) the scanned package itself is NOT in the popularity list. This three-way check dramatically reduces false positives while catching real typosquats like "expresss", "reqeusts", "lodasah".

**Fallback:** If edit distance still produces too many false positives, switch to a stricter heuristic: only flag exact character transpositions and single-character additions/deletions (not substitutions).

## Risks

- **Medium — npm rate limits:** npm registry has undocumented rate limits. Mitigation: add 50ms delay between requests, cache responses, scan deps concurrently but with a semaphore (max 10 concurrent).
- **Low — OSV.dev availability:** Free API, occasionally slow. Mitigation: 5-second timeout, continue scan without vuln data if OSV is down.
- **Low — Popular package list staleness:** Hardcoded top-200 list will age. Mitigation: good enough for MVP, can add auto-update later.
- **Low — Overlap with dep-diet:** Both are Rust CLIs analyzing dependencies. Different domains: dep-diet = bloat/size, supply-scan = security/trust. Clear differentiation in README.
