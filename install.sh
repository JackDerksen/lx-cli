#!/bin/bash

# Install script for lx-cli
# This script builds the project and copies the binary to ~/.local/bin/ so that
# you don't have to copy it over manually every time. Super usefull when testing
# during development!

set -e

echo "Building lx-cli..."
cargo build --release

# Create ~/.local/bin if it doesn't exist
mkdir -p ~/.local/bin

# Copy the binary
cp target/release/lx ~/.local/bin/

echo "✓ Installation complete!"
echo "The 'lx' binary has been installed to ~/.local/bin/"
echo ""
echo "Make sure ~/.local/bin is in your PATH by adding this to your shell config:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
