# boundform

**A fast guardrail before E2E** — validate HTML form constraints against a YAML spec, no browser required.

> Not a replacement for Playwright. A high-speed checkpoint you run *before* it.

## Installation

```bash
# Run directly (no install needed)
npx boundform --config boundform.yml

# Or install globally
npm install -g boundform

# Or via Cargo
cargo install boundform
```

### Project setup

```bash
npx boundform init
```

This creates:
- `boundform/boundform.yml` — starter config template
- `.claude/skills/boundform-guide/` — Claude Code usage guide skill

## Quick Start

### 1. Define your expected constraints

```yaml
# boundform/boundform.yml
pages:
  - url: "http://localhost:3000/register"
    forms:
      - index: 0
        fields:
          email:
            type: email
            required: true
          password:
            type: password
            required: true
            minlength: 10
```

### 2. Run

```bash
npx boundform --config boundform/boundform.yml
```

### 3. Output

```
[index=0] http://localhost:3000/register
  email
    ✓ type = email
    ✓ required = true
  password
    ✓ type = password
    ✓ required = true
    ✓ minlength = 10

1 page(s), 1 form(s), all 5 checks passed
```

## What This Is (and Isn't)

**It does:**
- Validate HTML5 constraint attributes (`required`, `min`, `max`, `minlength`, `maxlength`, `pattern`, `type`, `step`)
- Detect constraint drift between deploys
- Support authenticated pages via `--cookie` / `--header`
- Run in CI as a fast, zero-dependency check

**It does NOT:**
- Execute JavaScript or render client-side DOM
- Replace browser-based E2E testing
- Validate server-side business rules (e.g., "email must be unique")
- Inspect fields without a `name` or `id` attribute

## CLI Reference

```bash
npx boundform [OPTIONS]

Options:
  --config <path>     Path to YAML config [default: boundform.yml]
  --format <format>   Output format: terminal | json [default: terminal]
  --cookie <value>    Cookie to attach (repeatable)
  --header <value>    Custom header (repeatable)
  -V, --version       Print version
  -h, --help          Print help
```

## Authenticated Pages

```bash
# Auth.js / NextAuth session cookie
npx boundform --config boundform/boundform.yml \
  --cookie "authjs.session-token=eyJhbGc..."

# Bearer token
npx boundform --config boundform/boundform.yml \
  --header "Authorization: Bearer eyJhbGc..."
```

## YAML Config Reference

### Form matching

| Method | YAML key | When to use |
|--------|----------|-------------|
| Position | `index: 0` | Single form pages (recommended default) |
| CSS selector | `selector: "form.login"` | Multiple forms, need precision |
| Identifier | `id: "login-form"` | Form has `name` or `id` attr |

### Supported constraints

| Attribute | Example |
|-----------|---------|
| `type` | `type: email` |
| `required` | `required: true` |
| `min` / `max` | `min: 0`, `max: 150` |
| `minlength` / `maxlength` | `minlength: 8`, `maxlength: 128` |
| `pattern` | `pattern: "[a-zA-Z0-9]+"` |
| `step` | `step: 0.01` |

## Compatibility

| Framework | Compatible | Guide |
|-----------|:----------:|-------|
| Next.js (App Router) | Yes | [docs/usage/nextjs.md](docs/usage/nextjs.md) |
| SvelteKit | Yes | [docs/usage/sveltekit.md](docs/usage/sveltekit.md) |
| Nuxt | Yes | — |
| Rails / Laravel / Django | Yes | — |
| React (Vite, CRA) | Via Playwright | [docs/usage/react-spa.md](docs/usage/react-spa.md) |
| Vue (Vite) | Via Playwright | [docs/usage/react-spa.md](docs/usage/react-spa.md) |

## Prerequisites for Your HTML

Fields must have a `name` or `id` attribute to be detected:

```html
<!-- ✓ Detected -->
<input type="email" name="email" required />
<input type="text" id="username" required maxlength="50" />

<!-- ✗ NOT detected — no name or id -->
<input type="text" pattern="[0-9]*" />
```

Component libraries (shadcn/ui, Radix) render hidden `aria-hidden="true"` elements — these are intentionally ignored.

## CI Integration

```yaml
# GitHub Actions
- name: Form constraint check
  run: npx boundform --config boundform/boundform.yml --format json
```

## Development

Uses [Dev Containers](https://containers.dev/) with mise for toolchain management.

```bash
cargo test           # Run tests
cargo clippy         # Lint
cargo fmt            # Format
```

## Documentation

- [Next.js usage guide](docs/usage/nextjs.md) — SSR, Auth.js, Zod + conform
- [SvelteKit usage guide](docs/usage/sveltekit.md) — SSR, form actions
- [React SPA usage guide](docs/usage/react-spa.md) — Playwright capture workflow
- [Architecture decisions](docs/adr/) — ADR-0001 through ADR-0005

## License

MIT
