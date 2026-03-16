# boundform

**A fast guardrail before E2E** — validate HTML form constraints against a YAML spec, no browser required.

> Not a replacement for Playwright. A high-speed checkpoint you run *before* it.

## What This Is (and Isn't)

`boundform` is a lightweight Rust CLI that fetches rendered HTML, extracts `<form>` constraint attributes, and validates them against a YAML specification.

**It does:**
- Validate HTML5 constraint attributes (`required`, `min`, `max`, `minlength`, `maxlength`, `pattern`, `type`, `step`) against expected values
- Detect constraint drift (e.g., `maxlength` disappeared between deploys)
- Support authenticated pages via `--cookie` / `--header`
- Run in CI as a fast, zero-dependency check

**It does NOT:**
- Execute JavaScript or render client-side DOM
- Replace browser-based E2E testing
- Validate server-side business rules (e.g., "email must be unique")
- Reproduce per-browser Constraint Validation API differences
- Inspect fields without a `name` or `id` attribute (see [Prerequisites for your HTML](#prerequisites-for-your-html))

## How It Works

```
YAML spec + GET request → HTML response → Parse <form> → Compare constraints → Report
```

1. Reads expected form constraints from a YAML config file
2. Fetches HTML from each URL (or reads local files)
3. Parses `<form>` elements and their `<input>`, `<select>`, `<textarea>` fields
4. Compares actual HTML attributes against expected values
5. Reports matches, mismatches, missing fields, and unexpected fields

## Example

### Config (`boundform.yml`)

```yaml
pages:
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
          password:
            type: password
            required: true
            minlength: 10
```

### Run

```bash
boundform --config boundform.yml
```

### Output

```
[index=0] http://localhost:3000/register
  username
    ✓ type = text
    ✓ required = true
  email
    ✓ type = email
    ✓ required = true
  password
    ✓ type = password
    ✓ required = true
    ✓ minlength = 10

1 page(s), 1 form(s), all 7 checks passed
```

## Authenticated Pages

```bash
# Pass a session cookie (e.g., Auth.js / NextAuth)
boundform --config boundform.yml \
  --cookie "authjs.session-token=eyJhbGc..."

# Pass a custom header
boundform --config boundform.yml \
  --header "Authorization: Bearer eyJhbGc..."
```

## Supported Constraints

| HTML Attribute | What is checked |
|---|---|
| `type` | Input type matches expected (text, email, number, etc.) |
| `required` | Presence or absence of the required attribute |
| `min` / `max` | Numeric boundary values |
| `minlength` / `maxlength` | String length boundaries |
| `pattern` | Regex pattern string |
| `step` | Numeric step value |

## Prerequisites for Your HTML

boundform identifies form fields by their `name` attribute, falling back to `id` if `name` is absent. **Fields without either attribute are invisible to boundform.**

```html
<!-- ✓ Detected: has name -->
<input type="email" name="email" required />

<!-- ✓ Detected: has id (common in React/state-managed forms) -->
<input type="text" id="username" required maxlength="50" />

<!-- ✗ NOT detected: no name, no id -->
<input type="text" pattern="[0-9]*" />
```

### Recommendations for testable forms

- **Always set `name` or `id`** on inputs you want to validate. This is also good practice for accessibility (`<label for="...">`) and debugging.
- **Component libraries** (shadcn/ui, Radix, Headless UI) often render hidden internal elements (`aria-hidden="true"`) without `name` or `id`. These are not real form controls and are intentionally ignored.
- **The page must be accessible via GET.** boundform fetches HTML with a simple HTTP GET request. Pages that require POST or multi-step navigation can still be tested if they also respond to direct URL access (which is common with SSR frameworks).

## Compatibility

Works with any framework that returns rendered HTML via HTTP — no browser engine needed.

| Framework | SSR by Default | Compatible |
|---|---|---|
| Next.js (App Router) | Yes | Yes |
| SvelteKit | Yes | Yes |
| Nuxt | Yes | Yes |
| Rails / Laravel / Django | Yes | Yes |
| React (Vite, CRA) | No (CSR) | No* |
| Vue (Vite) | No (CSR) | No* |

\* CSR-only apps return an empty `<div id="root">` — the form HTML is generated client-side by JavaScript and cannot be extracted with a simple GET request.

## Installation

```bash
cargo install boundform
```

Or build from source:

```bash
git clone https://github.com/your-username/boundform.git
cd boundform
cargo build --release
```

## Development

This project uses a [Dev Container](https://containers.dev/) for a consistent development environment.

### Prerequisites

- [Docker](https://www.docker.com/)
- [VS Code](https://code.visualstudio.com/) with [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

### Getting Started

1. Open this project in VS Code
2. When prompted, click **"Reopen in Container"** (or run `Dev Containers: Reopen in Container` from the command palette)
3. The container includes Rust toolchain, Claude Code CLI, and all development dependencies

```bash
cargo test           # Run tests
cargo clippy         # Lint
cargo fmt            # Format
cargo run -- --config boundform.yml  # Run
```

## Output Formats

```bash
# Terminal (default) — human-readable with colors
boundform --config boundform.yml

# JSON — for programmatic consumption
boundform --config boundform.yml --format json
```

## CI Integration

```yaml
# GitHub Actions example
- name: Form constraint check
  run: boundform --config boundform.yml --format json

- name: Upload test results
  uses: actions/upload-artifact@v4
  with:
    name: boundform-report
    path: boundform-report.json
```

## License

MIT
