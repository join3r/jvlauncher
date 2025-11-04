# Testing the Auto-Updater

## Current Setup

- **Current app version:** 0.1.3 (built for testing)
- **Latest release:** 0.1.4 (on GitHub)
- **Update endpoint:** https://github.com/join3r/jvlauncher/releases/latest/download/latest.json

## How to Test

### 1. Open the App

The app should already be running. If not:
```bash
open /Users/join3r/local/active/jvlauncher/target/release/bundle/macos/jvlauncher.app
```

### 2. Open Settings

- Click the menu bar icon (top right of screen)
- Select "Settings" from the menu

### 3. Check for Updates

In the Settings window:
1. Scroll to the "Updates" section
2. Click the "Check for Updates" button
3. Wait a few seconds

### Expected Results

✅ **If working correctly:**
- Status should show: "Update available: 0.1.4"
- Current version: 0.1.3
- Latest version: 0.1.4
- Release notes may be displayed

❌ **If still broken:**
- Error message about not being able to fetch updates
- Check the console logs for errors

### 4. View Logs

To see detailed logs:
```bash
# View app logs
log stream --predicate 'process == "jvlauncher"' --level debug
```

Or check the app's console output if running in dev mode.

### 5. Test Download (Optional)

If the update is detected:
1. Click "Download & Install" button
2. Wait for download to complete
3. App should install the update and prompt to restart

**Note:** This will actually download and install version 0.1.4!

## Verification Checklist

- [ ] App opens without errors
- [ ] Settings window opens
- [ ] "Check for Updates" button is visible
- [ ] Clicking button shows update is available (0.1.4)
- [ ] No error messages in the UI
- [ ] (Optional) Download and install works

## Troubleshooting

### Issue: "No updates available"

**Possible causes:**
1. The app version is already 0.1.4 or higher
2. Check `tauri.conf.json` - version should be 0.1.3

**Fix:**
```bash
# Check current version
grep '"version"' src-tauri/tauri.conf.json
```

### Issue: "Could not fetch a valid release JSON"

**Possible causes:**
1. `latest.json` is not accessible
2. Network issue
3. GitHub is down

**Fix:**
```bash
# Test endpoint manually
curl -sL https://github.com/join3r/jvlauncher/releases/latest/download/latest.json

# Should return:
# {
#   "version": "0.1.4",
#   "date": "2025-11-04T21:52:03Z",
#   "platforms": {
#     "darwin-aarch64": {
#       "url": "https://github.com/join3r/jvlauncher/releases/download/0.1.4/jvlauncher_0.1.4_aarch64.dmg"
#     }
#   }
# }
```

### Issue: "Signature verification failed"

**This should NOT happen** since we removed the `pubkey` from the config.

If it does happen:
```bash
# Check tauri.conf.json
grep -A 10 '"updater"' src-tauri/tauri.conf.json

# Should NOT contain "pubkey" field
```

## After Testing

Once you've confirmed the updater works, restore the version to 0.1.5:

```bash
# Edit src-tauri/tauri.conf.json
# Change version from "0.1.3" to "0.1.5"
```

Or use this command:
```bash
sed -i '' 's/"version": "0.1.3"/"version": "0.1.5"/' src-tauri/tauri.conf.json
```

## Success Criteria

The updater is working if:
1. ✅ App detects version 0.1.4 is available
2. ✅ No error messages
3. ✅ Download button appears
4. ✅ (Optional) Download and install completes successfully

## Next Steps After Successful Test

1. Restore version to 0.1.5 in `tauri.conf.json`
2. Create a new release (0.1.5) using GitHub Actions or the helper scripts
3. The updater will then work for all users!

