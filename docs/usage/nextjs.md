# Using boundform with Next.js

## Compatibility

Next.js App Router uses SSR by default — even `'use client'` components are server-side rendered on the initial request. This means boundform can fetch and parse forms directly via HTTP GET.

## Basic setup

```bash
npx boundform init
```

Edit `boundform/boundform.yml`:

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

Run:

```bash
npx boundform --config boundform/boundform.yml
```

## Authenticated pages (Auth.js / NextAuth)

Protected pages require a session cookie. Get it from browser DevTools:

1. Open your app and log in
2. F12 → **Application** → **Cookies** → `localhost`
3. Copy the value of `authjs.session-token`

```bash
npx boundform --config boundform/boundform.yml \
  --cookie "authjs.session-token=eyJhbGc..."
```

### Important: split public and auth configs

Authenticated users accessing `/login` or `/register` may get redirected to `/dashboard`. Use separate configs:

```bash
# Public pages (no cookie)
npx boundform --config boundform/boundform-public.yml

# Protected pages (with cookie)
npx boundform --config boundform/boundform-auth.yml \
  --cookie "authjs.session-token=eyJ..."
```

## Form identifier: React Server Actions

Next.js injects `action="javascript:throw new Error('React form unexpectedly submitted.')"` into `<form>` tags when using Server Actions. Use `index: 0` instead of `id:` to avoid this:

```yaml
forms:
  # ✓ Works
  - index: 0
    fields: ...

  # ✗ Fragile — framework-generated action string
  - id: "javascript:throw new Error('React form unexpectedly submitted.')"
    fields: ...
```

## CI Integration

```yaml
# .github/workflows/form-check.yml
name: Form Constraints
on: [push]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Start app
        run: npm start &
        env:
          DATABASE_URL: ${{ secrets.DATABASE_URL }}

      - name: Wait for app
        run: npx wait-on http://localhost:3000

      - name: Check public forms
        run: npx boundform --config boundform/boundform-public.yml

      - name: Check auth forms
        run: |
          npx boundform --config boundform/boundform-auth.yml \
            --cookie "authjs.session-token=${{ secrets.TEST_SESSION_TOKEN }}"
```

## Troubleshooting

### "0 field(s) found"

- Check if fields have `name` or `id` attributes (boundform falls back to `id`)
- If using shadcn/ui, the hidden `aria-hidden="true"` elements are ignored — your visible fields need `name`/`id`

### "form not found in HTML"

- Use `curl http://localhost:3000/your-page | grep '<form'` to see what forms exist
- Count forms (0-based) and adjust `index:` accordingly
- Framework wrapper forms may add extra `<form>` tags before yours

### Login page shows instead of protected page

- Cookie is missing or expired — get a fresh one from DevTools
- Check cookie name: `authjs.session-token` (Auth.js v5) vs `next-auth.session-token` (NextAuth v4)
