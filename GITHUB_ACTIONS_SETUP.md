# GitHub Actions Setup Guide

This guide will help you set up and use the GitHub Actions workflow for automated macOS releases.

## Quick Start

### 1. Enable Workflow Permissions

Before the workflow can create releases, you need to enable write permissions:

1. Go to your GitHub repository
2. Click **Settings** → **Actions** → **General**
3. Scroll down to **Workflow permissions**
4. Select **"Read and write permissions"**
5. Click **Save**

### 2. Commit and Push the Workflow

The workflow file has been created at `.github/workflows/release.yml`. Commit and push it:

```bash
git add .github/workflows/release.yml
git add .github/workflows/README.md
git add GITHUB_ACTIONS_SETUP.md
git commit -m "Add GitHub Actions workflow for macOS releases"
git push origin master
```

### 3. Create Your First Release

#### Manual Trigger (Only Way to Run)

1. Make sure your version is updated in `src-tauri/tauri.conf.json`
2. Commit and push your changes to the `master` branch
3. Go to the **Actions** tab in your GitHub repository
4. Click on **"Release macOS"** in the left sidebar
5. Click **"Run workflow"** button
6. Select the branch (usually `master`)
7. Click **"Run workflow"**

The workflow will:
- Build for both Apple Silicon and Intel Macs
- Create a draft GitHub release
- Upload the DMG files as release assets

### 4. Review and Publish the Release

1. Go to the **Releases** section of your repository
2. You'll see a draft release named `jvlauncher v0.1.0` (or your current version)
3. Review the release notes and attached files
4. Click **"Edit"** to modify the release notes if needed
5. Click **"Publish release"** when ready

## What Gets Built

The workflow creates the following files:

