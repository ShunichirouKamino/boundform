# Design Decisions & Background

This document captures the design discussions and decisions made during the inception of `boundform`.

## Motivation

Testing form input boundary values is a common need, but existing tools don't hit the sweet spot:

- **Playwright / Cypress**: Full browser automation — too heavy for just validating form constraints.
- **Unit testing validation logic**: Doesn't catch drift between HTML attributes and code-level validation.

There's a gap between these two extremes:

```
[Heavy]  Playwright E2E         ← full browser, slow, complex setup
[Gap]    ← boundform fills this
[Light]  Unit tests on logic    ← no HTML awareness
```

## Core Insight: SSR Makes This Possible

Modern frameworks (Next.js App Router, SvelteKit, Nuxt, Rails, Laravel, Django) render HTML on the server by default. A simple HTTP GET returns fully-rendered HTML including `<form>` and `<input>` elements with all their constraint attributes.

### Verified Behavior

**Next.js 15+ (App Router) with `'use client'` components:**
- Even components marked with `'use client'` are SSR-rendered on the first request (server-side pre-rendering before hydration).
- A plain `curl` to `http://localhost:3000/register` returns HTML containing:
  ```html
  <input type="text" required="" name="username" />
  <input type="email" required="" name="email" />
  <input type="password" required="" minLength="10" name="password" />
  ```
- This means `boundform` can extract constraints without a browser engine.

**SvelteKit:**
- SSR is the default (`ssr = true`). Only explicitly setting `export const ssr = false` disables it.
- Works with `boundform` out of the box.

### Compatibility Matrix

| Framework | SSR by Default | Compatible |
|---|---|---|
| Next.js (App Router) | Yes | Yes |
| SvelteKit | Yes | Yes |
| Nuxt | Yes | Yes |
| Rails / Laravel / Django | Yes | Yes |
| React (Vite, CRA) | No (CSR) | No |
| Vue (Vite) | No (CSR) | No |

## Architecture Decision: No Browser Engine

We deliberately chose NOT to embed a browser engine (headless Chrome, WebKit, etc.):

- **Pros**: Tiny binary, fast execution, minimal dependencies, easy CI integration
- **Cons**: Cannot analyze CSR-only (SPA) apps directly via URL
- **Trade-off**: This is acceptable because the majority of modern full-stack frameworks use SSR by default.

### SPA support via composition (not embedding)

Rather than adding a browser engine (which would make boundform another Playwright), SPA users can capture rendered HTML externally and pass it as a local file:

```
[Playwright / browser] → rendered.html → [boundform] → report
```

This follows the Unix philosophy: boundform is an HTML constraint validator, not an HTML renderer. The "how to get the HTML" is left to the user's toolchain.

Alternatives considered and rejected:

| Approach | Why rejected |
|---|---|
| Embed headless Chrome (via `chromiumoxide` / `headless_chrome`) | Adds ~50MB+ dependency, slow startup, contradicts "lightweight" positioning |
| Ship a separate `boundform-browser` binary | Maintenance burden of two binaries, confusing UX |
| `--headless` flag with optional browser feature | Feature-flagged complexity, build matrix issues |

The composition approach (Playwright CLI → file → boundform) achieves the same result with zero added complexity to boundform itself.

## Boundary Value Strategy

For each HTML constraint attribute, generate test values at and around the boundary:

| Constraint | Test Values |
|---|---|
| `required` | `""`, `" "` (whitespace) |
| `min=N` | `N-1`, `N`, `N+1` |
| `max=N` | `N-1`, `N`, `N+1` |
| `minlength=N` | string of length `N-1`, `N`, `N+1` |
| `maxlength=N` | string of length `N-1`, `N`, `N+1` |
| `pattern=regex` | matching string, non-matching string |
| `type=email` | valid email, invalid formats |
| `type=number` | numeric, non-numeric, boundaries |
| `step=N` | on-step, off-step |

## Field Name Resolution: `name` → `id` Fallback

HTML forms traditionally use the `name` attribute to identify fields in form submissions. However, React and other state-managed frameworks often omit `name` entirely — they use `useState` / `useForm` to track values in JavaScript and submit via `fetch()` or Server Actions, so the `name` attribute is unnecessary for their workflow.

In practice, these frameworks still set `id` on inputs (for `<label htmlFor>` association and accessibility). Boundform falls back to `id` when `name` is absent:

```
name="email"           →  field name: "email"
id="email" (no name)   →  field name: "email"
(neither)              →  field skipped
```

