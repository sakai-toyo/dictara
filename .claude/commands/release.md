---
description: Release
argument-hint: [mode]
---

Arguments:

- {mode} - release mode. Allowed values: release | release candidate | rc. Default: release candidate.

# Validation

- If the `mode` is not semantically one of the allowed values, stop and show the user:
  ```
  Invalid mode '{mode}'. Available options:
  - release (or r) - Create a stable release
  - release candidate (or rc) - Create a release candidate/pre-release
  ```
- Validate that no uncommitted changes are present.

# Prerequisites

Checkout the `main` branch and pull latest changes.

# Step 1: Bump version in `Cargo.toml`

Update the version in `src-tauri/Cargo.toml` file.

If `mode` is "release" (or similar affirmative value like "r"):
 - increment the patch version
 - drop the `-rc.X` suffix if present (e.g., 0.1.23-rc.2 → 0.1.23)

If `mode` is "release candidate", "rc" (or similar):
 - if current version is stable (e.g., 0.1.22): bump to next patch with -rc.1 (0.1.23-rc.1)
 - if current version is already RC (e.g., 0.1.23-rc.1): increment the rc number (0.1.23-rc.2)

# Step 2: Run check

Run `npm run be:check` to update the lock file.

# Step 3: Commit and push changes

Commit and push changes using `/git commit auto accept commit message` with message: `chore: bump version to x.x.x` (replace x.x.x with actual version).

# Step 4: Trigger release

Always trigger the release workflow, but with the appropriate prerelease flag:

If `mode` is "release" (stable release):
```bash
gh workflow run release.yml --ref main -f prerelease=false
```

If `mode` is "release candidate" or "rc" (RC/pre-release):
```bash
gh workflow run release.yml --ref main -f prerelease=true
```

After triggering:
1. Wait 2 seconds for the run to register: `sleep 2`
2. Get the run URL and show it to the user: `gh run list --workflow=release.yml --limit=1 --json url --jq '.[0].url'`
