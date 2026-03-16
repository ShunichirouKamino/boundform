---
name: boundform-guide
description: Usage guide for boundform — a Rust CLI that validates HTML form constraints against a YAML spec. Use this skill whenever the user asks how to use boundform, write YAML configs, configure form matching, set up authentication with cookies, debug missing fields, troubleshoot form detection issues, or asks about boundform's limitations and compatibility. Also trigger when the user mentions form validation, constraint checking, SSR form testing, or boundary-value testing for HTML forms.
---

# boundform Usage Guide

boundform validates HTML form constraints against a YAML specification. It fetches SSR-rendered pages via HTTP GET, extracts form attributes, and compares them to expected values — no browser required.

## Quick Start

```bash
# Validate forms against a YAML spec
boundform --config boundform/boundform.yml

# With authentication (e.g., Auth.js session cookie)
boundform --config boundform/boundform.yml --cookie "authjs.session-token=eyJ..."

# JSON output
boundform --config boundform/boundform.yml --format json
```

## CLI Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--config <path>` | `boundform.yml` | Path to YAML config file |
| `--format <format>` | `terminal` | Output format: `terminal` or `json` |
| `--cookie <value>` | — | Cookie to attach to requests (repeatable) |
| `--header <value>` | — | Custom header, e.g. `"Authorization: Bearer xxx"` (repeatable) |

## YAML Config Format

```yaml
pages:
  - url: "http://localhost:3000/register"  # URL or local file path
    forms:
      - index: 0          # Match form by position (0-based)
        fields:
          email:           # Field name (from name attr, or id fallback)
            type: email
            required: true
          password:
            type: password
            required: true
            minlength: 10
            maxlength: 128
```

### Form matching

Forms can be identified three ways (in priority order):

| Method | YAML key | When to use |
|--------|----------|-------------|
| CSS selector | `selector: "form.login"` | Multiple forms, need precision |
| Position | `index: 0` | Simple pages, single form |
| Identifier | `id: "login-form"` | Form has `name` or `id` attr |

Use `index: 0` as the default — it works for most single-form pages and avoids issues with framework-generated attributes (e.g., Next.js injects JavaScript into `action` attributes).

For details on why these three methods exist, read `docs/adr/0002-form-identifier-by-selector-and-index.md`.

### Field constraints

All constraint fields are optional. Only specified constraints are checked — omitting a field means "don't check this."

```yaml
fields:
  username:
    type: text              # Input type (text, email, number, password, url, tel, date, etc.)
    required: true           # true = must be required, false = must NOT be required
    min: 0                   # Numeric min value
    max: 150                 # Numeric max value
    minlength: 3             # Minimum string length
    maxlength: 50            # Maximum string length
    pattern: "[a-zA-Z0-9]+"  # Regex pattern attribute
    step: 1                  # Numeric step value
```

### Multiple pages and forms

```yaml
pages:
  - url: "http://localhost:3000/login"
    forms:
      - index: 0
        fields:
          email:
            type: email
            required: true

  - url: "http://localhost:3000/register"
    forms:
      - index: 0
        fields:
          username:
            type: text
            required: true
          email:
            type: email
            required: true
```

## Authentication

Protected pages require authentication cookies or headers.

### Auth.js / NextAuth

```bash
# Get cookie from browser DevTools: Application → Cookies → authjs.session-token
boundform --config boundform/boundform.yml \
  --cookie "authjs.session-token=eyJhbGc..."
```

Important: authenticated users accessing `/login` or `/register` may get redirected to `/dashboard`. Split configs into public and authenticated if needed:

```bash
# Public pages (no cookie)
boundform --config boundform-public.yml

# Authenticated pages (with cookie)
boundform --config boundform-auth.yml --cookie "authjs.session-token=..."
```

For design decisions about authentication, read `docs/adr/0003-authentication-support.md`.

### Bearer token / API auth

```bash
boundform --config boundform/boundform.yml \
  --header "Authorization: Bearer eyJhbGc..."
```

## SPA / CSR Workaround

boundform cannot render JavaScript — CSR-only apps return empty HTML. Use Playwright to capture rendered HTML, then validate the saved file:

```bash
# Capture rendered HTML
npx playwright evaluate \
  --url http://localhost:5173/register \
  "document.documentElement.outerHTML" > rendered.html

# Validate the captured HTML
# In your YAML, use the file path instead of URL:
# pages:
#   - url: "rendered.html"
```

## Troubleshooting

### "0 field(s) found" or "Missing fields"

**Cause**: Fields lack both `name` and `id` attributes.

boundform identifies fields by `name`, falling back to `id`. Fields without either are invisible.

```html
<!-- ✓ Detected -->
<input name="email" type="email" required />
<input id="email" type="email" required />

<!-- ✗ Not detected -->
<input type="email" required />
```

**Fix**: Add `name` or `id` to your form inputs. This is also good for accessibility.

### "form not found in HTML"

**Cause**: Form identifier mismatch between YAML and actual HTML.

- If using `id:`, check the form's actual `name`/`id`/`action` attribute
- If using `index:`, count the `<form>` tags (0-based). Framework wrappers may add extra forms.
- Use `curl <url> | grep '<form'` to see what forms exist

### Login page shows up instead of protected page

**Cause**: Missing or expired cookie.

- Get a fresh cookie from browser DevTools
- Make sure you're passing `--cookie` flag
- Check if the cookie name is correct (`authjs.session-token`, `next-auth.session-token`, etc.)

### Radix / shadcn hidden elements

Component libraries like Radix render hidden `<select>` and `<input type="checkbox">` with `aria-hidden="true"`. These are internal implementation details, not real form controls. boundform correctly ignores them.

### Constraint present in source code but missing in HTML

React JSX attributes (e.g., `maxLength={50}`) may not always render as HTML attributes. This is a known observation — see `docs/DESIGN_DECISIONS.md` section "Observation: Attribute Drift."

## Compatibility

| Framework | Compatible | Notes |
|-----------|:----------:|-------|
| Next.js (App Router) | Yes | `'use client'` components still SSR |
| SvelteKit | Yes | SSR by default |
| Nuxt | Yes | SSR by default |
| Rails / Laravel / Django | Yes | Traditional SSR |
| React (Vite) | Via file | Save HTML first, validate file |
| Vue (Vite) | Via file | Save HTML first, validate file |

## Architecture References

For deeper understanding of design decisions:
- `docs/adr/0001-form-identifier-resolution.md` — How forms are identified
- `docs/adr/0002-form-identifier-by-selector-and-index.md` — Index and selector matching
- `docs/adr/0003-authentication-support.md` — Cookie and header auth
- `docs/DESIGN_DECISIONS.md` — Field name fallback, scope boundaries, SPA approach