- `jvlauncher_0.1.0_aarch64.dmg` - For Apple Silicon Macs (M1/M2/M3)
- `jvlauncher_0.1.0_x64.dmg` - For Intel Macs
- `latest.json` - Updater manifest (if you configure Tauri's updater)

## Workflow Optimizations

The workflow is optimized for speed and cost-efficiency:

### Caching Strategy

1. **Rust Build Cache** (saves ~5-10 minutes)
   - Caches compiled Rust dependencies
   - Separate cache per architecture
   - Shared across branches

2. **Tauri CLI Cache** (saves ~2-3 minutes)
   - Caches the installed Tauri CLI binary
   - Avoids reinstalling on every build

3. **Concurrency Control**
   - Cancels outdated builds when you push new commits
   - Prevents wasted build minutes

### Expected Build Times

| Build Type | Duration |
|------------|----------|
| First build (no cache) | ~15-20 minutes per architecture |
| Subsequent builds (with cache) | ~5-8 minutes per architecture |
| No code changes (hot cache) | ~3-5 minutes per architecture |

Both architectures build in parallel, so total time ≈ time for one architecture.

## Versioning

The workflow reads the version from `src-tauri/tauri.conf.json`:

```json
{
  "version": "0.1.0"
}
```

**Before each release:**
1. Update the version number in `tauri.conf.json`
2. Update `CHANGELOG.md` with release notes
3. Commit the changes
4. Trigger the workflow

## Troubleshooting

### "Resource not accessible by integration" Error

**Problem:** Workflow can't create releases

**Solution:** Enable write permissions (see step 1 above)

### "Release already exists" Error

**Problem:** A release with the same version tag already exists

**Solutions:**
- Delete the existing release and tag, then re-run the workflow
- Update the version in `tauri.conf.json` to a new version

### Build Takes Too Long

**Check the logs:**
1. Go to **Actions** tab
2. Click on the failed/slow workflow run
3. Expand the steps to see which one is slow

**Common causes:**
- First build is always slow (no cache yet)
- Rust cache not working (check if `Cargo.lock` is committed)
- Network issues downloading dependencies

### Cache Not Working

**Verify cache is being used:**
1. Check workflow logs for "Cache restored successfully" messages
2. Go to **Actions** → **Caches** to see stored caches

**Clear cache if needed:**
1. Go to **Actions** → **Caches**
2. Delete old or corrupted caches
3. Re-run the workflow

## Advanced Configuration

### Add Code Signing (for distribution)

To sign your macOS app for distribution outside the App Store:

1. **Get an Apple Developer account** ($99/year)

2. **Create a Developer ID Application certificate**

3. **Export the certificate** as a `.p12` file

4. **Add GitHub Secrets:**
   - `APPLE_CERTIFICATE`: Base64-encoded certificate
   - `APPLE_CERTIFICATE_PASSWORD`: Certificate password
   - `APPLE_ID`: Your Apple ID email
   - `APPLE_PASSWORD`: App-specific password
   - `APPLE_TEAM_ID`: Your team ID

5. **Update the workflow** to include signing steps (see `.github/workflows/README.md`)

### Trigger on Tags or Branch Push

The workflow is currently set to **manual trigger only**. If you want to trigger automatically on tags or branch pushes, edit `.github/workflows/release.yml`:

```yaml
# Trigger on version tags
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
```

Then create releases with:
```bash
git tag v1.0.0
git push origin v1.0.0
```

### Auto-Publish Releases

Change `releaseDraft: true` to `releaseDraft: false` in the workflow to automatically publish releases without manual review.

⚠️ **Warning:** Only do this if you're confident in your build process!

### Add Windows/Linux Builds

To build for other platforms, add them to the matrix in the workflow:

```yaml
matrix:
  settings:
    - platform: 'macos-latest'
      target: 'aarch64-apple-darwin'
      arch: 'aarch64'
    - platform: 'macos-latest'
      target: 'x86_64-apple-darwin'
      arch: 'x64'
    - platform: 'ubuntu-22.04'
      target: ''
      arch: 'x64'
    - platform: 'windows-latest'
      target: ''
      arch: 'x64'
```

You'll also need to add platform-specific dependency installation steps.

## Cost Considerations

### GitHub Actions Pricing

- **Public repositories:** 2,000 free macOS minutes/month
- **Private repositories:** macOS minutes cost 10x Linux minutes

### Estimated Costs Per Release

| Scenario | Minutes Used | Linux-Equivalent Minutes |
|----------|--------------|--------------------------|
| First build (both architectures) | ~30-40 min | ~300-400 min |
| Cached build (both architectures) | ~10-16 min | ~100-160 min |

**For private repos:** At $0.008 per Linux minute, a cached release costs ~$0.80-$1.28

## Best Practices

1. ✅ **Test locally first** - Run `./build.sh` before pushing
2. ✅ **Use draft releases** - Review before publishing
3. ✅ **Update version** - Bump version in `tauri.conf.json` before each release
4. ✅ **Write changelogs** - Document what changed in each release
5. ✅ **Use semantic versioning** - Follow `MAJOR.MINOR.PATCH` format
6. ✅ **Tag releases** - Use git tags for version tracking
7. ✅ **Monitor builds** - Check the Actions tab for build status

## Next Steps

1. ✅ Enable workflow permissions (see step 1)
2. ✅ Commit and push the workflow files
3. ✅ Update version in `tauri.conf.json`
4. ✅ Create a release branch or manually trigger the workflow
5. ✅ Review and publish the draft release

## Resources

- [Workflow Documentation](.github/workflows/README.md) - Detailed workflow documentation
- [Tauri GitHub Actions Guide](https://v2.tauri.app/distribute/pipelines/github/)
- [tauri-action Repository](https://github.com/tauri-apps/tauri-action)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)

## Support

If you encounter issues:

1. Check the [Troubleshooting](#troubleshooting) section above
2. Review the workflow logs in the Actions tab
3. Check the [tauri-action issues](https://github.com/tauri-apps/tauri-action/issues)
4. Ask in the [Tauri Discord](https://discord.gg/tauri)

