#!/bin/bash

# Script to manually generate update files for Tauri updater
# This is needed when you manually upload releases instead of using GitHub Actions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Tauri Update File Generator ===${NC}"
echo ""

# Get version from Cargo.toml
VERSION=$(grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "Version: ${GREEN}${VERSION}${NC}"

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    TAURI_ARCH="aarch64"
elif [ "$ARCH" = "x86_64" ]; then
    TAURI_ARCH="x86_64"
else
    echo -e "${RED}Unsupported architecture: $ARCH${NC}"
    exit 1
fi

echo -e "Architecture: ${GREEN}${TAURI_ARCH}${NC}"
echo ""

# Find the DMG file
DMG_PATH="src-tauri/target/${TAURI_ARCH}-apple-darwin/release/bundle/dmg/jvlauncher_${VERSION}_${TAURI_ARCH}.dmg"

if [ ! -f "$DMG_PATH" ]; then
    echo -e "${RED}Error: DMG file not found at: $DMG_PATH${NC}"
    echo ""
    echo "Please build the app first with:"
    echo "  bun tauri build"
    exit 1
fi

echo -e "${GREEN}Found DMG:${NC} $DMG_PATH"

# Get file size
FILE_SIZE=$(stat -f%z "$DMG_PATH")
echo -e "File size: ${GREEN}${FILE_SIZE} bytes${NC}"

# Calculate SHA256
echo -e "\nCalculating SHA256 checksum..."
SHA256=$(shasum -a 256 "$DMG_PATH" | cut -d ' ' -f 1)
echo -e "SHA256: ${GREEN}${SHA256}${NC}"

# Sign the DMG if private key exists
PRIVATE_KEY_PATH="$HOME/.tauri/jvlauncher.key"
SIG_FILE="${DMG_PATH}.sig"

if [ -f "$PRIVATE_KEY_PATH" ]; then
    echo -e "\n${GREEN}Signing DMG...${NC}"
    
    # Check if tauri CLI is available
    if ! command -v cargo-tauri &> /dev/null; then
        echo -e "${YELLOW}Warning: tauri CLI not found. Installing...${NC}"
        cargo install tauri-cli
    fi
    
    # Sign the file
    cargo tauri signer sign "$DMG_PATH" -k "$PRIVATE_KEY_PATH" -p "${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}"
    
    if [ -f "$SIG_FILE" ]; then
        SIGNATURE=$(cat "$SIG_FILE")
        echo -e "Signature: ${GREEN}${SIGNATURE}${NC}"
    else
        echo -e "${RED}Error: Signature file not created${NC}"
        exit 1
    fi
else
    echo -e "\n${YELLOW}Warning: Private key not found at $PRIVATE_KEY_PATH${NC}"
    echo -e "${YELLOW}Skipping signature generation. Updates will work but won't be cryptographically verified.${NC}"
    echo ""
    echo "To generate a key pair, run:"
    echo "  cargo tauri signer generate -w ~/.tauri/jvlauncher.key"
    echo ""
    SIGNATURE=""
fi

# Get current date in ISO 8601 format
CURRENT_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# GitHub release URL
GITHUB_USER="join3r"
GITHUB_REPO="jvlauncher"
DOWNLOAD_URL="https://github.com/${GITHUB_USER}/${GITHUB_REPO}/releases/download/${VERSION}/jvlauncher_${VERSION}_${TAURI_ARCH}.dmg"

# Create latest.json
OUTPUT_DIR="src-tauri/target/release/bundle"
mkdir -p "$OUTPUT_DIR"
JSON_FILE="${OUTPUT_DIR}/latest.json"

echo -e "\n${GREEN}Generating latest.json...${NC}"

if [ -n "$SIGNATURE" ]; then
    # With signature
    cat > "$JSON_FILE" << EOF
{
  "version": "${VERSION}",
  "date": "${CURRENT_DATE}",
  "platforms": {
    "darwin-aarch64": {
      "signature": "${SIGNATURE}",
      "url": "${DOWNLOAD_URL}"
    }
  }
}
EOF
else
    # Without signature
    cat > "$JSON_FILE" << EOF
{
  "version": "${VERSION}",
  "date": "${CURRENT_DATE}",
  "platforms": {
    "darwin-aarch64": {
      "url": "${DOWNLOAD_URL}"
    }
  }
}
EOF
fi

echo -e "${GREEN}✓ Generated: ${JSON_FILE}${NC}"
echo ""
cat "$JSON_FILE"
echo ""

# Summary
echo -e "${GREEN}=== Summary ===${NC}"
echo ""
echo "Files to upload to GitHub Release ${VERSION}:"
echo -e "  1. ${GREEN}${DMG_PATH}${NC}"
if [ -f "$SIG_FILE" ]; then
    echo -e "  2. ${GREEN}${SIG_FILE}${NC}"
fi
echo -e "  3. ${GREEN}${JSON_FILE}${NC} (as 'latest.json')"
echo ""
echo -e "${YELLOW}Upload instructions:${NC}"
echo "  1. Go to: https://github.com/${GITHUB_USER}/${GITHUB_REPO}/releases/tag/${VERSION}"
echo "  2. Click 'Edit release'"
echo "  3. Upload the files listed above"
echo "  4. Make sure the DMG is named: jvlauncher_${VERSION}_${TAURI_ARCH}.dmg"
if [ -f "$SIG_FILE" ]; then
    echo "  5. Make sure the signature is named: jvlauncher_${VERSION}_${TAURI_ARCH}.dmg.sig"
    echo "  6. Make sure the JSON is named: latest.json"
else
    echo "  5. Make sure the JSON is named: latest.json"
fi
echo ""
echo -e "${GREEN}Done!${NC}"

