# Release Checklist

Quick reference for creating releases with working auto-updates.

## Method 1: GitHub Actions (Recommended)

This is the easiest and most reliable method.

### Steps:

1. **Update version**
   ```bash
   # Edit src-tauri/Cargo.toml
   # Change version = "0.1.4" to version = "0.1.5"
   ```

2. **Commit changes**
   ```bash
   git add src-tauri/Cargo.toml
   git commit -m "Bump version to 0.1.5"
   git push
   ```

3. **Create and push tag**
   ```bash
   git tag v0.1.5
   git push origin v0.1.5
   ```

4. **Trigger workflow**
   - Go to: https://github.com/join3r/jvlauncher/actions
   - Select "Release" workflow
   - Click "Run workflow"
   - Select branch: `master`
   - Click "Run workflow"

5. **Wait for completion**
   - The workflow will automatically:
     - Build the DMG
     - Sign it (using GitHub secrets)
     - Create `latest.json`
     - Create a GitHub release
     - Upload all files

### ✅ What you get:
- Signed DMG
- `latest.json` with signature
- Automatic release creation
- Working auto-updates

---

## Method 2: Manual Build + Script

Use this if you want to build locally.

### Steps:

1. **Update version**
   ```bash
   # Edit src-tauri/Cargo.toml
   # Change version = "0.1.4" to version = "0.1.5"
   ```

2. **Build the app**
   ```bash
   bun tauri build
   ```

3. **Generate latest.json**
   ```bash
   ./scripts/create-latest-json.sh
   ```

4. **Create GitHub release**
   ```bash
   VERSION="0.1.5"
   gh release create "v${VERSION}" \
     "src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/jvlauncher_${VERSION}_aarch64.dmg" \
     latest.json \
     --title "jvlauncher v${VERSION}" \
     --notes "Release v${VERSION}"
   ```

### ⚠️ Limitations:
- No signature (updates work but aren't verified)
- Manual process
- Easy to forget steps

---

## Method 3: Manual Upload (Existing Release)

If you already uploaded a DMG manually and need to add `latest.json`.

### Steps:

1. **Generate latest.json**
   ```bash
   ./scripts/create-latest-json.sh 0.1.4
   ```

2. **Upload to existing release**
   ```bash
   gh release upload 0.1.4 latest.json --clobber
   ```

3. **Verify**
   ```bash
   curl -sL https://github.com/join3r/jvlauncher/releases/latest/download/latest.json
   ```

---

## Verification

After creating a release, always verify:

### 1. Check release assets
```bash
gh release view v0.1.5 --json assets --jq '.assets[].name'
```

Should show:
- `jvlauncher_0.1.5_aarch64.dmg`
- `latest.json`
- `jvlauncher_0.1.5_aarch64.dmg.sig` (if using GitHub Actions)

### 2. Check latest.json is accessible
```bash
curl -sL https://github.com/join3r/jvlauncher/releases/latest/download/latest.json
```

Should return valid JSON with version and download URL.

### 3. Test updater
1. Build app with older version (e.g., 0.1.4)
2. Run the app
3. Open Settings
4. Click "Check for Updates"
5. Should detect new version

---

## Common Issues

### Issue: "Could not fetch a valid release JSON"

**Cause:** `latest.json` is missing from the release

**Fix:**
```bash
./scripts/create-latest-json.sh [version]
gh release upload [version] latest.json --clobber
```

### Issue: "Signature verification failed"

**Cause:** `pubkey` is set in config but DMG isn't signed

**Fix Option 1:** Remove pubkey from `src-tauri/tauri.conf.json`

**Fix Option 2:** Use GitHub Actions which signs automatically

### Issue: Updater says "No updates available" but new version exists

**Cause:** Version in `latest.json` is not higher than current version

**Fix:** Make sure version in Cargo.toml is incremented before building

---

## Quick Commands

```bash
# Check current version
grep '^version = ' src-tauri/Cargo.toml

# List all releases
gh release list

# View specific release
gh release view v0.1.5

# Delete a release (if you need to redo it)
gh release delete v0.1.5 --yes

# Delete a tag
git tag -d v0.1.5
git push origin :refs/tags/v0.1.5
```

---

## Notes

- Always increment version number before building
- Use semantic versioning (MAJOR.MINOR.PATCH)
- Test the updater before announcing the release
- GitHub Actions is the most reliable method
- Keep the password for signing key safe: `compactor-prancing-headpiece-defiling-overpower-educated`

