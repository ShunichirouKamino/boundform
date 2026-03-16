# ADR-0004: Release and Distribution Strategy

## Status

Accepted

## Date

2026-03-16

## Context

boundform is a Rust CLI tool that needs to be distributed to end users. The project needs:

1. A way to build and publish releases with pre-built binaries
2. A familiar installation experience for web developers (the primary audience)
3. A way to distribute Claude Code skills alongside the binary
4. Support for multiple platforms (Windows, Linux, macOS)

### Target audience considerations

boundform's primary users are frontend/fullstack developers working with SSR frameworks (Next.js, SvelteKit, Nuxt, Rails, Laravel, Django). These users are comfortable with `npm`/`npx` but may not have Rust or `cargo` installed. The installation experience should not require the Rust toolchain.

## Decision

### Distribution via npm + GitHub Releases

The binary is distributed through two channels:

1. **GitHub Releases**: Pre-built binaries for each platform, attached to tagged releases
2. **npm package**: A thin wrapper that downloads the correct binary from GitHub Releases on first run

Users install and run via:

```bash
# No install needed — downloads binary on first run
npx boundform --config boundform.yml

# Or install globally
npm install -g boundform
```

### npm package structure

```
npm/
├── package.json
├── bin/
│   └── boundform.js          # Entry point — routes to binary or init
├── scripts/
│   ├── download-binary.js    # Downloads platform-specific binary from GitHub Releases
│   └── init.js               # Sets up project with skills and YAML template
├── skills/
│   ├── boundform-guide/      # Usage guide skill
│   └── release/              # Release automation skill
└── templates/
    └── boundform.yml         # Starter config template
```

### Binary download strategy

Rather than publishing platform-specific npm packages (e.g., `@boundform/win32-x64`), the npm package downloads the binary from GitHub Releases at runtime. This was chosen because:

| Approach | Pros | Cons |
|---|---|---|
| **Platform-specific npm packages** (biome, turbo style) | Fast install, works offline | Complex publish pipeline, 4+ packages to maintain |
| **Download from GitHub Releases** (chosen) | Single npm package, simple CI | First run downloads binary, requires internet |

The download-from-releases approach is simpler and sufficient for the current scale. The binary is cached at `~/.cache/boundform/{version}/` so the download only happens once per version.

### Execution flow: what happens when you run `npx boundform`

```
User runs: npx boundform --config boundform.yml
  │
  ├─ npm downloads the boundform npm package (7KB, no binary inside)
  │
  ├─ bin/boundform.js starts
  │    ├─ If argv[2] === "init" → run scripts/init.js (copy skills & template)
  │    └─ Otherwise → call scripts/download-binary.js
  │
  ├─ scripts/download-binary.js
  │    ├─ Detect platform (os.platform() + os.arch())
  │    │   e.g. win32-x64 → target: x86_64-pc-windows-gnu
  │    │
  │    ├─ Check cache: ~/.cache/boundform/{version}/boundform.exe
  │    │   ├─ Cache hit → use cached binary, skip download
  │    │   └─ Cache miss → download from GitHub Releases:
  │    │        GET https://github.com/ShunichirouKamino/boundform/
  │    │            releases/download/v{version}/boundform-{target}.exe
  │    │        Save to ~/.cache/boundform/{version}/boundform.exe
  │    │
  │    └─ Return path to binary
  │
  └─ bin/boundform.js spawns the binary as a child process
       with all CLI arguments forwarded
       └─ boundform.exe --config boundform.yml
           (native Rust binary runs, validates forms, outputs results)
```

Key points:
- The npm package contains **only JavaScript** (7KB). No Rust binary is shipped via npm.
- The binary is fetched from **GitHub Releases** — the same artifacts created by the CI pipeline.
- After the first run, the binary is **cached locally** at `~/.cache/boundform/{version}/`. Subsequent runs start instantly.
- Platform detection maps Node.js `os.platform()`/`os.arch()` to Rust target triples (e.g., `darwin-arm64` → `aarch64-apple-darwin`).

### Platform support

Binaries are built for four targets:

| Platform | Target | Asset name |
|---|---|---|
| Windows x64 | `x86_64-pc-windows-gnu` | `boundform-x86_64-pc-windows-gnu.exe` |
| Linux x64 | `x86_64-unknown-linux-gnu` | `boundform-x86_64-unknown-linux-gnu` |
| macOS x64 (Intel) | `x86_64-apple-darwin` | `boundform-x86_64-apple-darwin` |
| macOS ARM64 (Apple Silicon) | `aarch64-apple-darwin` | `boundform-aarch64-apple-darwin` |

### Skill distribution via `npx boundform init`

Claude Code skills are bundled in the npm package and copied to the user's project via an init command:

```bash
npx boundform init
```

This creates:
- `.claude/skills/boundform-guide/` — usage guide, YAML config help, troubleshooting
- `.claude/skills/release/` — release automation for maintainers
- `boundform.yml` — starter config template
- `.gitignore` update — excludes skill workspace directories

### CI/CD pipeline

Releases are automated via GitHub Actions (`.github/workflows/release.yml`):

```
Push tag v* → Build (4 platforms in parallel) → Create GitHub Release → Publish to npm
```

1. **Build job** (matrix): Compiles for all four targets in parallel
2. **Release job**: Collects artifacts, generates conventional-commits release notes, creates GitHub Release with all binaries attached
3. **Publish job**: Syncs version from git tag, publishes npm package

The pipeline requires one secret: `NPM_TOKEN` (npm access token with Automation type).

### Release notes format

Release notes are auto-generated from git log using conventional commits:

```markdown
## What's Changed

### Features
- feat: add authentication support via --cookie flag

### Bug Fixes
- fix: handle minLength case sensitivity in parser

### Other Changes
- docs: update README with SPA workaround

### Assets
- boundform-x86_64-pc-windows-gnu.exe — Windows x64
- boundform-x86_64-unknown-linux-gnu — Linux x64
- boundform-x86_64-apple-darwin — macOS x64 (Intel)
- boundform-aarch64-apple-darwin — macOS ARM64 (Apple Silicon)

**Full Changelog**: https://github.com/.../compare/v0.1.0...v0.1.1
```

## Consequences

### Positive

- **Zero Rust dependency for users.** Web developers can use `npx` without installing Rust.
- **Skills ship with the tool.** `npx boundform init` gives users immediate access to Claude Code integration.
- **Single npm package.** No need to maintain per-platform packages.
- **Fully automated releases.** Push a tag, get binaries + npm publish.
- **Conventional commits drive release notes.** No manual changelog maintenance.

### Negative

- **First run requires internet.** The binary download adds latency on first execution. Mitigated by caching.
- **GitHub Releases as CDN.** If GitHub is down, `npx boundform` fails. Acceptable for the current scale.
- **npm version must be synced with Cargo.toml.** The CI pipeline handles this automatically via the git tag, but manual releases need to update both files.

### Future considerations

- **Platform-specific npm packages**: If download latency becomes a problem, switch to the biome/turbo pattern with `optionalDependencies`.
- **cargo install**: Once published to crates.io, Rust users can `cargo install boundform` directly.
- **Homebrew formula**: For macOS users who prefer `brew install boundform`.
