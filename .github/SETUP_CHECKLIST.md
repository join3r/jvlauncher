# GitHub Actions Setup Checklist

Use this checklist to set up and verify your GitHub Actions workflow for macOS releases.

## Pre-Flight Checklist

### ☐ 1. Repository Settings
- [ ] Go to **Settings** → **Actions** → **General**
- [ ] Scroll to **Workflow permissions**
- [ ] Select **"Read and write permissions"**
- [ ] Click **Save**

### ☐ 2. Version Configuration
- [ ] Open `src-tauri/tauri.conf.json`
- [ ] Verify the version is set correctly (e.g., `"version": "0.1.0"`)
- [ ] Update version if needed for your first release

### ☐ 3. Commit Workflow Files
```bash
# Add all workflow files
git add .github/

# Commit
git commit -m "Add GitHub Actions workflow for macOS releases"

# Push to master branch
git push origin master
```

## First Release Checklist

### ☐ 4. Prepare Release
- [ ] Update version in `src-tauri/tauri.conf.json`
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Test build locally: `./build.sh`
- [ ] Commit version changes

### ☐ 5. Trigger Workflow

**Manual Trigger (Only Option)**
- [ ] Go to **Actions** tab
- [ ] Click **"Release macOS"**
- [ ] Click **"Run workflow"**
- [ ] Select branch (usually `master`)
- [ ] Click **"Run workflow"**

### ☐ 6. Monitor Build
- [ ] Go to **Actions** tab
- [ ] Click on the running workflow
- [ ] Watch the build progress
- [ ] Check for any errors

**Expected duration:**
- First build: ~15-20 minutes per architecture
- Subsequent builds: ~5-8 minutes per architecture

### ☐ 7. Review Release
- [ ] Go to **Releases** section
- [ ] Find the draft release
- [ ] Verify both DMG files are attached:
  - `jvlauncher_*_aarch64.dmg` (Apple Silicon)
  - `jvlauncher_*_x64.dmg` (Intel)
- [ ] Review release notes
- [ ] Edit if needed

### ☐ 8. Publish Release
- [ ] Click **"Edit"** on the draft release
- [ ] Make any final changes
- [ ] Click **"Publish release"**

## Verification Checklist

### ☐ 9. Test Downloads
- [ ] Download the appropriate DMG for your Mac
- [ ] Open the DMG file
- [ ] Drag app to Applications
- [ ] Launch the app
- [ ] Verify it works correctly

### ☐ 10. Verify Workflow
- [ ] Check **Actions** → **Caches** for cached data
- [ ] Verify Rust cache was created
- [ ] Verify Tauri CLI cache was created

## Troubleshooting Checklist

### If Build Fails

#### ☐ Check Permissions
- [ ] Verify "Read and write permissions" is enabled
- [ ] Re-run the workflow

#### ☐ Check Logs
- [ ] Go to **Actions** tab
- [ ] Click on failed workflow
- [ ] Expand failed step
- [ ] Read error message
- [ ] Search for error in [tauri-action issues](https://github.com/tauri-apps/tauri-action/issues)

#### ☐ Check Version
- [ ] Verify version in `tauri.conf.json` is valid
- [ ] Ensure version doesn't already exist as a release
- [ ] Delete old release/tag if needed

#### ☐ Check Files
- [ ] Verify `src-tauri/Cargo.lock` is committed
- [ ] Verify `dist/` directory exists with frontend files
- [ ] Verify `src-tauri/tauri.conf.json` is valid JSON

### If Cache Not Working

#### ☐ Verify Cache Keys
- [ ] Check workflow logs for "Cache restored" messages
- [ ] Go to **Actions** → **Caches**
- [ ] Verify caches exist

#### ☐ Clear and Rebuild
- [ ] Go to **Actions** → **Caches**
- [ ] Delete all caches
- [ ] Re-run workflow
- [ ] Verify new caches are created

## Optimization Checklist

### ☐ 11. Monitor Performance
- [ ] Track build times in Actions tab
- [ ] Compare first build vs cached builds
- [ ] Expected improvement: 50-70% faster with cache

### ☐ 12. Monitor Costs (Private Repos)
- [ ] Go to **Settings** → **Billing**
- [ ] Check Actions minutes usage
- [ ] Verify costs are within budget

### ☐ 13. Review Cache Hit Rate
- [ ] Check workflow logs for cache hits
- [ ] Aim for >80% cache hit rate after first build
- [ ] Investigate if cache hit rate is low

## Maintenance Checklist

### Monthly
- [ ] Review and delete old releases if needed
- [ ] Check for workflow updates
- [ ] Update action versions if available

### Per Release
- [ ] Update version in `tauri.conf.json`
- [ ] Update `CHANGELOG.md`
- [ ] Test build locally
- [ ] Trigger workflow
- [ ] Review and publish release

### Quarterly
- [ ] Review build times and optimize if needed
- [ ] Update Rust dependencies: `cargo update`
- [ ] Update Tauri: Check for new versions
- [ ] Clear old caches

## Advanced Setup Checklist (Optional)

### ☐ Code Signing
- [ ] Get Apple Developer account ($99/year)
- [ ] Create Developer ID Application certificate
- [ ] Export certificate as `.p12`
- [ ] Add GitHub secrets:
  - `APPLE_CERTIFICATE`
  - `APPLE_CERTIFICATE_PASSWORD`
  - `APPLE_ID`
  - `APPLE_PASSWORD`
  - `APPLE_TEAM_ID`
- [ ] Update workflow with signing steps

### ☐ Auto-Updater
- [ ] Configure updater in `tauri.conf.json`
- [ ] Set up update server or use GitHub releases
- [ ] Test updater functionality
- [ ] Verify `latest.json` is generated

### ☐ Multi-Platform Builds
- [ ] Add Windows to matrix
- [ ] Add Linux to matrix
- [ ] Test builds on all platforms
- [ ] Update release notes for all platforms

## Success Criteria

Your setup is complete when:

- ✅ Workflow runs without errors
- ✅ Both DMG files are created and attached to release
- ✅ Cache is working (check logs for "Cache restored")
- ✅ Build time is optimized (5-8 min with cache)
- ✅ Release is published and downloadable
- ✅ App installs and runs correctly

## Quick Reference

### Trigger Release
```
Go to Actions → Release macOS → Run workflow → Select master → Run workflow
```

### Update Version
Edit `src-tauri/tauri.conf.json`:
```json
{
  "version": "1.0.0"
}
```

### View Workflow Status
```
Repository → Actions → Release macOS
```

### View Releases
```
Repository → Releases
```

### View Caches
```
Repository → Actions → Caches
```

## Need Help?

- 📖 [Setup Guide](../GITHUB_ACTIONS_SETUP.md)
- 📖 [Workflow Documentation](./workflows/README.md)
- 📖 [Workflow Summary](./WORKFLOW_SUMMARY.md)
- 🔗 [Tauri Docs](https://v2.tauri.app/distribute/pipelines/github/)
- 🔗 [tauri-action](https://github.com/tauri-apps/tauri-action)
- 💬 [Tauri Discord](https://discord.gg/tauri)

## Notes

- First build will take longer (no cache)
- Subsequent builds are much faster (cache hits)
- macOS runners cost 10x Linux minutes (private repos)
- Public repos get 2,000 free macOS minutes/month
- Cache expires after 7 days of inactivity
- Maximum cache size: 10GB per repository

