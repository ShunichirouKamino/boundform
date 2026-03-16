# ADR-0002: Form Identifier by Selector and Index

## Status

Accepted

## Date

2026-03-16

## Context

[ADR-0001](0001-form-identifier-resolution.md) documents the current form identifier resolution, which relies on `name → id → action → "unnamed"`. This breaks down in real-world scenarios:

1. **Next.js / React Server Actions** inject `action="javascript:throw new Error(...)"` into `<form>` tags automatically. When the form has no `name` or `id`, this framework-generated string becomes the identifier.

2. **Multiple unnamed forms** on the same page all resolve to `"unnamed"`, making them indistinguishable.

3. **Component libraries** (shadcn/ui, Radix, etc.) often render `<form>` without `name` or `id` attributes, since form identity is managed by React state rather than HTML semantics.

These are not edge cases — they are the default behavior of the most popular frameworks.

### Verified example

A real Next.js 15 project renders:

```html
<form class="space-y-4"
      action="javascript:throw new Error('React form unexpectedly submitted.')">
  <input type="email" name="email" required />
  <input type="password" name="password" required />
</form>
```

The current YAML config must use the full `action` string as the identifier, which is fragile and unreadable.

## Decision (Proposed)

Add two new optional fields to `FormConfig`: `selector` and `index`. The `id` field remains supported for backward compatibility.

### Resolution priority

```
selector  →  match by CSS selector (first match)
index     →  match by 0-based position in the page
id        →  match by resolved identifier (name → id → action, current behavior)
```

Only one should be specified. If multiple are present, use the priority above.

### YAML config examples

```yaml
pages:
  - url: "http://localhost:3000/login"
    forms:
      # Match by position — simplest option for single-form pages
      - index: 0
        fields:
          email:
            type: email
            required: true

  - url: "http://localhost:3000/register"
    forms:
      # Match by CSS selector — useful when multiple forms exist
      - selector: "form.space-y-4"
        fields:
          username:
            type: text
            required: true

  - url: "http://localhost:3000/settings"
    forms:
      # Match by id — backward compatible, works when form has name/id
      - id: "profile-form"
        fields:
          username:
            type: text
            maxlength: 50
```

### Config struct changes

```rust
#[derive(Debug, Deserialize)]
pub struct FormConfig {
    /// Match by resolved identifier (name/id/action). Backward compatible.
    pub id: Option<String>,
    /// Match by CSS selector (first matching <form>).
    pub selector: Option<String>,
    /// Match by 0-based index among all <form> elements on the page.
    pub index: Option<usize>,
    /// Expected fields.
    pub fields: IndexMap<String, FieldExpectation>,
}
```

### Matching logic changes

```rust
fn find_matching_form<'a>(
    config: &FormConfig,
    forms: &'a [FormInfo],
    document: &Html,  // needed for selector matching
) -> Option<&'a FormInfo> {
    if let Some(ref selector) = config.selector {
        // Use scraper to find the <form> matching the CSS selector,
        // then match it against parsed forms by position
    } else if let Some(index) = config.index {
        forms.get(index)
    } else if let Some(ref id) = config.id {
        forms.iter().find(|f| f.identifier == *id)
    } else {
        None
    }
}
```

### Validation rules

- At least one of `id`, `selector`, or `index` must be specified → error otherwise.
- If `selector` matches zero forms → report as "form not found".
- If `index` is out of bounds → report as "form not found".

## Consequences

### Positive

- **Framework-agnostic.** Works regardless of what Next.js, SvelteKit, or any framework does to the `<form>` tag.
- **Readable YAML.** `index: 0` is far more intuitive than pasting a JavaScript error string.
- **Backward compatible.** Existing configs using `id` continue to work.
- **Handles multiple forms.** Index-based matching cleanly distinguishes unnamed forms.

### Negative

- **`selector` requires passing the raw HTML to the comparator**, not just parsed `FormInfo`. This changes the function signature.
- **`index` is fragile** — adding/removing a form above the target shifts the index. Should be documented as a trade-off.
- **Increased config complexity.** Three ways to identify a form may confuse new users. Mitigate with clear docs and examples.

### Recommended user guidance

| Scenario | Recommended method |
|---|---|
| Single form on page | `index: 0` |
| Form has `name` or `id` | `id: "form-name"` |
| Multiple unnamed forms | `selector` or `index` |
| Framework-generated attributes | `index` or `selector` |

## Implementation notes

- `FormInfo` should store its positional index (0-based) during parsing so that index matching is trivial.
- For `selector` matching, the comparator will need access to the raw `Html` document. Consider passing it alongside `Vec<FormInfo>`.
- Consider adding a `--discover` flag to the CLI that prints all forms with their resolved identifiers, selectors, and indices to help users write configs.
