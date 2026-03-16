# CLAUDE.md — boundform

## Project Overview

`boundform` is a lightweight Rust CLI tool that validates HTML form constraints against a YAML specification — without needing a browser. It fetches SSR-rendered pages via HTTP GET, extracts form constraint attributes (required, min, max, minlength, maxlength, pattern, type, step), and compares them against expected values defined in YAML.

## Architecture

```
CLI (clap) → Source (reqwest GET / file read) → Parser (scraper) → Analyzer → Reporter
```

### Core Modules

- `src/main.rs` — CLI entrypoint using `clap`
- `src/source.rs` — Fetch HTML from URL or read from file (supports `--cookie` / `--header`)
- `src/parser.rs` — Parse HTML, extract `<form>` and `<input>` elements and their constraint attributes
- `src/model.rs` — Data structures: `FormField`, `FormInfo`, `InputType`
- `src/config.rs` — YAML configuration parsing
- `src/comparator.rs` — Compare expected constraints (YAML) against actual HTML
- `src/reporter.rs` — Output results (terminal, JSON)
- `src/error.rs` — Custom error types using `thiserror`

### Key Data Model

```rust
struct FormField {
    name: String,
    input_type: InputType,      // text, number, email, url, password, tel, etc.
    required: bool,
    min: Option<f64>,
    max: Option<f64>,
    minlength: Option<usize>,
    maxlength: Option<usize>,
    pattern: Option<String>,
    step: Option<f64>,
}
```

## Tech Stack

- **Language**: Rust (latest stable)
- **HTTP client**: `reqwest` (with `blocking` or `tokio` async)
- **HTML parser**: `scraper` (CSS selector based)
- **CLI framework**: `clap` (derive API)
- **Output**: colored terminal output via `colored`, optional JSON via `serde_json`
- **Testing**: standard `cargo test` + integration tests with HTML fixtures

## Development Guidelines

- Write idiomatic Rust: use `Result<T, E>` for error handling, avoid `.unwrap()` in library code
- All public functions must have doc comments
- Keep modules focused — single responsibility
- Use `thiserror` for custom error types
- Integration tests go in `tests/` with HTML fixture files in `tests/fixtures/`
- Commit messages in English, conventional commits style (feat:, fix:, docs:, refactor:, test:)

## Commands

```bash
cargo build          # Build
cargo test           # Run all tests
cargo clippy         # Lint
cargo fmt            # Format
cargo run -- --config boundform.yml  # Run the tool
```

## Target Audience

- Frontend/fullstack developers who want lightweight form validation checks
- CI pipelines that need fast HTML constraint verification
- Teams using SSR frameworks (Next.js, SvelteKit, Nuxt, Rails, Laravel, Django)
