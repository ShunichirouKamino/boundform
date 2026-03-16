# Claude Security Review — 2026-03-16

**Model**: Claude Opus 4.6 (1M context)
**Commit**: d89b63d (post security fix)

## Findings (7 total)

### [MEDIUM] SSRF: localhost explicitly allowed in SSRF check

**File**: `src/source.rs:76`
**Category**: SSRF
**Description**: The SSRF protection explicitly allows `localhost` and `127.0.0.1` through the loopback check. While intentional (users check local dev servers), a malicious YAML config can still reach local services like databases, admin panels, and cloud metadata proxies bound to localhost.
**Impact**: In a CI environment with untrusted YAML configs, an attacker could probe internal services on the runner.
**Recommendation**: Document as accepted risk for CLI use case. For CI, recommend running in a network-isolated container.

### [MEDIUM] YAML config parsed without size limit

**File**: `src/config.rs:83`
**Category**: Deserialization
**Description**: `load_config` reads the entire YAML file into memory with `std::fs::read_to_string` and then deserializes it without any size limit. A malicious YAML file with deep alias expansion (YAML bomb) could consume excessive memory.
**Impact**: Memory exhaustion if an untrusted config file is provided.
**Recommendation**: Add a file size check before parsing (similar to `read_from_file` in source.rs).

### [MEDIUM] Checksum verification gracefully falls back to no verification

**File**: `npm/scripts/download-binary.js:141`
**Category**: Supply Chain
**Description**: When the `boundform-checksums.sha256` file is unavailable, `fetchChecksum` returns `null` and the binary is accepted without integrity check. An attacker could force this by blocking only the checksums URL.
**Impact**: Binary executed without integrity verification if checksums file is selectively blocked.
**Recommendation**: Make checksum verification mandatory for versions >= 0.1.4. Log a visible warning if integrity was not verified.

### [MEDIUM] Full environment passthrough to spawned binary

**File**: `npm/bin/boundform.js:22`
**Category**: Information Disclosure
**Description**: `spawn(binaryPath, args, { env: process.env })` passes all environment variables including secrets like `NPM_TOKEN`, `GITHUB_TOKEN`, `AWS_SECRET_ACCESS_KEY`.
**Impact**: If the binary were compromised, it would have access to all environment secrets.
**Recommendation**: Remove the explicit `env` option (inherits by default) or filter to necessary variables only.

### [LOW] Windows system path blocklist is incomplete

**File**: `src/source.rs:178`
**Category**: Path Traversal
**Description**: Only `C:\Windows` and `C:\Program` are blocked. Other sensitive paths like `C:\Users\<user>\AppData`, UNC paths (`\\server\share`) are not covered.
**Impact**: Limited — defense-in-depth for a CLI tool.
**Recommendation**: Consider allowlist approach (only paths relative to CWD) instead of blocklist.

### [LOW] Invalid cookies/headers silently swallowed

**File**: `src/source.rs:102,112`
**Category**: Usability/Security
**Description**: When `HeaderValue::from_str` fails, the cookie/header is silently dropped. User gets no warning that authentication data wasn't sent.
**Impact**: User thinks they're authenticated but request goes out without credentials.
**Recommendation**: Log a warning when a cookie or header value is rejected.

### [INFO] Pre-stable dependency: serde_yml 0.0.12

**File**: `Cargo.toml:16`
**Category**: Dependency
**Description**: `serde_yml` version `0.0.12` is pre-1.0. Pre-stable libraries may have less rigorous security review.
**Recommendation**: Monitor for updates. Consider switching to `serde_yaml` if API is compatible.
