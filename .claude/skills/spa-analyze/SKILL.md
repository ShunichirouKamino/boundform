---
name: spa-analyze
description: Capture and analyze SPA/CSR form pages using Playwright or Chrome DevTools, then auto-generate boundform YAML config. Use this skill when the user wants to analyze a client-side rendered page, capture SPA HTML for boundform, generate YAML from a live page, or set up boundform for React/Vue/Svelte apps that don't use SSR. Also trigger when the user mentions "SPA", "CSR", "client-side rendered forms", or asks how to use boundform with Vite/CRA apps.
---

# SPA Form Analyzer

This skill captures rendered HTML from SPA/CSR pages and auto-generates boundform YAML config. It bridges the gap between client-side rendered apps and boundform's static HTML analysis.

## Usage

```
/spa-analyze <url> [--method playwright|devtools]
```

- `url`: The page URL to analyze (**must start with `http://` or `https://`**)
- `--method`: Capture method — `playwright` (default) or `devtools` (uses Chrome DevTools MCP)

For authenticated pages, the skill will prompt for cookie details interactively rather than accepting them as CLI arguments (to avoid exposure in process listings and shell history).

## Workflow

### Step 0: Validate URL

Before doing anything, verify the URL is safe:

```
if (!url.startsWith("http://") && !url.startsWith("https://")) {
  ERROR: Only http:// and https:// URLs are allowed.
  Reject file://, ftp://, data://, and other schemes.
}
```

This prevents local file read and SSRF via non-HTTP schemes.

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

Write a capture script to a file first (not inline) to avoid shell injection:

```bash
mkdir -p boundform/rendered
```

Create `boundform/rendered/capture.js`:

```javascript
const { chromium } = require('playwright');

const url = process.env.BOUNDFORM_CAPTURE_URL;
const output = process.env.BOUNDFORM_CAPTURE_OUTPUT;
const cookieHeader = process.env.BOUNDFORM_CAPTURE_COOKIE || '';

if (!url || !output) {
  console.error('BOUNDFORM_CAPTURE_URL and BOUNDFORM_CAPTURE_OUTPUT must be set');
  process.exit(1);
}

// Validate URL scheme
if (!url.startsWith('http://') && !url.startsWith('https://')) {
  console.error('Only http:// and https:// URLs are allowed');
  process.exit(1);
}

(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();

  // Add cookies if provided (format: "name=value; name2=value2")
  if (cookieHeader) {
    const url_obj = new URL(url);
    const cookies = cookieHeader.split(';').map(c => {
      const [name, ...rest] = c.trim().split('=');
      return { name: name.trim(), value: rest.join('='), domain: url_obj.hostname, path: '/' };
    });
    await context.addCookies(cookies);
  }

  const page = await context.newPage();
  await page.goto(url, { waitUntil: 'networkidle' });
  await page.waitForSelector('form', { timeout: 10000 }).catch(() => {});
  const html = await page.content();
  require('fs').writeFileSync(output, html);
  console.log(`Captured: ${output}`);
  await browser.close();
})();
```

Then run it with environment variables (not shell interpolation):

```bash
BOUNDFORM_CAPTURE_URL="<the url>" \
BOUNDFORM_CAPTURE_OUTPUT="boundform/rendered/<filename>.html" \
BOUNDFORM_CAPTURE_COOKIE="<cookie if needed>" \
node boundform/rendered/capture.js
```

The URL and cookie are passed via environment variables rather than shell string interpolation. This eliminates command injection risk from crafted URLs.

**Chrome DevTools MCP method:**

Use the MCP tools in this order:
1. `mcp__chrome-devtools__navigate_page` — navigate to the URL
2. `mcp__chrome-devtools__wait_for` — wait for `form` selector
3. `mcp__chrome-devtools__evaluate_script` — extract `document.documentElement.outerHTML`
4. Save the HTML to `boundform/rendered/{filename}.html`

### Step 3: Analyze the captured HTML

Read the saved HTML file and extract form information:

- `<form>` elements — count them, note their attributes
- `<input>`, `<textarea>`, `<select>` elements within each form
- For each field: `name`/`id`, `type`, `required`, `min`, `max`, `minlength`, `maxlength`, `pattern`, `step`
- Skip `aria-hidden="true"` elements (component library internals)
- Skip `type="hidden"`, `type="checkbox"`, `type="radio"`, `type="submit"`

Present the analysis:

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

Generate a boundform YAML config using the local file path:

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
- Show the generated YAML and ask the user if they want to append or create a separate file.

**If it doesn't exist:**
- Create `boundform/boundform.yml` with the generated config.

### Step 6: Verify

Run boundform against the generated config:

```bash
npx boundform --config boundform/boundform.yml
```

## CI Integration

For CI, use the capture script with environment variables:

```yaml
# .github/workflows/form-check.yml
steps:
  - name: Start app
    run: npm start &

  - name: Wait for app
    run: npx wait-on http://localhost:5173

  - name: Capture SPA pages
    env:
      BOUNDFORM_CAPTURE_URL: http://localhost:5173/register
      BOUNDFORM_CAPTURE_OUTPUT: boundform/rendered/register.html
    run: node boundform/rendered/capture.js

  - name: Validate forms
    run: npx boundform --config boundform/boundform.yml
```

## Troubleshooting

**"0 forms found after capture"**
- The page might need more time to render. Increase the wait time or use `waitUntil: 'networkidle'`.
- Check if the page requires authentication — provide cookie via `BOUNDFORM_CAPTURE_COOKIE` env var.

**"Fields have no name or id"**
- This is a limitation of the source HTML, not the capture. Recommend adding `name`/`id` attributes to inputs.
