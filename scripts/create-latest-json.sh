#!/bin/bash

# Quick script to generate latest.json for an existing release
# Usage: ./scripts/create-latest-json.sh [version] [dmg-path]

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get version from argument or Cargo.toml
if [ -n "$1" ]; then
    VERSION="$1"
else
    VERSION=$(grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
fi

echo -e "${GREEN}Creating latest.json for version: ${VERSION}${NC}"

# Get DMG path from argument or find it
if [ -n "$2" ]; then
    DMG_PATH="$2"
else
    # Try to find the DMG in common locations
    ARCH=$(uname -m)
    if [ "$ARCH" = "arm64" ]; then
        TAURI_ARCH="aarch64"
    else
        TAURI_ARCH="x86_64"
    fi
    
    DMG_PATH="src-tauri/target/${TAURI_ARCH}-apple-darwin/release/bundle/dmg/jvlauncher_${VERSION}_${TAURI_ARCH}.dmg"
fi

# GitHub info
GITHUB_USER="join3r"
GITHUB_REPO="jvlauncher"

# Determine architecture from filename
if [[ "$DMG_PATH" == *"aarch64"* ]]; then
    PLATFORM="darwin-aarch64"
elif [[ "$DMG_PATH" == *"x86_64"* ]]; then
    PLATFORM="darwin-x86_64"
else
    PLATFORM="darwin-aarch64"  # default
fi

DOWNLOAD_URL="https://github.com/${GITHUB_USER}/${GITHUB_REPO}/releases/download/${VERSION}/$(basename "$DMG_PATH")"
CURRENT_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Check if signature file exists
SIG_FILE="${DMG_PATH}.sig"
if [ -f "$SIG_FILE" ]; then
    SIGNATURE=$(cat "$SIG_FILE")
    echo -e "${GREEN}Found signature file${NC}"
else
    SIGNATURE=""
    echo -e "${YELLOW}No signature file found (updates will work but won't be verified)${NC}"
fi

# Create latest.json
OUTPUT_FILE="latest.json"

if [ -n "$SIGNATURE" ]; then
    cat > "$OUTPUT_FILE" << EOF
{
  "version": "${VERSION}",
  "date": "${CURRENT_DATE}",
  "platforms": {
    "${PLATFORM}": {
      "signature": "${SIGNATURE}",
      "url": "${DOWNLOAD_URL}"
    }
  }
}
EOF
else
    # Include empty signature field (required by Tauri updater even when pubkey is empty)
    cat > "$OUTPUT_FILE" << EOF
{
  "version": "${VERSION}",
  "date": "${CURRENT_DATE}",
  "platforms": {
    "${PLATFORM}": {
      "signature": "",
      "url": "${DOWNLOAD_URL}"
    }
  }
}
EOF
fi

echo -e "${GREEN}✓ Created: ${OUTPUT_FILE}${NC}"
echo ""
cat "$OUTPUT_FILE"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Upload this file to your GitHub release as 'latest.json'"
echo "2. Make sure the DMG is also uploaded with the exact name: $(basename "$DMG_PATH")"
if [ -f "$SIG_FILE" ]; then
    echo "3. Upload the signature file: $(basename "$SIG_FILE")"
fi
echo ""
echo "Release URL: https://github.com/${GITHUB_USER}/${GITHUB_REPO}/releases/tag/${VERSION}"

