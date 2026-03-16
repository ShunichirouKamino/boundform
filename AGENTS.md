# AGENTS.md — boundform

## Project Overview

boundform is a Rust CLI tool that validates HTML form constraints against a YAML specification. It fetches SSR-rendered pages via HTTP GET, extracts form attributes, and compares them to expected values.

## Security Review Instructions

When reviewing this project for security vulnerabilities, check ALL of the following areas systematically. Report findings with severity (Critical / High / Medium / Low / Info) and provide specific file paths and line numbers.

### 1. Dependency Audit (Rust crates)

Run `cargo audit` to check for known vulnerabilities in dependencies.

Key dependencies to scrutinize:
- `reqwest` (HTTP client) — handles user-supplied URLs and cookies
- `scraper` (HTML parser) — parses untrusted HTML from external sources
- `serde_yml` (YAML parser) — parses user-supplied config files
- `clap` (CLI parser) — handles user input

Check for:
- Known CVEs in any dependency
- Outdated dependencies with available security patches
- Dependencies with unsafe code that handle untrusted input

### 2. Input Validation & Injection

#### URL handling (`src/source.rs`)
- Can a user supply a malicious URL (e.g., `file:///etc/passwd`, `gopher://`, SSRF to internal services)?
- Is there URL scheme validation (only `http://` and `https://` should be allowed)?
- Are redirects followed? If so, can an attacker redirect to internal services?

#### YAML config parsing (`src/config.rs`)
- Can a malicious YAML file cause deserialization attacks (YAML bombs, alias expansion)?
- Is there a size limit on config files?
- Can `url` fields in config reference local files outside the project?

#### Cookie/Header injection (`src/source.rs`)
- Can `--cookie` or `--header` values inject additional headers (CRLF injection)?
- Are header names and values properly validated?

#### HTML parsing (`src/parser.rs`)
- Can maliciously crafted HTML cause DoS (deeply nested elements, extremely large documents)?
- Are there memory limits on parsed documents?

### 3. Error Handling & Information Disclosure

- Check all `.unwrap()` calls — these can cause panics on untrusted input
- Are error messages leaking sensitive information (file paths, internal URLs)?
- Does the JSON output format (`--format json`) expose anything unexpected?

Files to check:
- `src/error.rs` — custom error types
- `src/reporter.rs` — output formatting
- `src/main.rs` — top-level error handling

### 4. File System Access

- `src/source.rs` reads local files when the source is not a URL
- Can path traversal (`../../etc/passwd`) be used to read arbitrary files?
- Is there symlink following that could escape intended directories?

### 5. npm Package Security (`npm/`)

#### Supply chain
- `npm/scripts/download-binary.js` downloads binaries from GitHub Releases
- Is the download URL validated (could it be redirected to a malicious binary)?
- Is there integrity/checksum verification of downloaded binaries?
- Is HTTPS enforced for all downloads?

#### Binary execution
- `npm/bin/boundform.js` spawns the downloaded binary as a child process
- Is the binary path sanitized before execution?
- Could a malicious binary be placed in the cache directory?

#### Init script
- `npm/scripts/init.js` copies files to the user's project
- Is there path traversal protection in the copy logic?

### 6. Sensitive Data Handling

- Session cookies are passed via `--cookie` CLI flag — are they logged anywhere?
- Are cookies visible in process listings (`ps aux`)?
- Does `--format json` output include cookie values?
- Are there any hardcoded secrets or credentials in the codebase?

### 7. Regex / Pattern Handling

- `src/comparator.rs` compares pattern attributes
- If patterns are compiled as regex, is there ReDoS risk?
- Are user-supplied patterns bounded in complexity/length?

### 8. Network Security

- Is TLS certificate validation enabled for HTTPS requests?
- Is there a request timeout to prevent hanging on slow/malicious servers?
- Are there limits on response size to prevent memory exhaustion?

## Report Format

For each finding, use this format:

```
### [SEVERITY] Title

**File**: src/source.rs:18
**Category**: Input Validation
**Description**: Brief description of the vulnerability
**Impact**: What an attacker could do
**Recommendation**: How to fix it
**References**: CVE numbers, OWASP references if applicable
```

Severity levels:
- **Critical**: Remote code execution, arbitrary file read/write
- **High**: SSRF, credential exposure, significant DoS
- **Medium**: Information disclosure, limited injection
- **Low**: Minor issues, defense-in-depth improvements
- **Info**: Best practice suggestions, code quality

## Project Structure

```
src/
├── main.rs        — CLI entrypoint, argument parsing
├── source.rs      — HTTP fetching and file reading (CRITICAL for security)
├── parser.rs      — HTML parsing with scraper
├── config.rs      — YAML config loading
├── comparator.rs  — Expected vs actual constraint comparison
├── model.rs       — Data structures
├── reporter.rs    — Output formatting (terminal, JSON)
└── error.rs       — Error types

npm/
├── bin/boundform.js           — Entry point, spawns binary
├── scripts/download-binary.js — Downloads binary from GitHub Releases
├── scripts/init.js            — Project initialization
└── package.json
```
