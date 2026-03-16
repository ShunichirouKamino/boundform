---
name: spa-analyze
description: Capture and analyze SPA/CSR form pages using Playwright or Chrome DevTools, then auto-generate boundform YAML config. Use this skill when the user wants to analyze a client-side rendered page, capture SPA HTML for boundform, generate YAML from a live page, or set up boundform for React/Vue/Svelte apps that don't use SSR. Also trigger when the user mentions "SPA", "CSR", "client-side rendered forms", or asks how to use boundform with Vite/CRA apps.
---

# SPA Form Analyzer

This skill captures rendered HTML from SPA/CSR pages and auto-generates boundform YAML config. It bridges the gap between client-side rendered apps and boundform's static HTML analysis.

## Usage

```
/spa-analyze <url> [--cookie "..."] [--method playwright|devtools]
```

- `url`: The page URL to analyze (must be running/accessible)
- `--cookie`: Optional session cookie for authenticated pages
- `--method`: Capture method — `playwright` (default) or `devtools` (uses Chrome DevTools MCP)

## Workflow

### Step 1: Check prerequisites

Verify the capture tool is available:

**For Playwright (default):**
```bash
npx playwright --version 2>/dev/null || echo "NOT_INSTALLED"
```

If not installed, offer to install:
```bash
npx playwright install chromium
```

**For Chrome DevTools MCP:**
Check if the `mcp__chrome-devtools__take_snapshot` tool is available. If not, fall back to Playwright.

### Step 2: Capture rendered HTML

**Playwright method:**

Create and run a capture script:

```bash
mkdir -p boundform/rendered

node -e "
const { chromium } = require('playwright');
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  // Add cookies if provided
  // await page.context().addCookies([...]);
  await page.goto('${URL}', { waitUntil: 'networkidle' });
  // Wait for forms to render
  await page.waitForSelector('form', { timeout: 10000 }).catch(() => {});
  const html = await page.content();
  require('fs').writeFileSync('boundform/rendered/${FILENAME}.html', html);
  await browser.close();
})();
"
```

The `waitUntil: 'networkidle'` ensures all JS has executed and forms are rendered. The `waitForSelector('form')` adds an extra safety wait for form elements.

**Chrome DevTools MCP method:**

Use the MCP tools in this order:
1. `mcp__chrome-devtools__navigate_page` — navigate to the URL
2. `mcp__chrome-devtools__wait_for` — wait for `form` selector
3. `mcp__chrome-devtools__evaluate_script` — extract `document.documentElement.outerHTML`
4. Save the HTML to `boundform/rendered/{filename}.html`

### Step 3: Analyze the captured HTML

Read the saved HTML file and extract form information. Either:

**Option A: Use boundform directly (if installed)**
```bash
npx boundform --config /dev/null 2>&1  # Check if available
```

If available, create a minimal YAML pointing to the file and run it to see what's detected.

**Option B: Analyze manually by reading the HTML**

Read the captured HTML file and look for:
- `<form>` elements — count them, note their attributes
- `<input>`, `<textarea>`, `<select>` elements within each form
- For each field: `name`/`id`, `type`, `required`, `min`, `max`, `minlength`, `maxlength`, `pattern`, `step`
- Skip `aria-hidden="true"` elements (component library internals)
- Skip `type="hidden"`, `type="checkbox"`, `type="radio"`, `type="submit"`

Present the analysis to the user:

```
Found 1 form(s) on http://localhost:5173/register

Form #0 (index: 0)
  Fields:
    - email (type=email, required)
    - password (type=password, required, minlength=8)
    - username (type=text, required, maxlength=50)

  Skipped (no name/id):
    - 2x input[type=checkbox] (aria-hidden)
    - 1x select (aria-hidden)
```

### Step 4: Generate YAML config

Generate a boundform YAML config using `url` as the local file path:

```yaml
pages:
  - url: "boundform/rendered/register.html"
    forms:
      - index: 0
        fields:
          email:
            type: email
            required: true
          password:
            type: password
            required: true
            minlength: 8
          username:
            type: text
            required: true
            maxlength: 50
```

### Step 5: Save and integrate

**If `boundform/boundform.yml` already exists:**
- Show the generated YAML and ask the user if they want to append it to the existing file or create a separate file.

**If it doesn't exist:**
- Create `boundform/boundform.yml` with the generated config.

Also save a capture script for CI use:

```bash
# boundform/capture-spa.sh
#!/bin/bash
# Capture SPA pages for boundform analysis
# Run this before boundform validation in CI

URL="${1:-http://localhost:5173/register}"
OUTPUT="${2:-boundform/rendered/register.html}"

mkdir -p "$(dirname "$OUTPUT")"

npx playwright evaluate \
  --url "$URL" \
  "document.documentElement.outerHTML" > "$OUTPUT"

echo "Captured: $OUTPUT"
```

### Step 6: Verify

Run boundform against the generated config:

```bash
npx boundform --config boundform/boundform.yml
```

Show the results and ask if the user wants to adjust anything.

## CI Integration

For CI, the user needs to capture HTML before running boundform:

```yaml
# .github/workflows/form-check.yml
steps:
  - name: Start app
    run: npm start &

  - name: Wait for app
    run: npx wait-on http://localhost:5173

  - name: Capture SPA pages
    run: bash boundform/capture-spa.sh

  - name: Validate forms
    run: npx boundform --config boundform/boundform.yml
```

## Multiple pages

If the user wants to analyze multiple pages, run the capture for each:

```
/spa-analyze http://localhost:5173/register
/spa-analyze http://localhost:5173/login
/spa-analyze http://localhost:5173/settings --cookie "session=..."
```

Each page gets its own HTML file in `boundform/rendered/` and the YAML entries are appended.

## Troubleshooting

**"0 forms found after capture"**
- The page might need more time to render. Increase the wait time or use `waitUntil: 'networkidle'`.
- Check if the page requires authentication — use `--cookie`.

**"Fields have no name or id"**
- This is a limitation of the source HTML, not the capture. Recommend adding `name`/`id` attributes to inputs.
- Show which fields were skipped and why.
