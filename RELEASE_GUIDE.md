# jvlauncher Release Guide

This guide explains how to create a new release of jvlauncher with proper code signing and auto-update support.

## Quick Start

To create a new release, simply run:

```bash
./release.sh
```

The script will:
1. ✅ Check all prerequisites
2. ✅ Show current version and ask for confirmation
3. ✅ Build the application
4. ✅ Sign the DMG with your private key
5. ✅ Create a GitHub release
6. ✅ Upload the DMG and updater JSON
7. ✅ Verify everything is working

## Prerequisites

The release script checks for these automatically:

- **GitHub CLI (`gh`)** - For creating releases
  ```bash
  brew install gh
  gh auth login
  ```

- **Bun** - For building the Tauri app
  ```bash
  curl -fsSL https://bun.sh/install | bash
  ```

- **Rust/Cargo** - For Tauri
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **jq** - For JSON manipulation
  ```bash
  brew install jq
  ```

- **Signing Keys** - Located at `~/.tauri/jvlauncher.key` and `~/.tauri/jvlauncher.key.pub`
  - If missing, generate with: `cargo tauri signer generate -w ~/.tauri/jvlauncher.key`

## Before Releasing

### 1. Update Version

Edit `src-tauri/Cargo.toml`:

```toml
[package]
version = "0.1.6"  # Increment this
```

The script will automatically sync `src-tauri/tauri.conf.json` to match.

### 2. Test Your Changes

```bash
./dev.sh
```

Make sure everything works as expected.

### 3. Commit Your Changes

```bash
git add .
git commit -m "Prepare release v0.1.6"
git push
```

## Running the Release Script

### Interactive Mode

Simply run:

```bash
./release.sh
```

The script will:
1. Show you the current version
2. Ask for confirmation
3. Ask for your signing key password (if not set in environment)
4. Build, sign, and upload everything

### Environment Variables

You can set these to avoid prompts:

```bash
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="your-password-here"
./release.sh
```

## What the Script Does

### Step 1: Version Check

The script reads the version from `src-tauri/Cargo.toml` and ensures `src-tauri/tauri.conf.json` matches.

### Step 2: Build

Runs `bun tauri build` to create:
- DMG file at `src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/*.dmg`
- App bundle at `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/*.app`

### Step 3: Sign

Uses `cargo tauri signer sign` to create a cryptographic signature of the DMG file.

This signature is required for the auto-updater to verify the download is authentic.

### Step 4: Create GitHub Release

Creates a new release with tag `vX.Y.Z` and uploads:
- The DMG file
- A `latest.json` file for the auto-updater

### Step 5: Verify

Checks that `latest.json` is accessible at:
```
https://github.com/join3r/jvlauncher/releases/download/vX.Y.Z/latest.json
```

## The `latest.json` File

The auto-updater reads this file to check for new versions:

```json
{
  "version": "0.1.5",
  "date": "2025-11-04T22:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRRzFmV3VETlBjVkI0TlJ0eW5KU0pDMERzY1hHT1NEWDErM0I2VW5WdWFWNnkvMko3Z0JhSGMK...",
      "url": "https://github.com/join3r/jvlauncher/releases/download/v0.1.5/jvlauncher_0.1.5_aarch64.dmg"
    }
  }
}
```

## Troubleshooting

### "Missing signing key"

Generate a new key:

```bash
cargo tauri signer generate -w ~/.tauri/jvlauncher.key
```

**Important:** Save the password! You'll need it for every release.

### "GitHub CLI not authenticated"

Run:

```bash
gh auth login
```

### "Build failed"

Check the build output for errors. Common issues:
- Missing dependencies
- TypeScript errors in the frontend
- Rust compilation errors

Fix the errors and try again.

### "Release already exists"

The script will ask if you want to delete and recreate it. Choose yes to proceed.

### "Invalid encoding in minisign data"

This error occurs when:
1. The signature in `latest.json` is empty or malformed
2. The public key in `tauri.conf.json` doesn't match the signing key

**Solution:** Use the `release.sh` script which handles signing correctly.

## Testing the Auto-Updater

After releasing version `0.1.6`:

1. **Build an older version** (e.g., 0.1.5):
   ```bash
   # Edit Cargo.toml to set version = "0.1.5"
   bun tauri build
   ```

2. **Install the older version**

3. **Open the app and check for updates:**
   - Click the menu bar icon
   - Go to Settings
   - Click "Check for Updates"
   - Should show: "Update to version 0.1.6 is available"

4. **Download and install the update**

5. **Verify the app updated correctly**

## Manual Release (Not Recommended)

If you need to create a release manually:

1. Build: `bun tauri build`
2. Find DMG: `src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/*.dmg`
3. Sign: `cargo tauri signer sign <dmg-file> -k ~/.tauri/jvlauncher.key -p <password>`
4. Create release: `gh release create vX.Y.Z <dmg-file>`
5. Create `latest.json` with the signature from step 3
6. Upload: `gh release upload vX.Y.Z latest.json`

**But seriously, just use `./release.sh` instead!**

## Security Notes

### Private Key

Your private key (`~/.tauri/jvlauncher.key`) is **password-protected** and should:
- ✅ Be backed up securely
- ✅ Never be committed to git
- ✅ Never be shared publicly
- ✅ Be stored in a secure location

### Public Key

Your public key is embedded in `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDU0RENEMzBDQUVGNUQ1MDYKUldRRzFmV3VETlBjVkI0TlJ0eW5KU0pDMERzY1hHT1NEWDErM0I2VW5WdWFWNnkvMko3Z0JhSGMK"
    }
  }
}
```

This is safe to commit and distribute. It's used by the app to verify signatures.

### Signature Verification

When a user checks for updates:
1. App downloads `latest.json`
2. App downloads the DMG from the URL in `latest.json`
3. App verifies the DMG signature using the public key
4. If signature is valid, app installs the update
5. If signature is invalid, app rejects the update

This prevents malicious updates from being installed.

## CI/CD with GitHub Actions

The repository includes `.github/workflows/release.yml` for automated releases.

To use it:

1. Add secrets to your GitHub repository:
   - `TAURI_SIGNING_PRIVATE_KEY` - Your private key file content
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - Your key password

2. Trigger the workflow:
   - Go to Actions tab
   - Select "Release macOS"
   - Click "Run workflow"

The workflow will build, sign, and release automatically.

## Version Numbering

Follow semantic versioning:

- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backwards compatible
- **Patch** (0.0.1): Bug fixes, backwards compatible

Examples:
- `0.1.5` → `0.1.6` - Bug fix
- `0.1.5` → `0.2.0` - New feature
- `0.1.5` → `1.0.0` - Major release

## After Release

1. **Test the updater** with an older version
2. **Update version** in `Cargo.toml` for next release
3. **Commit the version bump:**
   ```bash
   git add src-tauri/Cargo.toml
   git commit -m "Bump version to 0.1.7-dev"
   git push
   ```

## Getting Help

If you encounter issues:

1. Check the script output for error messages
2. Verify all prerequisites are installed
3. Check GitHub release page for uploaded files
4. Test the `latest.json` URL in a browser

For more help, check:
- [Tauri Updater Documentation](https://tauri.app/v1/guides/distribution/updater)
- [GitHub CLI Documentation](https://cli.github.com/manual/)

