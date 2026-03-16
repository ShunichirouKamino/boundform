---
name: release
description: Create a GitHub release for boundform with a built binary. Use this skill whenever the user says /release, wants to publish a new version, create a release, tag a version, or ship boundform. Also trigger when the user mentions bumping the version, cutting a release, or publishing the binary.
---

# Release boundform

This skill automates creating a GitHub release for boundform from the local machine (Devcontainer). It builds the binary, creates a git tag, generates release notes, and publishes a GitHub release with the binary attached.

## Prerequisites

Before running, verify these tools are available:
- `cargo` (Rust toolchain)
- `git` (with push access to origin)
- `gh` (GitHub CLI, authenticated)

If any are missing, inform the user and suggest how to install them.

## Release workflow

### Step 1: Determine the version

Check if the user passed a version argument (e.g., `/release 0.2.0`).

- **If a version is provided**: use it as-is
- **If no version is provided**: read from `Cargo.toml`

```bash
# Extract version from Cargo.toml
grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/'
```

The tag format is `v{version}` (e.g., `v0.1.0`).

### Step 2: Pre-flight checks

Before proceeding, verify:

1. Working directory is clean (`git status --porcelain` is empty). If not, warn the user and ask whether to proceed.
2. The tag does not already exist (`git tag -l v{version}`). If it does, tell the user and abort.
3. Current branch is `main`. If not, warn the user.

### Step 3: Build the binary

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

Verify the binary exists at:
```
target/x86_64-pc-windows-gnu/release/boundform.exe
```

If the build fails, show the error output and stop.

### Step 4: Generate release notes

Collect commits since the last tag (or all commits if this is the first release):

```bash
# Get the previous tag
PREV_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -z "$PREV_TAG" ]; then
  # First release — all commits
  COMMITS=$(git log --oneline --pretty=format:"- %s" HEAD)
else
  COMMITS=$(git log --oneline --pretty=format:"- %s" ${PREV_TAG}..HEAD)
fi
```

Organize the commits into conventional-commits categories:

```markdown
## What's Changed

### Features
- feat: description here

### Bug Fixes
- fix: description here

### Other Changes
- docs/refactor/chore/test: description here

**Full Changelog**: https://github.com/ShunichirouKamino/boundform/compare/{prev_tag}...v{version}
```

If there is no previous tag, omit the "Full Changelog" comparison link and instead use:
```markdown
**Full Changelog**: https://github.com/ShunichirouKamino/boundform/commits/v{version}
```

Show the generated release notes to the user and ask for confirmation before proceeding.

### Step 5: Create tag and push

```bash
git tag -a v{version} -m "Release v{version}"
git push origin main
git push origin v{version}
```

### Step 6: Create GitHub release

```bash
gh release create v{version} \
  target/x86_64-pc-windows-gnu/release/boundform.exe \
  --title "v{version}" \
  --notes "{release_notes}"
```

Use a heredoc for the notes to preserve formatting:
```bash
gh release create v{version} \
  target/x86_64-pc-windows-gnu/release/boundform.exe \
  --title "v{version}" \
  --notes "$(cat <<'EOF'
{release_notes}
EOF
)"
```

### Step 7: Report success

After the release is created, show:
- The release URL (returned by `gh release create`)
- The version that was released
- A reminder about any next steps

## Version bumping

If the user wants to bump the version before releasing, update `Cargo.toml`:

```bash
# Example: bump from 0.1.0 to 0.2.0
sed -i 's/^version = ".*"/version = "0.2.0"/' Cargo.toml
```

Then commit the version bump before tagging:
```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
```

## Future: npm publishing

npm publishing is planned but not yet implemented. When ready, the release workflow should also:
1. Update `package.json` version
2. Run `npm publish` to publish to npm registry
3. Users will then be able to install via `npx boundform`

This requires creating an npm package wrapper that downloads the correct platform binary. See the `npm/` directory (to be created) for the npm package structure.

## CI note

A GitHub Actions workflow exists at `.github/workflows/release.yml` for future CI-based releases. Currently releases are done manually from the Devcontainer. When CI is available again, switch to triggering releases by pushing a tag.
