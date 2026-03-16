# ADR-0001: Form Identifier Resolution

## Status

Accepted (current implementation)

## Date

2026-03-16

## Context

When `boundform` parses HTML and extracts `<form>` elements, each form needs an identifier so that the YAML config can reference it. HTML forms may or may not have attributes that serve as identifiers.

## Decision

Forms are identified by checking attributes in the following priority order:

1. `name` attribute
2. `id` attribute
3. `action` attribute
4. Falls back to `"unnamed"` if none are present

In the YAML config, the `id` field matches against this resolved identifier:

```yaml
forms:
  - id: "login"      # matches <form name="login"> or <form id="login">
    fields: ...
```

### Implementation

```rust
// src/parser.rs
let identifier = form_el.value()
    .attr("name")
    .or_else(|| form_el.value().attr("id"))
    .or_else(|| form_el.value().attr("action"))
    .unwrap_or("unnamed")
    .to_string();
```

### Matching in comparator

The comparator (`src/comparator.rs`) builds a `HashMap<&str, &FormInfo>` keyed by the resolved identifier, then does exact string matching against the YAML config's `id` field.

## Consequences

### Positive

- Simple and predictable resolution logic
- Works well for forms that follow HTML best practices (i.e., having `name` or `id`)
- No additional config syntax required

### Negative

- **Framework-generated `action` values become identifiers.** For example, Next.js (React) injects `action="javascript:throw new Error('React form unexpectedly submitted.')"` into `<form>` tags when using Server Actions. If the form has no `name` or `id`, this string becomes the identifier, making the YAML config awkward:

  ```yaml
  forms:
    - id: "javascript:throw new Error('React form unexpectedly submitted.')"
  ```

- **Multiple unnamed forms on the same page collide.** If two forms both lack `name`, `id`, and `action`, they both resolve to `"unnamed"`, making it impossible to distinguish them.

- **No support for CSS selectors or positional indexing.** Users cannot say "the second form on the page" or "the form with class `login-form`".

## Related

- See [ADR-0002](0002-form-identifier-by-selector-and-index.md) for a proposed improvement.
