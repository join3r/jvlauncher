#!/bin/bash

# jvlauncher Release Script
# This script automates the entire release process:
# 1. Version verification
# 2. Building the app
# 3. Signing the DMG
# 4. Creating GitHub release
# 5. Uploading assets with proper updater JSON

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="join3r/jvlauncher"
PLATFORM="darwin-aarch64"
TAURI_KEY_PATH="$HOME/.tauri/jvlauncher.key"
TAURI_PUBKEY_PATH="$HOME/.tauri/jvlauncher.key.pub"

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"

    # Check for required commands
    local missing_deps=()

    if ! command -v gh &> /dev/null; then
        missing_deps+=("gh (GitHub CLI)")
    fi

    if ! command -v bun &> /dev/null; then
        missing_deps+=("bun")
    fi

    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo")
    fi

    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing required dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        exit 1
    fi

    # Check for signing keys
    if [ ! -f "$TAURI_KEY_PATH" ]; then
        print_error "Signing key not found at $TAURI_KEY_PATH"
        print_info "Generate one with: cargo tauri signer generate -w ~/.tauri/jvlauncher.key"
        exit 1
    fi

    if [ ! -f "$TAURI_PUBKEY_PATH" ]; then
        print_error "Public key not found at $TAURI_PUBKEY_PATH"
        exit 1
    fi

    # Signing is optional - warn if not set
    if [ -z "$TAURI_SIGNING_PRIVATE_KEY_PASSWORD" ]; then
        print_warning "TAURI_SIGNING_PRIVATE_KEY_PASSWORD not set - releases will not be signed"
        print_info "This is OK for testing but not recommended for production"
    fi

    # Check GitHub CLI authentication
    if ! gh auth status &> /dev/null; then
        print_error "GitHub CLI not authenticated"
        print_info "Run: gh auth login"
        exit 1
    fi

    print_success "All prerequisites met"
}

