---
name: security-review
description: Perform a comprehensive security vulnerability review of the boundform codebase. Use this skill whenever the user asks for a security audit, vulnerability check, code review for security issues, penetration testing guidance, or wants to check for CVEs, SSRF, injection, or supply chain risks in the project. Also trigger when the user mentions OWASP, cargo audit, dependency vulnerabilities, or asks about the safety of cookie/header handling.
---

# Security Review for boundform

This skill performs a comprehensive security vulnerability review of the boundform codebase covering Rust code, npm package, and dependencies.

## Review Procedure

Perform each check in order. For each finding, report severity, file location, and recommendation.

### Step 1: Dependency Audit

```bash
# Rust dependencies
cargo audit

# Check for outdated crates with known issues
cargo outdated
```

If `cargo audit` is not installed: `cargo install cargo-audit`.

Review the output and flag any known CVEs. Even if no CVEs are found, review key dependencies for unsafe code:
- `reqwest` — HTTP client, handles user URLs
- `scraper` — HTML parser, processes untrusted HTML
- `serde_yml` — YAML parser, processes user config

### Step 2: Code-level Security Review

Read the following files and check for the vulnerabilities listed. Refer to `AGENTS.md` in the project root for the full checklist — it contains detailed descriptions of what to look for in each file.

**Critical files** (handle untrusted input directly):
1. `src/source.rs` — URL handling, file reading, cookie/header injection
2. `src/config.rs` — YAML deserialization
3. `src/parser.rs` — HTML parsing of external content

**Important files**:
4. `src/reporter.rs` — Output formatting, potential info disclosure
5. `src/main.rs` — Error handling, argument processing
6. `npm/scripts/download-binary.js` — Binary download, integrity verification
7. `npm/bin/boundform.js` — Binary execution
8. `npm/scripts/init.js` — File copy operations

### Step 3: Specific Vulnerability Checks

For each category, read the relevant source files and assess:

| Category | Key Files | What to Check |
|---|---|---|
| SSRF | `src/source.rs` | URL scheme validation, redirect following |
| Path Traversal | `src/source.rs` | Local file access boundaries |
| CRLF Injection | `src/source.rs` | Cookie/header value validation |
| YAML Bomb | `src/config.rs` | Deserialization limits |
| HTML DoS | `src/parser.rs` | Document size/depth limits |
| ReDoS | `src/comparator.rs` | Pattern complexity bounds |
| Supply Chain | `npm/scripts/download-binary.js` | Download integrity, HTTPS enforcement |
| Credential Exposure | all output paths | Cookie values in logs/output |
| Unwrap Panics | all `.rs` files | `.unwrap()` on untrusted input |

### Step 4: Generate Report

Use this format for each finding:

```markdown
### [SEVERITY] Title

**File**: path/to/file.rs:line_number
**Category**: Category name
**Description**: What the vulnerability is
**Impact**: What an attacker could achieve
**Recommendation**: How to fix it
```

Severity: Critical > High > Medium > Low > Info

### Step 5: Summary

End with:
- Total findings by severity
- Top 3 most important issues to fix
- Overall security posture assessment

## Cross-agent Review

This project also has an `AGENTS.md` file in the project root with the same security checklist formatted for OpenAI Codex. When doing a multi-agent review:
1. Run this skill in Claude Code for one perspective
2. Point Codex at the repo — it will read `AGENTS.md` and produce its own review
3. Compare findings from both agents to catch blind spots
