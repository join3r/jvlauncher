#!/bin/bash

# Verification script for App Launcher project

echo "🔍 Verifying App Launcher Project..."
echo ""

# Check for required tools
echo "Checking prerequisites..."

check_command() {
    if command -v $1 &> /dev/null; then
        echo "✅ $1 is installed"
        return 0
    else
        echo "❌ $1 is NOT installed"
        return 1
    fi
}

check_command "cargo" || echo "   Install from https://rustup.rs/"
check_command "rustc" || echo "   Install from https://rustup.rs/"

echo ""

# Check project structure
echo "Checking project structure..."

check_file() {
    if [ -f "$1" ]; then
        echo "✅ $1"
        return 0
    else
        echo "❌ $1 is missing"
        return 1
    fi
}

check_dir() {
    if [ -d "$1" ]; then
        echo "✅ $1/"
        return 0
    else
        echo "❌ $1/ is missing"
        return 1
    fi
}

# Backend files
check_file "src-tauri/src/main.rs"
check_file "src-tauri/src/commands.rs"
check_file "src-tauri/src/database.rs"
check_file "src-tauri/src/launcher.rs"
check_file "src-tauri/src/terminal.rs"
check_file "src-tauri/src/icon_extractor.rs"
check_file "src-tauri/src/shortcut_manager.rs"
check_file "src-tauri/Cargo.toml"
check_file "src-tauri/tauri.conf.json"
check_file "src-tauri/build.rs"

echo ""

# Frontend files
check_file "dist/index.html"
check_file "dist/app.js"
check_file "dist/styles.css"
check_file "dist/terminal.html"

echo ""

# Documentation
check_file "README.md"
check_file "SETUP.md"
check_file "QUICK_START.md"
check_file "ARCHITECTURE.md"
check_file "CONTRIBUTING.md"
check_file "CHANGELOG.md"
check_file "PROJECT_OVERVIEW.md"

echo ""

# Scripts
check_file "build.sh"
check_file "dev.sh"

echo ""

# Configuration
check_file "Cargo.toml"
check_file ".gitignore"

echo ""
echo "📊 Summary:"
echo ""

# Try to compile (but don't wait for completion)
echo "Testing Rust compilation (this may take a while)..."
cd src-tauri

if cargo check --quiet 2>/dev/null; then
    echo "✅ Project compiles successfully"
else
    echo "⚠️  Compilation check failed or incomplete"
    echo "   Run 'cd src-tauri && cargo check' for details"
fi

cd ..

echo ""
echo "✨ Verification complete!"
echo ""
echo "Next steps:"
echo "  1. Read QUICK_START.md for setup instructions"
echo "  2. Run ./dev.sh to start development"
echo "  3. Run ./build.sh to create production build"

