# Using boundform with SvelteKit

## Compatibility

SvelteKit uses SSR by default. Forms are rendered server-side and returned as complete HTML via HTTP GET — boundform works out of the box.

The only exception is pages that explicitly disable SSR:

```js
// +page.ts
export const ssr = false; // ← boundform cannot analyze this page
```

## Basic setup

```bash
npx boundform init
```

Edit `boundform/boundform.yml`:

```yaml
pages:
  - url: "http://localhost:5173/register"
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
```

Run:

```bash
npx boundform --config boundform/boundform.yml
```

## SvelteKit form actions

SvelteKit's [form actions](https://kit.svelte.dev/docs/form-actions) use standard `<form>` elements with `method="POST"`. These forms typically include `name` attributes on all fields (required for `FormData` extraction), making them ideal for boundform.

```svelte
<!-- +page.svelte -->
<form method="POST" action="?/register">
  <input name="email" type="email" required />
  <input name="password" type="password" required minlength="8" />
  <button type="submit">Register</button>
</form>
```

## Authenticated pages

```bash
npx boundform --config boundform/boundform.yml \
  --cookie "session_id=abc123"
```

Check your SvelteKit auth setup for the correct cookie name. Common patterns:
- `session_id` (custom auth)
- `authjs.session-token` (Auth.js adapter)
- `sb-access-token` (Supabase)

## CI Integration

```yaml
steps:
  - uses: actions/checkout@v4

  - name: Install and build
    run: npm ci && npm run build

  - name: Start preview server
    run: npm run preview &

  - name: Wait for server
    run: npx wait-on http://localhost:4173

  - name: Validate forms
    run: npx boundform --config boundform/boundform.yml
```

## Troubleshooting

### Page returns empty HTML

Check if the page has `export const ssr = false` in `+page.ts`. If so, you need to use the [SPA capture workflow](react-spa.md) instead.

### Form action URL in identifier

SvelteKit forms use `action="?/register"` which becomes the form identifier. Use `index: 0` to avoid this:

```yaml
forms:
  - index: 0        # ✓ Simple and reliable
    fields: ...
```