# Get version from Cargo.toml
get_cargo_version() {
    grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Get version from tauri.conf.json
get_tauri_version() {
    jq -r '.version' src-tauri/tauri.conf.json
}

# Update version in tauri.conf.json
update_tauri_version() {
    local version=$1
    local temp_file=$(mktemp)
    jq --arg version "$version" '.version = $version' src-tauri/tauri.conf.json > "$temp_file"
    mv "$temp_file" src-tauri/tauri.conf.json
    print_success "Updated tauri.conf.json to version $version"
}

# Get public key content
get_pubkey() {
    cat "$TAURI_PUBKEY_PATH"
}

# Update pubkey in tauri.conf.json
update_pubkey() {
    local pubkey=$(get_pubkey)
    local temp_file=$(mktemp)
    jq --arg pubkey "$pubkey" '.plugins.updater.pubkey = $pubkey' src-tauri/tauri.conf.json > "$temp_file"
    mv "$temp_file" src-tauri/tauri.conf.json
    print_success "Updated pubkey in tauri.conf.json"
}

# Sign a file with the Tauri key
sign_file() {
    local file_path=$1
    local signature_path="${file_path}.sig"

    print_info "Signing $file_path..."

    # Read the password if not set
    if [ -z "$TAURI_SIGNING_PRIVATE_KEY_PASSWORD" ]; then
        print_info "Password not set in environment variable"
        print_info "Set it with: export TAURI_SIGNING_PRIVATE_KEY_PASSWORD='your-password'"
        print_error "Please set TAURI_SIGNING_PRIVATE_KEY_PASSWORD environment variable"
        exit 1
    fi

    # Sign the file using cargo tauri signer
    local sign_output
    sign_output=$(cargo tauri signer sign "$file_path" -k "$TAURI_KEY_PATH" -p "$TAURI_SIGNING_PRIVATE_KEY_PASSWORD" 2>&1)
    local sign_exit_code=$?

    if [ $sign_exit_code -ne 0 ]; then
        print_error "Failed to sign file"
        echo "$sign_output"
        return 1
    fi

    if [ ! -f "$signature_path" ]; then
        print_error "Signature file not created at $signature_path"
        return 1
    fi

    # Read the signature (it's in the .sig file)
    local signature=$(cat "$signature_path")
    echo "$signature"
}

# Create latest.json for updater
create_updater_json() {
    local version=$1
    local dmg_url=$2
    local signature=$3
    local output_file="latest.json"
    
    print_info "Creating updater JSON..."
    
    local current_date=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    cat > "$output_file" << EOF
{
  "version": "${version}",
  "date": "${current_date}",
  "platforms": {
    "${PLATFORM}": {
      "signature": "${signature}",
      "url": "${dmg_url}"
    }
  }
}
EOF
    
    print_success "Created $output_file"
    cat "$output_file"
}

# Main release process
main() {
    print_header "jvlauncher Release Script"
    
    # Check prerequisites
    check_prerequisites
    
    # Get current version
    CARGO_VERSION=$(get_cargo_version)
    TAURI_VERSION=$(get_tauri_version)
    
    print_info "Current version in Cargo.toml: $CARGO_VERSION"
    print_info "Current version in tauri.conf.json: $TAURI_VERSION"
    
    # Check if versions match
    if [ "$CARGO_VERSION" != "$TAURI_VERSION" ]; then
        print_warning "Version mismatch detected!"
        print_info "Syncing tauri.conf.json to match Cargo.toml ($CARGO_VERSION)"
        update_tauri_version "$CARGO_VERSION"
        TAURI_VERSION=$CARGO_VERSION
    fi
    
    VERSION=$CARGO_VERSION
    
    # Confirm version
    echo ""
    print_warning "About to release version: $VERSION"
    read -p "$(echo -e ${YELLOW}?${NC}) Is this correct? (y/N): " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Release cancelled"
        exit 1
    fi
    
    # Update pubkey in config
    print_header "Updating Configuration"
    update_pubkey
    
    # Build the app
    print_header "Building Application"
    print_info "Running: bun tauri build"
    
    if ! bun tauri build; then
        print_error "Build failed"
        exit 1
    fi
    
    print_success "Build completed"

    # Find the DMG file (check all possible locations)
    DMG_FILE=$(find target/release/bundle/dmg -name "*.dmg" 2>/dev/null | head -1)

    if [ -z "$DMG_FILE" ]; then
        DMG_FILE=$(find src-tauri/target/release/bundle/dmg -name "*.dmg" 2>/dev/null | head -1)
    fi

    if [ -z "$DMG_FILE" ]; then
        DMG_FILE=$(find target/aarch64-apple-darwin/release/bundle/dmg -name "*.dmg" 2>/dev/null | head -1)
    fi

    if [ -z "$DMG_FILE" ]; then
        DMG_FILE=$(find src-tauri/target/aarch64-apple-darwin/release/bundle/dmg -name "*.dmg" 2>/dev/null | head -1)
    fi

    if [ -z "$DMG_FILE" ]; then
        print_error "DMG file not found"
        print_info "Searched in:"
        echo "  - target/release/bundle/dmg/"
        echo "  - src-tauri/target/release/bundle/dmg/"
        echo "  - target/aarch64-apple-darwin/release/bundle/dmg/"
        echo "  - src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/"
        exit 1
    fi
    
    print_success "Found DMG: $DMG_FILE"
    
    # Sign the DMG (optional)
    print_header "Signing DMG"

    if [ -n "$TAURI_SIGNING_PRIVATE_KEY_PASSWORD" ]; then
        SIGNATURE=$(sign_file "$DMG_FILE")

        if [ -z "$SIGNATURE" ]; then
            print_warning "Failed to sign DMG - continuing without signature"
            SIGNATURE=""
        else
            print_success "DMG signed successfully"
            print_info "Signature: ${SIGNATURE:0:50}..."
        fi
    else
        print_warning "Skipping signing (no password set)"
        SIGNATURE=""
    fi
    
    # Create GitHub release
    print_header "Creating GitHub Release"
    
    TAG="v$VERSION"
    RELEASE_NAME="jvlauncher v$VERSION"
    DMG_FILENAME=$(basename "$DMG_FILE")
    
    # Check if release already exists
    if gh release view "$TAG" &> /dev/null; then
        print_warning "Release $TAG already exists"
        read -p "$(echo -e ${YELLOW}?${NC}) Delete and recreate? (y/N): " -n 1 -r
        echo ""
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_info "Deleting existing release..."
            gh release delete "$TAG" -y
            git push --delete origin "$TAG" 2>/dev/null || true
        else
            print_error "Release cancelled"
            exit 1
        fi
    fi
    
    print_info "Creating release $TAG..."
    
    gh release create "$TAG" \
        --title "$RELEASE_NAME" \
        --notes "## What's Changed

Download for Apple Silicon Macs (M1/M2/M3/M4):
- \`$DMG_FILENAME\`

See the assets below to download and install this version." \
        "$DMG_FILE"
    
    print_success "Release created"
    
    # Create and upload updater JSON
    print_header "Creating Updater JSON"
    
    DMG_URL="https://github.com/$REPO/releases/download/$TAG/$DMG_FILENAME"
    create_updater_json "$VERSION" "$DMG_URL" "$SIGNATURE"
    
    print_info "Uploading latest.json to release..."
    gh release upload "$TAG" latest.json --clobber
    
    print_success "Updater JSON uploaded"
    
    # Verify the upload
    print_header "Verification"
    
    LATEST_JSON_URL="https://github.com/$REPO/releases/download/$TAG/latest.json"
    print_info "Verifying latest.json is accessible..."
    
    sleep 2  # Give GitHub a moment to process
    
    if curl -sf "$LATEST_JSON_URL" > /dev/null; then
        print_success "latest.json is accessible at:"
        echo "  $LATEST_JSON_URL"
    else
        print_warning "latest.json may not be immediately accessible (CDN delay)"
    fi
    
    # Final summary
    print_header "Release Complete! 🎉"
    
    echo ""
    print_success "Version $VERSION has been released successfully!"
    echo ""
    print_info "Release URL: https://github.com/$REPO/releases/tag/$TAG"
    print_info "DMG URL: $DMG_URL"
    print_info "Updater JSON: $LATEST_JSON_URL"
    echo ""
    print_warning "Next steps:"
    echo "  1. Test the updater with an older version of the app"
    echo "  2. Verify the DMG downloads and installs correctly"
    echo "  3. Update version in Cargo.toml for next release"
    echo ""
}

# Run main function
main

