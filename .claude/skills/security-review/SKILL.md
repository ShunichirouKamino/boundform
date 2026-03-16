---
name: security-review
description: Perform a multi-agent security vulnerability review of the boundform codebase using both Claude Code and OpenAI Codex in parallel. Use this skill whenever the user asks for a security audit, vulnerability check, code review for security, penetration testing guidance, or wants to check for CVEs, SSRF, injection, or supply chain risks. Also trigger when the user mentions OWASP, cargo audit, dependency vulnerabilities, cookie/header handling safety, or wants a cross-agent security review.
---

# Multi-Agent Security Review

This skill performs a comprehensive security review by running two agents in parallel:
1. **Claude Code** — reviews the code inline following the checklist below
2. **OpenAI Codex** — launched as a separate process via `codex review`

The dual-agent approach catches blind spots that a single reviewer might miss.

## Workflow

### Step 1: Launch Codex in background

Launch Codex review as a background task using the Bash tool with `run_in_background: true`. Use the prompt-only form because `codex review` does not allow `--uncommitted` or `--base` together with a prompt argument.

```bash
codex review "Read the file AGENTS.md in this repository root. It contains a detailed security review checklist. Follow every section systematically. Report all findings with severity (Critical/High/Medium/Low/Info), file paths, line numbers, and fix recommendations." 2>&1 | tee codex-security-review.md
```

Run this command with `run_in_background: true` so Claude's review can proceed in parallel. The output file will be available when the background task completes.

**If Codex fails** (authentication error, not installed, etc.), log the error and continue with Claude-only review. Common failures:
- `refresh_token_reused` → user needs to run `codex logout && codex login`
- `command not found` → install with `npm install -g @openai/codex`

### Step 2: Run Claude's review

While Codex runs in the background, perform Claude's own review by reading source files and checking each category.

#### 2a. Dependency audit

Check `Cargo.toml` for dependency versions. If `cargo audit` is available, run it. Flag known-vulnerable or pre-stable dependencies (e.g., `serde_yml 0.0.x`).

#### 2b. Critical file review

Read and analyze these files for vulnerabilities:

| File | What to check |
|------|--------------|
| `src/source.rs` | SSRF (URL scheme, host validation, redirect following), path traversal (local file reads), CRLF injection (cookie/header values), request timeout, response size limits |
| `src/config.rs` | YAML deserialization safety (bombs, alias expansion), config size limits |
| `src/parser.rs` | HTML parsing DoS (document size/depth), memory limits |
| `src/comparator.rs` | Pattern/regex handling (ReDoS risk) |
| `src/reporter.rs` | Information disclosure in output |
| `src/main.rs` | Error handling, credential exposure in CLI args |
| `npm/scripts/download-binary.js` | Binary integrity verification, HTTPS enforcement, redirect validation, cache poisoning |
| `npm/bin/boundform.js` | Binary path sanitization, environment passthrough |
| `npm/scripts/init.js` | Path traversal in file copy, prompt injection via skill files |

For each finding, use this format:

```markdown
### [SEVERITY] Title

**File**: path/to/file:line_number
**Category**: Category name
**Description**: What the vulnerability is
**Impact**: What an attacker could achieve
**Recommendation**: How to fix it
```

### Step 3: Collect Codex results

After Claude's review is complete, check if the Codex background task has finished. Read `codex-security-review.md` if it exists. If Codex is still running, inform the user that results will appear in `codex-security-review.md` when complete — do NOT delete this file.

### Step 4: Generate comparison report

```markdown
## Security Review Summary

### Claude Code Findings
| # | Severity | Category | Issue | File |
|---|----------|----------|-------|------|
| 1 | HIGH     | SSRF     | ...   | ...  |

### Codex Findings
(from codex-security-review.md, or "Codex still running / unavailable")

### Cross-Agent Analysis
- Findings detected by both agents (high confidence)
- Findings unique to Claude
- Findings unique to Codex

### Top 3 Priority Fixes
1. ...
2. ...
3. ...
```

### Step 5: Offer clean up

Ask the user if they want to keep `codex-security-review.md` for reference. Only delete if the user confirms.

## Reference

The full security checklist is in `AGENTS.md` at the project root. Both Claude and Codex use the same checklist for consistent coverage.
