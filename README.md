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
| React (Vite, CRA) | No (CSR) | Via workaround* |
| Vue (Vite) | No (CSR) | Via workaround* |

\* CSR-only apps return an empty `<div id="root">` — the form HTML is generated client-side by JavaScript and cannot be extracted with a simple GET request. See below for a workaround.

### Using boundform with SPA / CSR apps

boundform intentionally does not embed a browser engine — that would make it another Playwright. Instead, you can use an external tool to obtain the rendered HTML and pass it to boundform as a local file:

**Step 1: Capture rendered HTML with Playwright**

```bash
# Install Playwright if you haven't
npm install -g playwright

# Capture the fully rendered HTML
npx playwright evaluate \
  --url http://localhost:5173/register \
  "document.documentElement.outerHTML" > rendered.html
```

Or save it from a browser: open the page, right-click, **"Save As" → "Webpage, HTML Only"**.

**Step 2: Point boundform at the saved file**

```yaml
# boundform.yml
pages:
  - url: "rendered.html"    # local file path instead of URL
    forms:
      - index: 0
        fields:
          email:
            type: email
            required: true
```

```bash
boundform --config boundform.yml
```

**CI example (GitHub Actions):**

```yaml
steps:
  - name: Capture SPA HTML
    run: |
      npx playwright evaluate \
        --url http://localhost:5173/register \
        "document.documentElement.outerHTML" > rendered.html

  - name: Validate form constraints
    run: boundform --config boundform.yml
```

This keeps boundform fast and dependency-free while still supporting CSR apps through composition.

## Using with Zod + conform (Next.js)

When using Zod for form validation, constraints are defined in JavaScript and **not automatically reflected in HTML attributes**. Without HTML attributes, boundform has nothing to check.

The recommended approach is to use [conform](https://conform.guide/) to automatically generate HTML attributes from your Zod schema:

```
Zod schema (single source of truth)
  ├→ HTML attributes  ← conform generates automatically
  ├→ Client JS        ← zodResolver handles validation
  └→ Server-side      ← Server Actions validate with Zod
```

### Setup

```tsx
// schema.ts — define once
import { z } from "zod";

export const registerSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8).max(128),
});
```

```tsx
// register/page.tsx — conform auto-generates HTML attributes
import { getFormProps, getInputProps } from "@conform-to/react";
import { parseWithZod } from "@conform-to/zod";

export default function RegisterPage() {
  const [form, fields] = useForm({
    onValidate({ formData }) {
      return parseWithZod(formData, { schema: registerSchema });
    },
  });

  return (
    <form {...getFormProps(form)}>
      {/* conform adds required, type="email" automatically */}
      <input {...getInputProps(fields.email, { type: "email" })} />
      {/* conform adds required, minLength="8", maxLength="128" */}
      <input {...getInputProps(fields.password, { type: "password" })} />
    </form>
  );
}
```

The SSR-rendered HTML will include constraint attributes:

```html
<input type="email" name="email" required="" />
<input type="password" name="password" required="" minlength="8" maxlength="128" />
```

Now boundform can validate these constraints against your YAML spec.

### Why bother with HTML attributes if Zod handles validation?

1. **Progressive enhancement** — forms work even before JS loads (important for Server Actions)
2. **Instant UX** — browser-native validation fires immediately, no JS round-trip
3. **Accessibility** — `required` is announced by screen readers (WCAG compliance)
4. **Defense in depth** — HTML (browser) + Zod (JS) + Server Actions (server) = 3 layers of validation

## Installation

```bash
cargo install boundform
```

Or build from source:

```bash
git clone https://github.com/ShunichirouKamino/boundform.git
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
