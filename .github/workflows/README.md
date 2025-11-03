# GitHub Actions Workflows

## Release Workflow (`release.yml`)

This workflow automatically builds and releases macOS versions of jvlauncher.

### Features

- ✅ **Dual Architecture Support**: Builds for both Apple Silicon (M1/M2/M3) and Intel Macs
- ✅ **Aggressive Caching**: Optimized caching for Rust, Bun, and Tauri CLI to minimize build times
- ✅ **Automatic Releases**: Creates GitHub releases with DMG files attached
- ✅ **Draft Releases**: Releases are created as drafts for review before publishing
- ✅ **Auto-Updater Support**: Generates updater JSON for Tauri's built-in updater
- ✅ **Artifact Upload**: Uploads build artifacts for debugging and testing

### How to Trigger

The workflow is configured for **manual trigger only**.

#### Manual Trigger
1. Go to the **Actions** tab in your GitHub repository
2. Select **Release macOS** workflow
3. Click **Run workflow**
4. Choose the branch (usually `master`) and click **Run workflow**

### Prerequisites

#### 1. Enable Workflow Permissions
The workflow needs write permissions to create releases:

1. Go to **Settings** → **Actions** → **General**
2. Scroll to **Workflow permissions**
3. Select **Read and write permissions**
4. Click **Save**

#### 2. Ensure Version is Set
The release version is read from `src-tauri/tauri.conf.json`:

```json
{
  "version": "0.1.0"
}
```

Update this version before creating a release.

### Workflow Steps

1. **Checkout**: Clones the repository
2. **Setup Bun**: Installs Bun package manager
3. **Cache Bun Dependencies**: Caches `node_modules` and Bun cache
4. **Install Rust**: Installs Rust toolchain with target architecture
5. **Cache Rust**: Caches Rust build artifacts (biggest time saver!)
6. **Install Dependencies**: Runs `bun install --frozen-lockfile`
7. **Cache Tauri CLI**: Caches the Tauri CLI binary
8. **Install Tauri CLI**: Installs if not cached
9. **Build**: Builds the Tauri app for the target architecture
10. **Upload Artifacts**: Uploads DMG and .app files to GitHub

### Build Outputs

The workflow produces the following artifacts:

- `jvlauncher_<version>_aarch64.dmg` - Apple Silicon installer
- `jvlauncher_<version>_x64.dmg` - Intel installer
- `latest.json` - Updater manifest (if updater is configured)

### Optimization Features

#### Caching Strategy
The workflow implements multi-layer caching:

1. **Bun Dependencies Cache**
   - Caches `node_modules` and Bun's install cache
   - Key: Based on `bun.lockb` hash
   - Saves ~30-60 seconds per build

2. **Rust Build Cache**
   - Caches compiled dependencies and build artifacts
   - Separate cache per target architecture
   - Shared across branches for maximum reuse
   - Saves ~5-10 minutes per build (biggest impact!)

3. **Tauri CLI Cache**
   - Caches the installed Tauri CLI binary
   - Key: Based on `Cargo.lock` hash
   - Saves ~2-3 minutes per build

#### Concurrency Control
- Cancels in-progress builds when new commits are pushed
- Prevents wasted build minutes on outdated code

#### Build Optimizations
- `CARGO_INCREMENTAL=1`: Enables incremental compilation
- `CARGO_PROFILE_RELEASE_LTO="thin"`: Uses thin LTO for faster builds
- `--frozen-lockfile`: Ensures consistent dependency versions
- Separate matrix jobs run in parallel

### Expected Build Times

| Scenario | First Build | Cached Build |
|----------|-------------|--------------|
| Cold start (no cache) | ~15-20 min | N/A |
| Warm cache (dependencies cached) | N/A | ~5-8 min |
| Hot cache (no code changes) | N/A | ~3-5 min |

*Times are per architecture. Both architectures build in parallel.*

### Troubleshooting

#### Build Fails with "Resource not accessible by integration"
- **Cause**: Workflow doesn't have write permissions
- **Fix**: Enable "Read and write permissions" in repository settings (see Prerequisites)

#### Cache Not Working
- **Check**: Ensure `bun.lockb` and `Cargo.lock` are committed to the repository
- **Clear Cache**: Go to Actions → Caches → Delete old caches

#### Release Already Exists
- **Cause**: A release with the same version tag already exists
- **Fix**: 
  - Delete the existing release/tag, or
  - Update the version in `tauri.conf.json`

#### Build Takes Too Long
- **Check**: Look at the workflow run logs to see which step is slow
- **Common Issues**:
  - First build is always slow (no cache)
  - Rust cache might not be working (check cache hit logs)
  - Network issues downloading dependencies

### Customization

#### Change Release Trigger
Edit the `on:` section in `release.yml`:

```yaml
# Trigger on version tags
on:
  push:
    tags:
      - 'v*'
```

#### Add Code Signing
Add these steps before the build step:

```yaml
- name: Import Code Signing Certificate
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  run: |
    # Import certificate
    echo $APPLE_CERTIFICATE | base64 --decode > certificate.p12
    security create-keychain -p actions temp.keychain
    security default-keychain -s temp.keychain
    security unlock-keychain -p actions temp.keychain
    security import certificate.p12 -k temp.keychain -P $APPLE_CERTIFICATE_PASSWORD -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k actions temp.keychain
```

#### Publish Release Automatically
Change `releaseDraft: true` to `releaseDraft: false` in the workflow.

#### Add Windows/Linux Builds
Add additional matrix entries:

```yaml
matrix:
  settings:
    - platform: 'macos-latest'
      target: 'aarch64-apple-darwin'
      arch: 'aarch64'
    - platform: 'ubuntu-22.04'
      target: ''
      arch: 'x64'
    - platform: 'windows-latest'
      target: ''
      arch: 'x64'
```

### Cost Considerations

GitHub Actions pricing for macOS runners:

- **Public repositories**: 2,000 free minutes/month
- **Private repositories**: macOS minutes cost 10x Linux minutes

**Estimated costs per release** (both architectures):
- First build: ~30-40 minutes = 300-400 Linux-equivalent minutes
- Cached build: ~10-16 minutes = 100-160 Linux-equivalent minutes

### Best Practices

1. **Test locally first**: Run `./build.sh` before pushing to ensure the build works
2. **Use draft releases**: Review the release before publishing
3. **Version bumping**: Update version in `tauri.conf.json` before each release
4. **Changelog**: Update `CHANGELOG.md` with release notes
5. **Tag releases**: Use semantic versioning (e.g., `v1.0.0`, `v1.0.1`)

### Additional Resources

- [Tauri GitHub Actions Guide](https://v2.tauri.app/distribute/pipelines/github/)
- [tauri-action Documentation](https://github.com/tauri-apps/tauri-action)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Bun Documentation](https://bun.sh/docs)

