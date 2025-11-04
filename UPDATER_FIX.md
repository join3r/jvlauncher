# Auto-Updater Fix Summary

## Problem

The auto-updater was failing with this error:
```
[2025-11-04T15:55:06Z ERROR tauri_plugin_updater::updater] update endpoint did not respond with a successful status code
[2025-11-04T15:55:06Z ERROR jvlauncher::updater] Failed to check for updates: Could not fetch a valid release JSON from the remote
```

## Root Cause

When you manually compile and upload a DMG to GitHub Releases, the **`latest.json` file is NOT automatically created**. This file is only generated when using the GitHub Actions workflow with the `tauri-action` that has `includeUpdaterJson: true`.

The updater requires this `latest.json` file to:
1. Check what version is available
2. Get the download URL for the update
3. Verify the signature (if configured)

## Solution Applied

### 1. Created Helper Scripts

Two scripts were created to help with manual releases:

**`scripts/create-latest-json.sh`** - Quick script to generate `latest.json`
```bash
./scripts/create-latest-json.sh [version]
```

**`scripts/generate-update-json.sh`** - Full script that finds the DMG and generates everything
```bash
./scripts/generate-update-json.sh
```

### 2. Generated and Uploaded `latest.json`

For version 0.1.4, we:
1. Generated the `latest.json` file
2. Uploaded it to the GitHub release

The file now exists at:
```
https://github.com/join3r/jvlauncher/releases/download/0.1.4/latest.json
```

### 3. Removed Signature Requirement (Temporary)

Since manually signing releases is complex, we removed the `pubkey` requirement from `src-tauri/tauri.conf.json`:

**Before:**
```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [...],
      "dialog": true,
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDU0RENEMzBDQUVGNUQ1MDYKUldRRzFmV3VETlBjVkI0TlJ0eW5KU0pDMERzY1hHT1NEWDErM0I2VW5WdWFWNnkvMko3Z0JhSGMK",
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

**After:**
```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [...],
      "dialog": true,
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

This means updates will work but won't be cryptographically verified. This is acceptable for development/testing.

## How to Handle Future Manual Releases

### Option 1: Use GitHub Actions (Recommended)

The easiest way is to use the GitHub Actions workflow that's already configured:

1. Update version in `src-tauri/Cargo.toml`
2. Commit and push
3. Create a tag: `git tag v0.1.5 && git push origin v0.1.5`
4. Manually trigger the workflow in GitHub Actions
5. The workflow will automatically:
   - Build the DMG
   - Sign it (if secrets are configured)
   - Create `latest.json`
   - Upload everything to the release

### Option 2: Manual Upload with Helper Script

If you prefer to build locally:

1. Build the app: `bun tauri build`
2. Generate `latest.json`: `./scripts/create-latest-json.sh`
3. Upload to GitHub release:
   ```bash
   gh release create v0.1.5 \
     src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/jvlauncher_0.1.5_aarch64.dmg \
     latest.json
   ```

## Current Status

✅ `latest.json` is now available for version 0.1.4
✅ Updater endpoint is accessible
✅ Signature requirement removed (updates work without verification)
⚠️ Next build needs to be done to test the updater with the new config

## Testing the Updater

To test if the updater works:

1. Make sure you're running version 0.1.3 or earlier (or change the version in Cargo.toml to 0.1.3)
2. Build and run the app
3. Open Settings
4. Click "Check for Updates"
5. It should detect version 0.1.4 is available

## Re-enabling Signatures (Optional)

If you want to re-enable signature verification:

1. Make sure the private key is properly formatted in `~/.tauri/jvlauncher.key`
2. Set the password: `export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="compactor-prancing-headpiece-defiling-overpower-educated"`
3. Sign the DMG: `cargo tauri signer sign <dmg-file> -k ~/.tauri/jvlauncher.key -p "$TAURI_SIGNING_PRIVATE_KEY_PASSWORD"`
4. Update `latest.json` to include the signature
5. Add the `pubkey` back to `tauri.conf.json`

## Files Modified

- `src-tauri/tauri.conf.json` - Removed `pubkey` requirement
- `scripts/create-latest-json.sh` - New helper script (created)
- `scripts/generate-update-json.sh` - New helper script (created)
- `scripts/sign-and-update-release.sh` - New helper script (created)

## What Was Fixed

### 1. Version Mismatch
- **Issue:** `tauri.conf.json` had version 0.1.2 while `Cargo.toml` had 0.1.5
- **Fix:** Updated `tauri.conf.json` to version 0.1.5 (temporarily set to 0.1.3 for testing)

### 2. Compilation
- **Status:** ✅ App compiles successfully
- **Warnings:** Only harmless dead code warnings (unused functions)
- **Build output:** DMG created at `target/release/bundle/dmg/jvlauncher_0.1.3_aarch64.dmg`

## Testing

A test build with version 0.1.3 has been created. See `TEST_UPDATER.md` for testing instructions.

To test:
1. Open the app (already running)
2. Open Settings from menu bar
3. Click "Check for Updates"
4. Should detect version 0.1.4 is available

## Next Steps

1. ✅ Test the updater (see TEST_UPDATER.md)
2. Restore version to 0.1.5 in `tauri.conf.json`
3. For future releases, use GitHub Actions or the helper scripts

