---
name: security-review
description: Perform a multi-agent security vulnerability review of the boundform codebase using both Claude Code and OpenAI Codex in parallel. Use this skill whenever the user asks for a security audit, vulnerability check, code review for security, penetration testing guidance, or wants to check for CVEs, SSRF, injection, or supply chain risks. Also trigger when the user mentions OWASP, cargo audit, dependency vulnerabilities, cookie/header handling safety, or wants a cross-agent security review.
---

# Multi-Agent Security Review

This skill performs a comprehensive security review by running two agents in parallel:
1. **Claude Code** — reviews the code inline following the checklist below
2. **OpenAI Codex** — launched as a separate process via `codex review`

Results are persisted to `reviews/NNNN-YYYY-MM-DD/` and committed to git for historical tracking.

## Workflow

### Step 0: Create review directory

Determine the next review number by counting existing directories in `reviews/`:

```bash
# Get next review number (0001, 0002, etc.)
NEXT=$(printf "%04d" $(( $(ls -d reviews/[0-9]* 2>/dev/null | wc -l) + 1 )))
DATE=$(date +%Y-%m-%d)
REVIEW_DIR="reviews/${NEXT}-${DATE}"
mkdir -p "$REVIEW_DIR"
```

All outputs from this review session will be saved into this directory.

### Step 1: Launch Codex in background

Launch Codex review as a background task using the Bash tool with `run_in_background: true`. Use the prompt-only form because `codex review` does not allow `--uncommitted` or `--base` together with a prompt argument.

Direct Codex output into the review directory:

```bash
codex review "Read the file AGENTS.md in this repository root. It contains a detailed security review checklist. Follow every section systematically. Report all findings with severity (Critical/High/Medium/Low/Info), file paths, line numbers, and fix recommendations." 2>&1 | tee ${REVIEW_DIR}/codex-review.md
```

Run this command with `run_in_background: true` so Claude's review can proceed in parallel.

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

Save Claude's findings to `${REVIEW_DIR}/claude-review.md`.

### Step 3: Collect Codex results

After Claude's review is complete, check if the Codex background task has finished. Read `${REVIEW_DIR}/codex-review.md` if it exists. If Codex is still running, inform the user that results will appear when complete.

### Step 4: Generate comparison report

Create `${REVIEW_DIR}/summary.md` with the cross-agent comparison:

```markdown
# Security Review #NNNN — YYYY-MM-DD

## Findings detected by both agents (high confidence)
| Issue | Claude Severity | Codex Severity |
|-------|:-:|:-:|
| ... | ... | ... |

## Findings unique to Claude
| # | Severity | Issue | File |
|---|----------|-------|------|

## Findings unique to Codex
| # | Severity | Issue | File |
|---|----------|-------|------|

## Top 3 Priority Fixes
1. ...
2. ...
3. ...

## Review Metadata
- Claude model: (model from session)
- Codex model: (from codex output header)
- Date: YYYY-MM-DD
- Commit: (current HEAD SHA)
```

### Step 5: Commit the review

Stage and commit the review directory:

```bash
git add ${REVIEW_DIR}/
git commit -m "docs: security review #${NEXT} (${DATE})

Claude + Codex multi-agent review.
Findings: X high, Y medium, Z low."
```

Ask the user if they want to push, or if they want to fix the findings first.

### Final directory structure

```
reviews/
├── 0001-2026-03-16/
│   ├── claude-review.md
│   ├── codex-review.md
│   └── summary.md
├── 0002-2026-03-17/
│   ├── claude-review.md
│   ├── codex-review.md
│   └── summary.md
└── ...
```

## Reference

The full security checklist is in `AGENTS.md` at the project root. Both Claude and Codex use the same checklist for consistent coverage.
