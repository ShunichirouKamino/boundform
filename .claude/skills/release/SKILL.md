---
name: release
description: Create a GitHub release for boundform via workflow_dispatch. Use this skill whenever the user says /release, wants to publish a new version, create a release, tag a version, or ship boundform. Also trigger when the user mentions bumping the version, cutting a release, or publishing the binary.
---

# Release boundform

This skill triggers a GitHub Actions release workflow that automatically resolves the next version, builds multi-platform binaries, creates a GitHub Release, and publishes to npm.

## Usage

```
/release <bump> [--dry-run]
```

- `bump`: Version bump category — `micro` (default), `minor`, or `major`
- `--dry-run`: Resolve version only, skip actual release

### Examples

```
/release micro          # 0.1.4 → 0.1.5
/release minor          # 0.1.4 → 0.2.0
/release major          # 0.1.4 → 1.0.0
/release micro --dry-run  # Show what would happen
```

## Version Resolution

Versions are automatically calculated from existing git tags — no manual version editing needed.

| Category | Rule | Example (current: 0.1.4) |
|----------|------|--------------------------|
| `micro` | Z+1 | 0.1.5 |
| `minor` | Y+1, Z=0 | 0.2.0 |
| `major` | X+1, Y=0, Z=0 | 1.0.0 |

Use `micro` for normal releases. Use `minor` for new features or config changes. Use `major` for breaking changes.

## Workflow

### Step 1: Determine bump category

Parse the user's argument. Default to `micro` if not specified.

Check for `--dry-run` flag.

### Step 2: Trigger the workflow

```bash
gh workflow run release.yml \
  -f bump=<bump> \
  -f dry_run=<true|false> \
  --repo ShunichirouKamino/boundform \
  --ref main
```

### Step 3: Monitor the run

Wait a few seconds for the run to appear, then watch it:

```bash
# Get the latest run ID
RUN_ID=$(gh run list --workflow=release.yml --limit=1 --json databaseId -q '.[0].databaseId' --repo ShunichirouKamino/boundform)

# Watch progress
gh run watch $RUN_ID --repo ShunichirouKamino/boundform
```

### Step 4: Report results

On success, show:
- The resolved version and tag
- Link to the GitHub Release
- Confirmation that npm publish succeeded

On failure, show:
```bash
gh run view $RUN_ID --log-failed --repo ShunichirouKamino/boundform
```

## What the CI does

The workflow (`release.yml`) handles everything automatically:

1. **Resolve version** — reads git tags, calculates next version based on bump category
2. **Build** — compiles for Windows, Linux, macOS (x64 + ARM64) in parallel
3. **Version bump commit** — updates `Cargo.toml` and `npm/package.json`, commits, creates tag
4. **GitHub Release** — attaches all 4 binaries with auto-generated release notes
5. **npm publish** — publishes to npmjs.org via Trusted Publishing (OIDC)

No manual version editing, no manual tagging, no manual npm publish.
