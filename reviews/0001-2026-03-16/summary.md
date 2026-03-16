# Security Review #0001 — 2026-03-16

## Findings detected by both agents (high confidence)

| Issue | Claude Severity | Codex Severity |
|-------|:-:|:-:|
| Cookie/credential exposure in CLI args or env passthrough | MEDIUM | P2 |

## Findings unique to Claude

| # | Severity | Issue | File |
|---|----------|-------|------|
| 1 | MEDIUM | localhost explicitly allowed bypasses SSRF protection | `src/source.rs:76` |
| 2 | MEDIUM | YAML config parsed without size limit | `src/config.rs:83` |
| 3 | MEDIUM | Checksum verification falls back to no verification | `npm/scripts/download-binary.js:141` |
| 4 | LOW | Windows system path blocklist incomplete | `src/source.rs:178` |
| 5 | LOW | Invalid cookies/headers silently swallowed | `src/source.rs:102` |
| 6 | INFO | Pre-stable dependency serde_yml 0.0.12 | `Cargo.toml:16` |

## Findings unique to Codex (gpt-5.4)

| # | Severity | Issue | File |
|---|----------|-------|------|
| 1 | P1 | URL injected into shell string (command injection) in spa-analyze | `.claude/skills/spa-analyze/SKILL.md:55` |
| 2 | P1 | No URL scheme validation in spa-analyze (file:// SSRF) | `.claude/skills/spa-analyze/SKILL.md:16` |
| 3 | P2 | Cookie passed on CLI in spa-analyze (process listing exposure) | `.claude/skills/spa-analyze/SKILL.md:13` |

## Top 3 Priority Fixes

1. **spa-analyze command injection** (Codex P1) — URL directly interpolated into JS string. Use environment variables or script arguments.
2. **spa-analyze URL scheme validation** (Codex P1) — No http/https restriction. `file://` allows arbitrary local file read.
3. **YAML config size limit** (Claude MEDIUM) — config.rs reads unlimited file. Add size check before parsing.

## Review Metadata

- Claude model: Claude Opus 4.6 (1M context)
- Codex model: gpt-5.4
- Date: 2026-03-16
- Commit: d89b63d
