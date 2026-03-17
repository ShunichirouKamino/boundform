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

## Using with Zod + conform

When using Zod for validation, constraints exist only as JavaScript objects — the rendered HTML may have **no constraint attributes at all**:

```tsx
// Zod schema
const schema = z.object({
  password: z.string().min(8).max(128),
});

// Without conform: HTML has NO attributes
// <input name="password" />  ← no required, no minlength, no maxlength
```

### Solution: use conform

[conform](https://conform.guide/) automatically generates HTML constraint attributes from Zod schemas:

```tsx
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

The SSR-rendered HTML now includes constraint attributes:

```html
<input type="email" name="email" required="" />
<input type="password" name="password" required="" minlength="8" maxlength="128" />
```

### getZodConstraint の変換対象

conform の `getZodConstraint` が HTML5 属性に変換できるのは以下の Zod メソッドだけです。これ以外の Zod バリデーション（`.regex()`, `.refine()`, `.transform()` など）は HTML 属性に変換されません。

| Zod メソッド | HTML5 属性 | 例 |
|---|---|---|
| `.min(n)` (string) | `minlength` | `z.string().min(8)` → `minlength="8"` |
| `.max(n)` (string) | `maxlength` | `z.string().max(128)` → `maxlength="128"` |
| `.min(n)` (number) | `min` | `z.number().min(0)` → `min="0"` |
| `.max(n)` (number) | `max` | `z.number().max(100)` → `max="100"` |
| `.email()` / `.url()` | `type` | `z.string().email()` → `type="email"` |
| `.optional()` でない | `required` | `z.string()` → `required` |

boundform の YAML を書く際は、**この表にあるもの**だけを `fields:` に定義してください。変換されない制約（`pattern` など）を YAML に書いても HTML には出力されないため、必ず mismatch になります。

```yaml
# ✓ conform が出力する属性だけを検査
fields:
  password:
    type: password
    required: true
    minlength: 8
    maxlength: 128

# ✗ pattern は conform が出力しないので mismatch になる
fields:
  password:
    type: password
    required: true
    pattern: "^(?=.*[A-Z]).*$"
```

### Why HTML attributes matter even with Zod

1. **Progressive enhancement** — forms validate before JS loads (critical for Server Actions)
2. **Instant UX** — browser-native validation fires immediately, no JS round-trip
3. **Accessibility** — screen readers announce `required` fields (WCAG compliance)
4. **Defense in depth** — HTML + Zod + Server Actions = 3-layer validation

### Recommended validation stack

| Layer | Tool | Role |
|-------|------|------|
| Schema definition | Zod | Single source of truth |
| HTML attributes | conform | Zod → HTML attributes |
| Client validation | react-hook-form + zodResolver | Rich client-side UX |
| HTML constraint check | **boundform** | CI check that HTML matches spec |
| Server validation | Zod (Server Actions) | Tamper prevention |

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
