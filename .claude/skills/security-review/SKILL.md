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

Start Codex review as a background process. The `codex review` command accepts EITHER a scope flag (`--uncommitted`, `--base`) OR a prompt, but NOT both. Use the prompt form to direct Codex to read AGENTS.md:

```bash
codex review "Read the file AGENTS.md in this repository root. It contains a detailed security review checklist. Follow every section systematically. Report all findings with severity (Critical/High/Medium/Low/Info), file paths, line numbers, and fix recommendations." 2>&1 | tee codex-security-review.md &
CODEX_PID=$!
```

**If Codex fails** (authentication error, not installed, etc.), log the error and continue with Claude-only review. Common failures:
- `refresh_token_reused` → user needs to run `codex logout && codex login`
- `command not found` → install with `npm install -g @openai/codex`
- `--uncommitted cannot be used with PROMPT` → this is expected, use the prompt-only form above

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

After Claude's review is complete, check if Codex has finished:

```bash
kill -0 $CODEX_PID 2>/dev/null && echo "Codex still running..." || echo "Codex finished"
cat codex-security-review.md 2>/dev/null || echo "No Codex output available"
```

If Codex is still running, inform the user and suggest checking `codex-security-review.md` later.

### Step 4: Generate comparison report

```markdown
## Security Review Summary

### Claude Code Findings
| # | Severity | Category | Issue | File |
|---|----------|----------|-------|------|
| 1 | HIGH     | SSRF     | ...   | ...  |

### Codex Findings
(from codex-security-review.md, or "Codex unavailable — auth expired / not installed")

### Cross-Agent Analysis
- Findings detected by both agents (high confidence)
- Findings unique to Claude
- Findings unique to Codex

### Top 3 Priority Fixes
1. ...
2. ...
3. ...
```

### Step 5: Clean up

```bash
rm -f codex-security-review.md
```

## Reference

The full security checklist is in `AGENTS.md` at the project root. Both Claude and Codex use the same checklist for consistent coverage.
