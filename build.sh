#!/bin/bash
set -e

# SuperTool Tauri build script
# Builds CLI binary first, then Tauri app

ARCH="${1:-native}"

echo "🔨 Building CLI (stool)..."
cd cli

case "$ARCH" in
  arm64|aarch64)
    echo "  → Target: aarch64-apple-darwin"
    cargo build --release --target aarch64-apple-darwin
    mkdir -p ../target/release
    cp target/aarch64-apple-darwin/release/stool ../target/release/stool
    ;;
  x64|x86_64)
    echo "  → Target: x86_64-apple-darwin"
    cargo build --release --target x86_64-apple-darwin
    mkdir -p ../target/release
    cp target/x86_64-apple-darwin/release/stool ../target/release/stool
    ;;
  native)
    echo "  → Target: native"
    cargo build --release
    ;;
  all)
    echo "  → Building universal binary..."
    cargo build --release --target aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
    mkdir -p ../target/release
    lipo -create -output ../target/release/stool \
      target/aarch64-apple-darwin/release/stool \
      target/x86_64-apple-darwin/release/stool
    ;;
  *)
    echo "Unknown architecture: $ARCH"
    echo "Usage: $0 [native|arm64|x64|all]"
    exit 1
    ;;
esac

cd ..

echo "✅ CLI build complete"
echo "📦 Starting Tauri build..."

# Run Tauri build
pnpm tauri build

echo "✅ Build complete!"
