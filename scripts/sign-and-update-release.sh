#!/bin/bash

# Script to sign an existing release and update latest.json
# Usage: ./scripts/sign-and-update-release.sh [version]

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Get version
VERSION="${1:-$(grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')}"

echo -e "${GREEN}=== Signing Release ${VERSION} ===${NC}\n"

# Configuration
GITHUB_USER="join3r"
GITHUB_REPO="jvlauncher"
PRIVATE_KEY="$HOME/.tauri/jvlauncher.key"
TEMP_DIR=$(mktemp -d)

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    TAURI_ARCH="aarch64"
    PLATFORM="darwin-aarch64"
else
    TAURI_ARCH="x86_64"
    PLATFORM="darwin-x86_64"
fi

DMG_NAME="jvlauncher_${VERSION}_${TAURI_ARCH}.dmg"
DMG_URL="https://github.com/${GITHUB_USER}/${GITHUB_REPO}/releases/download/${VERSION}/${DMG_NAME}"

echo -e "Version: ${GREEN}${VERSION}${NC}"
echo -e "Platform: ${GREEN}${PLATFORM}${NC}"
echo -e "DMG: ${GREEN}${DMG_NAME}${NC}\n"

# Check if private key exists
if [ ! -f "$PRIVATE_KEY" ]; then
    echo -e "${RED}Error: Private key not found at $PRIVATE_KEY${NC}"
    echo ""
    echo "Generate a key pair with:"
    echo "  cargo tauri signer generate -w ~/.tauri/jvlauncher.key"
    exit 1
fi

# Check if tauri CLI is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found${NC}"
    exit 1
fi

# Install tauri-cli if needed
if ! cargo tauri signer --help &> /dev/null; then
    echo -e "${YELLOW}Installing tauri-cli...${NC}"
    cargo install tauri-cli --locked
fi

# Download the DMG
echo -e "${GREEN}Downloading DMG...${NC}"
cd "$TEMP_DIR"
curl -L -o "$DMG_NAME" "$DMG_URL"

if [ ! -f "$DMG_NAME" ]; then
    echo -e "${RED}Error: Failed to download DMG${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Downloaded${NC}\n"

# Sign the DMG
echo -e "${GREEN}Signing DMG...${NC}"
cargo tauri signer sign "$DMG_NAME" -k "$PRIVATE_KEY" ${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:+-p "$TAURI_SIGNING_PRIVATE_KEY_PASSWORD"}

if [ ! -f "${DMG_NAME}.sig" ]; then
    echo -e "${RED}Error: Signature file not created${NC}"
    exit 1
fi

SIGNATURE=$(cat "${DMG_NAME}.sig")
echo -e "${GREEN}✓ Signed${NC}"
echo -e "Signature: ${SIGNATURE}\n"

# Create latest.json with signature
CURRENT_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > latest.json << EOF
{
  "version": "${VERSION}",
  "date": "${CURRENT_DATE}",
  "platforms": {
    "${PLATFORM}": {
      "signature": "${SIGNATURE}",
      "url": "${DMG_URL}"
    }
  }
}
EOF

echo -e "${GREEN}Created latest.json:${NC}"
cat latest.json
echo ""

# Upload signature and latest.json to GitHub release
echo -e "${GREEN}Uploading files to GitHub release...${NC}"

cd "$TEMP_DIR"
gh release upload "${VERSION}" "${DMG_NAME}.sig" --clobber -R "${GITHUB_USER}/${GITHUB_REPO}"
gh release upload "${VERSION}" latest.json --clobber -R "${GITHUB_USER}/${GITHUB_REPO}"

echo -e "${GREEN}✓ Uploaded${NC}\n"

# Copy latest.json to project root for reference
cp latest.json "/Users/join3r/local/active/jvlauncher/latest.json"

# Cleanup
cd /
rm -rf "$TEMP_DIR"

echo -e "${GREEN}=== Done! ===${NC}\n"
echo "Release ${VERSION} is now signed and ready for auto-updates."
echo ""
echo "Verify at: https://github.com/${GITHUB_USER}/${GITHUB_REPO}/releases/tag/${VERSION}"
echo ""
echo "Test the updater endpoint:"
echo "  curl https://github.com/${GITHUB_USER}/${GITHUB_REPO}/releases/latest/download/latest.json"

