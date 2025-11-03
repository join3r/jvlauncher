#!/bin/bash

# Build script for jvlauncher

echo "Building jvlauncher..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if Tauri CLI is installed
if ! command -v cargo-tauri &> /dev/null; then
    echo "Installing Tauri CLI..."
    cargo install tauri-cli --version "^2.0.0"
fi

# Build the application
cd src-tauri

echo "Building release version..."
cargo tauri build

echo ""
echo "Build complete! The installer can be found in:"
echo "src-tauri/target/release/bundle/"

