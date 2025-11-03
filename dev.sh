#!/bin/bash

# Development script for jvlauncher

echo "Starting jvlauncher in development mode..."

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

# Run in development mode
cd src-tauri
cargo tauri dev

