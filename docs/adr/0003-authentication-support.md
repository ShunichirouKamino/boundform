# ADR-0003: Authentication Support for Protected Pages

## Status

Proposed

## Date

2026-03-16

## Context

Many real-world forms live behind authentication. When `boundform` fetches a protected URL with a plain GET request, the server either redirects to a login page or returns empty HTML — making it impossible to validate form constraints.

The target project (mj-mng) uses **Auth.js (NextAuth v5)** with JWT session strategy and a `Credentials` provider. This is representative of a large portion of the Next.js ecosystem. We want a solution that:

1. Works immediately with any authentication mechanism (generic)
2. Provides first-class support for Auth.js specifically (convenient)
3. Is CI-friendly (automatable, no manual browser interaction)

### How Auth.js sessions work

Auth.js stores session tokens in cookies. The typical flow for Credentials login is:

```
1. GET  /api/auth/csrf             → receive CSRF token + cookies
2. POST /api/auth/callback/credentials
       body: { csrfToken, email, password }
                                    → receive Set-Cookie: authjs.session-token=<JWT>
3. GET  /protected-page
       Cookie: authjs.session-token=<JWT>
                                    → receive authenticated HTML
```

Other Auth.js session strategies (database sessions) use the same cookie mechanism — only the token format differs. The login flow is identical from the HTTP perspective.

### OAuth providers cannot be automated

OAuth login (Google, LINE, GitHub, etc.) involves browser redirects to third-party consent screens. Automating this is impractical without a browser. Users with OAuth-only auth must fall back to manually providing cookies.

## Decision (Proposed)

Implement authentication in two layers:

### Layer 1: Generic cookie/header injection (Phase 1)

CLI flags that attach arbitrary cookies or headers to every HTTP request:

```bash
# Pass a cookie directly
boundform validate --config boundform.yml \
  --cookie "authjs.session-token=eyJhbGc..."

# Pass a custom header (e.g., Bearer token for API-based auth)
boundform validate --config boundform.yml \
  --header "Authorization: Bearer eyJhbGc..."

# Multiple cookies/headers
boundform validate --config boundform.yml \
  --cookie "authjs.session-token=eyJ..." \
  --cookie "authjs.csrf-token=abc..."
```

Implementation: add cookies/headers to the `reqwest` client before making GET requests in `source.rs`.

### Layer 2: Auth.js auto-login (Phase 2)

A YAML `auth` section that automates the Credentials login flow:

```yaml
auth:
  provider: authjs
  login_url: "http://localhost:3000"
  credentials:
    email: "test@example.com"
    password: "testpassword123"

pages:
  - url: "http://localhost:3000/settings"
    forms:
      - index: 0
        fields:
          username:
            type: text
            required: true
            maxlength: 50
```

The auto-login flow:

```
1. GET  {login_url}/api/auth/csrf
   → Extract csrfToken from JSON response
   → Store cookies from Set-Cookie header

2. POST {login_url}/api/auth/callback/credentials
   → Send: { csrfToken, ...credentials }
   → Store session cookie from Set-Cookie header

3. Use stored cookies for all subsequent page fetches
```

### Credential security

Sensitive values like passwords should not be hardcoded in YAML. Support environment variable substitution:

```yaml
auth:
  provider: authjs
  login_url: "http://localhost:3000"
  credentials:
    email: "${BOUNDFORM_EMAIL}"
    password: "${BOUNDFORM_PASSWORD}"
```

In CI:

```yaml
# GitHub Actions
env:
  BOUNDFORM_EMAIL: ${{ secrets.TEST_USER_EMAIL }}
  BOUNDFORM_PASSWORD: ${{ secrets.TEST_USER_PASSWORD }}
steps:
  - run: boundform validate --config boundform.yml
```

## Consequences

### Positive

- **Layer 1 works with anything** — Auth.js, Devise (Rails), Laravel Sanctum, custom auth, API tokens. No assumptions about the auth mechanism.
- **Layer 2 removes friction for Auth.js users** — no need to manually copy cookies or write a login script.
- **CI-friendly** — both layers are fully automatable. Environment variable support keeps secrets out of config files.
- **Incremental** — Layer 1 can ship first and provides immediate value. Layer 2 is additive.

### Negative

- **Auth.js-specific logic is a maintenance burden.** If Auth.js changes its endpoint structure, we need to update. Mitigate: the CSRF+callback flow has been stable since NextAuth v4.
- **OAuth users still need manual cookie extraction.** This is a fundamental limitation, not a design flaw.
- **Credential storage in YAML is a security risk if misused.** Mitigate: document env var substitution prominently, add a warning when plaintext credentials are detected.

### Scope boundaries

This ADR intentionally does NOT cover:

- **OAuth auto-login** — requires browser interaction, out of scope
- **Multi-factor authentication** — too many variations, use `--cookie` fallback
- **Session refresh/renewal** — sessions are short-lived enough for a single validation run
- **Auth.js database session strategy differences** — the HTTP flow is identical, only the cookie content differs

## Implementation plan

| Phase | Feature | Effort |
|-------|---------|--------|
| 1 | `--cookie` and `--header` CLI flags | Low |
| 2 | YAML `auth.provider: authjs` auto-login | Medium |
| 3 | Environment variable substitution in YAML | Low |

Phase 1 should ship first. Phase 2 and 3 can follow in any order.