### Verified example

A real Next.js 15 game registration form renders:

```html
<input type="date" id="date" required />       <!-- no name -->
<input type="text" id="oka" pattern="[0-9]*" /> <!-- no name -->
<input type="text" id="tobi" pattern="[0-9]*" /><!-- no name -->
```

Without the `id` fallback, all three fields are invisible to boundform. With the fallback, they are correctly extracted and their constraints (`required`, `pattern`) can be validated.

## Scope Boundary: Fields Without `name` or `id`

Fields that have neither `name` nor `id` are intentionally skipped. We considered several approaches to handle them (nth-child indexing, CSS selectors, label proximity) but concluded that the cost/complexity outweighs the benefit:

- **nth-child indexing** is fragile — any HTML structure change shifts all indices.
- **CSS selectors** require users to understand the rendered DOM structure.
- **Label proximity** is unreliable across different markup patterns.

More importantly, fields without `name` or `id` are typically:
1. **Internal elements from component libraries** (Radix, shadcn/ui) marked `aria-hidden="true"` — not real form controls.
2. **State-managed inputs** where validation is handled entirely in JavaScript — HTML constraints are absent or incomplete.

In both cases, inspecting these elements provides little value.

### Recommendation

Rather than adding complexity to boundform, we recommend adding `name` or `id` to form inputs that should be testable. This is also good practice for accessibility (`<label for="...">`) and standard form submission.

### Verified example

A real Next.js 15 score input page renders 8 identical inputs with no identifiers:

```html
<input type="text" inputMode="numeric" pattern="[0-9]*" value="250" />
<input type="text" inputMode="numeric" pattern="[0-9]*" value="250" />
<input type="text" inputMode="numeric" pattern="[0-9]*" value="250" />
<!-- ... 5 more identical inputs -->
```

Even with nth-child support, a YAML config listing 8 entries by index would be painful to write and maintain. The better path is to add `name="score_east"`, `name="score_south"`, etc.

## Design Premise: HTML Attributes as the Validation Surface

boundform は **SSRで返されるHTMLに制約属性が正しく存在するか** を検証するツールである。Zodなどの JavaScript バリデーションライブラリの内容は検査対象外。

### なぜ HTML 属性が重要か

Zodだけで制約を定義した場合、HTML には `<input name="password">` のように属性が何もないタグが出力される。この状態では:

- JSが読み込まれる前のフォーム送信ではバリデーションが一切効かない
- スクリーンリーダーが `required` を認識できない（アクセシビリティ問題）
- Server Actions でJS無し送信された場合、サーバー側Zodチェックだけが最後の砦になる

HTML5 制約属性を付与することで、ブラウザネイティブのバリデーション（JS不要・即時フィードバック）が機能する。これはZodの代替ではなく、多層防御の1層目。

### 二重管理の解消: conform

Zodとhtml属性の両方を手書きするのは二重管理になる。この問題を解決するのが [conform](https://conform.guide/) 等のライブラリ:

```
Zodスキーマ（1箇所に定義）
  ├→ HTML属性       ← conform が自動生成
  ├→ クライアントJS  ← zodResolver
  └→ サーバー側      ← Server Actions
```

### boundform の位置づけ

```
Zodスキーマ → (conform等) → SSR HTML → (boundform) → 制約チェック
                                          ↑
                           ここだけが守備範囲
```

boundform は実装方法（手書き、conform、その他）には関与しない。最終的なHTML出力に対して「YAML仕様通りの制約属性があるか」だけを検証する。これにより:

- conform の設定漏れでHTML属性が付与されていないケースを検出
- デプロイ後にHTML属性が消えた制約ドリフトを検出
- フレームワーク非依存（Zod/conform/react-hook-form の選択に関係なく動作）

詳細は [ADR-0005](adr/0005-html-attributes-as-validation-surface.md) を参照。

## Observation: Attribute Drift

During testing with a real Next.js 15 project, we noticed that some attributes present in source code (e.g., `maxLength={50}` in React JSX) were NOT present in the rendered HTML. This suggests a potential feature: comparing source-level constraints against rendered HTML to detect drift.

## Tech Stack Rationale

- **Rust**: Single binary distribution, fast execution, strong type system for parsing
- **reqwest**: De facto HTTP client in Rust ecosystem
- **scraper**: CSS-selector-based HTML parsing, lightweight alternative to full DOM
- **clap**: Standard CLI framework with derive macros
- **No browser dependency**: Core differentiator from Playwright/Cypress
