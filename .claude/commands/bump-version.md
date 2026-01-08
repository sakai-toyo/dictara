---
description: Bump version
argument-hint: [should-release]
---

Arguments:

- {should-release} - should release. Allowed values: yes | no. Default: yes.

# Validation

- The `should-release` should be semantically one of allowed values, otherwise stop and let user know that it is invalid command.
- Validate that no uncommitted changes are present.

# Prerequisites

Checkout the `main` branch and pull latest changes.

# Step 1: Bump version in `Cargo.toml`

Update the patch version in `src-tauri/Cargo.toml` file.

# Step 2: Run check

Run `npm run be:check` to update the lock file.

# Step 3: Commit and push changes

Commit and push changes using `/git commit` with message: `chore: bump version to x.x.x` (replace x.x.x with actual version).

# Step 4: Trigger release (conditional)

If `should-release` is "yes" (or similar affirmative value):
1. Trigger release pipeline: `gh workflow run release.yml --ref main`
2. Wait 2 seconds for the run to register: `sleep 2`
3. Get the specific run URL and show it to the user: `gh run list --workflow=release.yml --limit=1 --json url --jq '.[0].url'`

If `should-release` is "no" (or similar negative value), skip this step and inform user that version was bumped but release was not triggered.
